// Read-only Carbon source inventory and active-layout Cmd+V resolution.
import Carbon
import Foundation

// Carbon virtual keycodes span the first 128 codes; four UTF-16 slots bound one translation.
let virtualKeycodeCount: UInt16 = 128
let translationCapacity = 4
// UCKeyTranslate consumes Carbon's cmdKey shifted right by eight bits.
let commandModifier: UInt32 = 1
let lowercaseV: UniChar = 118
let preferredLayoutIDs = ["com.apple.keylayout.ABC", "com.apple.keylayout.US", "com.apple.keylayout.Dvorak"]

/// Reads a borrowed TIS string while its owning source remains retained.
func stringProperty(_ source: TISInputSource, _ key: CFString) -> String? {
    guard let pointer = TISGetInputSourceProperty(source, key) else { return nil }
    // SAFETY: TIS property pointers are borrowed CF objects; the retained source keeps them alive during type validation.
    let borrowed = Unmanaged<CFTypeRef>.fromOpaque(pointer).takeUnretainedValue()
    guard CFGetTypeID(borrowed) == CFStringGetTypeID() else { return nil }
    // SAFETY: The runtime type check establishes CFString before the typed conversion.
    return Unmanaged<CFString>.fromOpaque(pointer).takeUnretainedValue() as String
}

/// Reads a borrowed TIS Boolean without changing the source configuration.
func booleanProperty(_ source: TISInputSource, _ key: CFString) -> Bool? {
    guard let pointer = TISGetInputSourceProperty(source, key) else { return nil }
    // SAFETY: The owning source retains this borrowed CF object during runtime type validation.
    let borrowed = Unmanaged<CFTypeRef>.fromOpaque(pointer).takeUnretainedValue()
    guard CFGetTypeID(borrowed) == CFBooleanGetTypeID() else { return nil }
    // SAFETY: The runtime type check establishes CFBoolean before reading its value.
    return CFBooleanGetValue(Unmanaged<CFBoolean>.fromOpaque(pointer).takeUnretainedValue())
}

/// Lists available sources; the copied array owns each borrowed entry during iteration.
func listSources(includeDisabled: Bool) throws -> [TISInputSource] {
    guard let copied = TISCreateInputSourceList(nil, includeDisabled) else {
        throw ProbeError.unavailable("source_list")
    }
    let sources = copied.takeRetainedValue()
    guard CFGetTypeID(sources) == CFArrayGetTypeID() else {
        throw ProbeError.unavailable("source_list_type")
    }
    return try (0..<CFArrayGetCount(sources)).map { index in
        guard let pointer = CFArrayGetValueAtIndex(sources, index) else {
            throw ProbeError.unavailable("source_entry")
        }
        // SAFETY: Validated Carbon CF arrays own their borrowed CF entries through this type check.
        let borrowed = Unmanaged<CFTypeRef>.fromOpaque(pointer).takeUnretainedValue()
        guard CFGetTypeID(borrowed) == TISInputSourceGetTypeID() else {
            throw ProbeError.unavailable("source_entry_type")
        }
        // SAFETY: Carbon's runtime type check establishes TISInputSource before conversion; ARC retains the result.
        return Unmanaged<TISInputSource>.fromOpaque(pointer).takeUnretainedValue()
    }
}

/// Describes source metadata only; no user text or clipboard content is inspected.
func describe(_ source: TISInputSource) throws -> [String: String] {
    guard let id = stringProperty(source, kTISPropertyInputSourceID),
          let category = stringProperty(source, kTISPropertyInputSourceCategory),
          let type = stringProperty(source, kTISPropertyInputSourceType) else {
        throw ProbeError.unavailable("source_metadata")
    }
    return ["id": id, "category": category, "type": type]
}

/// Resolves physical keys producing V with Command using the current native layout, without posting events.
func resolveCommandV(_ source: TISInputSource) throws -> [UInt16] {
    guard let pointer = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) else {
        throw ProbeError.unavailable("layout_data")
    }
    // SAFETY: The source retains the borrowed CF object during runtime type validation.
    let borrowed = Unmanaged<CFTypeRef>.fromOpaque(pointer).takeUnretainedValue()
    guard CFGetTypeID(borrowed) == CFDataGetTypeID() else {
        throw ProbeError.unavailable("layout_data_type")
    }
    // SAFETY: The runtime check establishes CFData; its owning source keeps the bytes alive through translation.
    let data = Unmanaged<CFData>.fromOpaque(pointer).takeUnretainedValue()
    guard let bytes = CFDataGetBytePtr(data), CFDataGetLength(data) > 0 else {
        throw ProbeError.unavailable("layout_bytes")
    }
    // SAFETY: Carbon supplies UCKeyboardLayout bytes; output buffers below have the announced capacities.
    let layout = UnsafeRawPointer(bytes).assumingMemoryBound(to: UCKeyboardLayout.self)
    return (0..<virtualKeycodeCount).filter { keycode in
        var deadKeys: UInt32 = 0
        var length = 0
        var characters = [UniChar](repeating: 0, count: translationCapacity)
        let status = UCKeyTranslate(
            layout, keycode, UInt16(kUCKeyActionDisplay), commandModifier,
            UInt32(LMGetKbdType()), UInt32(kUCKeyTranslateNoDeadKeysMask),
            &deadKeys, characters.count, &length, &characters
        )
        return status == noErr && length == 1 && characters[0] == lowercaseV
    }
}

/// Reports native API failures without copied text or other private values.
enum ProbeError: Error { case unavailable(String) }

// The executable only reads source metadata/layout data and emits one JSON object; no enable/select/post API is linked here.
do {
    guard Thread.isMainThread,
          let currentCopy = TISCopyCurrentKeyboardInputSource(),
          let layoutCopy = TISCopyCurrentKeyboardLayoutInputSource() else {
        throw ProbeError.unavailable("current_source")
    }
    let current = currentCopy.takeRetainedValue()
    let layout = layoutCopy.takeRetainedValue()
    let enabledIDs = try listSources(includeDisabled: false).map { source in
        guard let id = try describe(source)["id"], !id.isEmpty else {
            throw ProbeError.unavailable("source_id")
        }
        return id
    }
    let candidates: [[String: Any]] = try listSources(includeDisabled: true).compactMap { source in
        let row = try describe(source)
        guard let id = row["id"], !id.isEmpty else {
            throw ProbeError.unavailable("source_id")
        }
        guard preferredLayoutIDs.contains(id) else { return nil }
        guard let enabled = booleanProperty(source, kTISPropertyInputSourceIsEnabled),
              let selectable = booleanProperty(source, kTISPropertyInputSourceIsSelectCapable) else {
            throw ProbeError.unavailable("candidate_metadata")
        }
        return ["id": id, "enabled": enabled, "selectable": selectable]
    }
    let report: [String: Any] = [
        "feature": "007-smart-paste",
        "captured_at": ISO8601DateFormatter().string(from: Date()),
        "mode": "read_only_native_carbon_swift",
        "platform": ["product": "macOS", "version": ProcessInfo.processInfo.operatingSystemVersionString],
        "current_input_source": try describe(current),
        "current_layout": try describe(layout),
        "enabled_source_ids": enabledIDs,
        "installed_candidates": candidates,
        "command_v_keycodes": try resolveCommandV(layout),
        "physical_keyboard_type": LMGetKbdType(),
        "events_posted": 0,
        "layout_changed": false,
        "selection_test": "pending",
        "layout_restore_test": "pending",
        "timing_test": "pending",
        "delivery_test": "pending",
        "observation_boundary": "Only inventory and current-layout Cmd+V resolution observed; no enable, selection or event posting occurred."
    ]
    let encoded = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
    FileHandle.standardOutput.write(encoded)
    FileHandle.standardOutput.write(Data([10]))
} catch {
    FileHandle.standardError.write(Data("Keyboard read-only probe failed: \(error)\n".utf8))
    exit(1)
}
