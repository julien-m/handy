// N16 activation diagnostic. Runs as an accessory (no Dock icon), inactive process, like Handy.
// For each strategy: put Finder in front with `open -a Finder` (LaunchServices, no TCC), then
// try to bring TextEdit forward and sample the frontmost application for 1 s (the AC-002 budget).
// It types nothing, pastes nothing, and leaves TextEdit running if it was running; it quits
// TextEdit only if it launched it.
// Usage: activation-diag [target-bundle] [other-bundle] [--hide|--user-click|--foreign-switch|--manual-switch]
import AppKit
import ApplicationServices

let targetBundle = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "com.apple.TextEdit"
let budget: TimeInterval = 1.0

setbuf(stdout, nil)
let app = NSApplication.shared
app.setActivationPolicy(.accessory)

// Switcher mode: a separately launched process (own responsibility) brings `bundle` forward,
// so the diagnostic process below did not cause the current activation.
if let index = CommandLine.arguments.firstIndex(of: "--switch-to"), index + 1 < CommandLine.arguments.count {
    let bundle = CommandLine.arguments[index + 1]
    let accepted = NSRunningApplication.runningApplications(withBundleIdentifier: bundle).first?.activate(options: []) ?? false
    print("switcher accepted: \(accepted)")
    exit(0)
}

func frontmostBundle() -> String {
    // Spin the run loop so NSWorkspace's cached frontmostApplication is refreshed.
    RunLoop.main.run(until: Date().addingTimeInterval(0.01))
    return NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "?"
}

func axFocused() -> String {
    let system = AXUIElementCreateSystemWide()
    var value: CFTypeRef?
    let status = AXUIElementCopyAttributeValue(system, "AXFocusedApplication" as CFString, &value)
    guard status == .success, let element = value else { return "err\(status.rawValue)" }
    var pid: pid_t = 0
    AXUIElementGetPid(element as! AXUIElement, &pid)
    return "pid\(pid)"
}
// Timeline of the live AX focused-application read for `window` seconds after a strategy.
func axTimeline(_ window: TimeInterval) -> String {
    let start = Date()
    var last = ""
    var events: [String] = []
    while Date().timeIntervalSince(start) <= window {
        let now = axFocused()
        if now != last { events.append("\(Int(Date().timeIntervalSince(start) * 1000))ms:\(now)"); last = now }
        RunLoop.main.run(until: Date().addingTimeInterval(0.02))
    }
    return events.joined(separator: " ")
}

func shell(_ args: [String]) {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    process.arguments = args
    try? process.run()
    process.waitUntilExit()
}

func waitFront(_ bundle: String, timeout: TimeInterval) -> TimeInterval? {
    let start = Date()
    while Date().timeIntervalSince(start) <= timeout {
        if frontmostBundle() == bundle { return Date().timeIntervalSince(start) }
        RunLoop.main.run(until: Date().addingTimeInterval(0.02))
    }
    return nil
}

let otherBundle = CommandLine.arguments.count > 2 ? CommandLine.arguments[2] : "com.apple.finder"
let userSwitch = CommandLine.arguments.contains("--user-click")
let foreignSwitch = CommandLine.arguments.contains("--foreign-switch")
let switcherApp = URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent().appendingPathComponent("ActivationDiag.app").path
// Center of the Dock item of `bundle`, read through the Dock's Accessibility tree.
func dockItemCenter(_ bundle: String) -> CGPoint? {
    guard let dock = NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.dock").first,
          let url = NSRunningApplication.runningApplications(withBundleIdentifier: bundle).first?.bundleURL else { return nil }
    func children(_ element: AXUIElement) -> [AXUIElement] {
        var value: CFTypeRef?
        AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &value)
        return (value as? [AXUIElement]) ?? []
    }
    for list in children(AXUIElementCreateApplication(dock.processIdentifier)) {
        for item in children(list) {
            var itemURL: CFTypeRef?
            AXUIElementCopyAttributeValue(item, kAXURLAttribute as CFString, &itemURL)
            guard let itemURL = itemURL as? URL, itemURL.standardizedFileURL == url.standardizedFileURL else { continue }
            var pos: CFTypeRef?, size: CFTypeRef?
            AXUIElementCopyAttributeValue(item, kAXPositionAttribute as CFString, &pos)
            AXUIElementCopyAttributeValue(item, kAXSizeAttribute as CFString, &size)
            var p = CGPoint.zero, s = CGSize.zero
            AXValueGetValue(pos as! AXValue, .cgPoint, &p)
            AXValueGetValue(size as! AXValue, .cgSize, &s)
            return CGPoint(x: p.x + s.width / 2, y: p.y + s.height / 2)
        }
    }
    return nil
}
// A HID-level click on the top edge of the other app's first window: the WindowServer sees it as
// user input switching apps (no key is typed; the top edge holds no control).
func clickWindowEdge(_ bundle: String) -> Bool {
    guard let other = NSRunningApplication.runningApplications(withBundleIdentifier: bundle).first else { return false }
    // The app's first on-screen normal window (layer 0), from the window server.
    let list = (CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]]) ?? []
    guard let info = list.first(where: { ($0[kCGWindowOwnerPID as String] as? Int32) == other.processIdentifier && ($0[kCGWindowLayer as String] as? Int) == 0 }),
          let bounds = info[kCGWindowBounds as String] as? [String: CGFloat] else { return false }
    let p = CGPoint(x: bounds["X"] ?? 0, y: bounds["Y"] ?? 0), s = CGSize(width: bounds["Width"] ?? 0, height: bounds["Height"] ?? 0)
    let point = CGPoint(x: p.x + s.width / 2, y: p.y + 4)
    let saved = CGEvent(source: nil)?.location ?? point
    for type in [CGEventType.leftMouseDown, .leftMouseUp] {
        CGEvent(mouseEventSource: nil, mouseType: type, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
        usleep(30_000)
    }
    CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: saved, mouseButton: .left)?.post(tap: .cghidEventTap)
    return true
}
// Native validation mode: the person switches to the other app by hand (click or Cmd+Tab), the
// condition under which macOS 14+ declined Handy's workspace request (N16).
let manualSwitch = CommandLine.arguments.contains("--manual-switch")
func waitForManualSwitch() -> Bool {
    print("  -> switch to \(otherBundle) by hand now (click it or Cmd+Tab), within 20 s")
    guard waitFront(otherBundle, timeout: 20.0) != nil else { return false }
    // Leave the person a moment in the other app, as during a dictation.
    RunLoop.main.run(until: Date().addingTimeInterval(1.0))
    return frontmostBundle() == otherBundle
}
func putFinderInFront() -> Bool {
    if manualSwitch { return waitForManualSwitch() }
    if userSwitch { guard clickWindowEdge(otherBundle) else { return false } }
    else if foreignSwitch { shell(["open", "-n", "-W", switcherApp, "--args", "--switch-to", otherBundle]) }
    else { shell(["open", "-b", otherBundle]) }
    let inFront = waitFront(otherBundle, timeout: 2.0) != nil
    // Let the switch settle as it would while the user keeps working in the other app.
    RunLoop.main.run(until: Date().addingTimeInterval(0.3))
    return inFront
}

func target() -> NSRunningApplication {
    NSRunningApplication.runningApplications(withBundleIdentifier: targetBundle).first!
}

func axSetFrontmost(_ pid: pid_t) -> AXError {
    let element = AXUIElementCreateApplication(pid)
    return AXUIElementSetAttributeValue(element, kAXFrontmostAttribute as CFString, kCFBooleanTrue)
}

func axRaiseMainWindow(_ pid: pid_t) -> AXError {
    let element = AXUIElementCreateApplication(pid)
    var windows: CFTypeRef?
    let status = AXUIElementCopyAttributeValue(element, kAXWindowsAttribute as CFString, &windows)
    guard status == .success, let list = windows as? [AXUIElement], let first = list.first else {
        return status == .success ? .noValue : status
    }
    return AXUIElementPerformAction(first, kAXRaiseAction as CFString)
}

func onBackground(_ work: @escaping () -> String) -> String {
    var result = ""
    let done = DispatchSemaphore(value: 0)
    Thread.detachNewThread {
        result = work()
        done.signal()
    }
    while done.wait(timeout: .now()) == .timedOut {
        RunLoop.main.run(until: Date().addingTimeInterval(0.005))
    }
    return result
}

let strategies: [(String, () -> String)] = [
    ("activate(options: []) main thread", { "returned \(target().activate(options: []))" }),
    ("activate(options: []) background thread", {
        onBackground { "returned \(target().activate(options: []))" }
    }),
    ("activate(options: .activateIgnoringOtherApps) main (deprecated)", {
        "returned \(target().activate(options: [.activateIgnoringOtherApps]))"
    }),
    ("activate(from: current, options: []) main (macOS 14 API)", {
        "returned \(target().activate(from: NSRunningApplication.current, options: []))"
    }),
    ("NSApp.yieldActivation(to:) + activate() main", {
        NSApp.yieldActivation(to: target())
        return "returned \(target().activate(options: []))"
    }),
    ("AX kAXFrontmostAttribute=true main thread", { "AXError \(axSetFrontmost(target().processIdentifier).rawValue)" }),
    ("AX kAXFrontmostAttribute=true background thread", {
        onBackground { "AXError \(axSetFrontmost(target().processIdentifier).rawValue)" }
    }),
    ("AXRaise first window only", { "AXError \(axRaiseMainWindow(target().processIdentifier).rawValue)" }),
]

let initialFront = frontmostBundle()
let launchedByUs = NSRunningApplication.runningApplications(withBundleIdentifier: targetBundle).isEmpty
print("macOS \(ProcessInfo.processInfo.operatingSystemVersionString)")
print("AXIsProcessTrusted (inherited from the hosting app): \(AXIsProcessTrusted())")
print("user-like window click switch: \(userSwitch); foreign switcher process: \(foreignSwitch); manual switch: \(manualSwitch)")
print("diag isActive: \(NSApp.isActive); initial front: \(initialFront); TextEdit launched by diag: \(launchedByUs)")
if launchedByUs {
    shell(["open", "-g", "-a", "TextEdit"])
    _ = waitFront("__never__", timeout: 1.5)
}

let attempts = manualSwitch ? 1 : 3
if CommandLine.arguments.contains("--hide") { NSApp.hide(nil); RunLoop.main.run(until: Date().addingTimeInterval(0.2)); print("diag hidden: \(NSApp.isHidden)") }
for (name, run) in strategies {
    var successes = 0
    var details: [String] = []
    for _ in 0..<attempts {
        guard putFinderInFront() else { details.append("finder-not-front"); continue }
        let call = run()
        let elapsed = waitFront(targetBundle, timeout: budget)
        let ax = axTimeline(1.5)
        if let elapsed { successes += 1; details.append("\(call) -> front in \(Int(elapsed * 1000)) ms [AX after: \(ax)]") }
        else { details.append("\(call) -> NOT front after 1 s, front=\(frontmostBundle()) [AX after: \(ax)]") }
    }
    print("\n[\(successes)/\(attempts)] \(name)")
    details.forEach { print("    \($0)") }
}
print("\ntarget pid: \(target().processIdentifier)")

// Restore: quit TextEdit only if the diagnostic launched it, then return to the initial front app.
if launchedByUs { _ = target().terminate() }
if initialFront != "?" { shell(["open", "-b", initialFront]) }
print("restored front: \(waitFront(initialFront, timeout: 2.0) != nil ? initialFront : frontmostBundle())")
