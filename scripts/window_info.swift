import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 2,
      let pid = Int(CommandLine.arguments[1]) else {
    fputs("usage: window_info.swift PID\n", stderr)
    exit(2)
}

let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] ?? []
let candidates: [[String: Int]] = windows.compactMap { window in
    guard (window[kCGWindowOwnerPID as String] as? Int) == pid,
          (window[kCGWindowLayer as String] as? Int) == 0,
          let number = window[kCGWindowNumber as String] as? Int,
          let bounds = window[kCGWindowBounds as String] as? [String: Int],
          let width = bounds["Width"], width > 100,
          let height = bounds["Height"], height > 100 else {
        return nil
    }
    return ["id": number, "width": width, "height": height]
}

guard let largest = candidates.max(by: { ($0["width"]! * $0["height"]!) < ($1["width"]! * $1["height"]!) }),
      let data = try? JSONSerialization.data(withJSONObject: largest),
      let json = String(data: data, encoding: .utf8) else {
    fputs("no visible application window for PID \(pid)\n", stderr)
    exit(1)
}
print(json)
