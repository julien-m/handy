//
//  macos-window-list.swift
//  Handy settings native proof helper (feature 008, Step 7).
//
// Lists the on-screen windows owned by the given process IDs as JSON lines
// ({"pid":N,"layer":L,"width":W,"height":H}), using CGWindowListCopyWindowInfo. Window
// titles are not read (that needs Screen Recording permission). Layer 0 is a normal window;
// layer 25 is a menu-bar status item, which macOS 26 lists with empty bounds whether the icon
// is visible or not, so the tray is checked from the app log instead. Used by
// scripts/settings-native-proof.ts.
// Usage: swift scripts/macos-window-list.swift PID [PID...]
import CoreGraphics
import Foundation

let pids = Set(CommandLine.arguments.dropFirst().compactMap { Int32($0) })
let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
let windows = (CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]]) ?? []
for window in windows {
    guard let pid = window[kCGWindowOwnerPID as String] as? Int32, pids.contains(pid) else { continue }
    let layer = window[kCGWindowLayer as String] as? Int ?? -1
    let bounds = window[kCGWindowBounds as String] as? [String: Any] ?? [:]
    let width = bounds["Width"] as? Double ?? 0
    let height = bounds["Height"] as? Double ?? 0
    print("{\"pid\":\(pid),\"layer\":\(layer),\"width\":\(Int(width)),\"height\":\(Int(height))}")
}
