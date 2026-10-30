import Foundation

/// How a response body is presented on the Pretty tab.
enum ResponseKind: Equatable {
    case json, html, image, text, binary

    static func classify(contentType: String, bodyIsBase64: Bool) -> ResponseKind {
        let type = contentType.split(separator: ";").first
            .map { $0.trimmingCharacters(in: .whitespaces).lowercased() } ?? ""
        if bodyIsBase64 {
            return type.hasPrefix("image/") ? .image : .binary
        }
        if type == "application/json" || type.hasSuffix("+json") { return .json }
        if type == "text/html" || type == "application/xhtml+xml" { return .html }
        return .text
    }
}

/// Canonical hex dump (offset, 16 bytes in two groups of 8, ASCII gutter) of the first 64 KiB.
enum HexDump {
    static let limit = 64 * 1024

    static func format(_ data: Data) -> String {
        let shown = data.prefix(limit)
        let bytes = [UInt8](shown)
        var lines: [String] = []
        var offset = 0
        while offset < bytes.count {
            let chunk = bytes[offset..<min(offset + 16, bytes.count)]
            var hex = ""
            for i in 0..<16 {
                if i < chunk.count {
                    hex += String(format: "%02x ", chunk[chunk.startIndex + i])
                } else {
                    hex += "   "
                }
                if i == 7 { hex += " " }
            }
            let ascii = String(
                chunk.map { $0 >= 0x20 && $0 < 0x7f ? Character(UnicodeScalar($0)) : "." })
            lines.append(String(format: "%08x  ", offset) + hex + " |" + ascii + "|")
            offset += 16
        }
        if data.count > limit {
            lines.append("(showing first \(limit) of \(data.count) bytes)")
        }
        return lines.joined(separator: "\n")
    }
}

/// Large text bodies render truncated until the user asks for all of it.
enum BodyTruncation {
    static let limit = 2 * 1024 * 1024

    static func apply(_ text: String, limit: Int = BodyTruncation.limit) -> (text: String, truncated: Bool) {
        let utf8 = text.utf8
        guard utf8.count > limit else { return (text, false) }
        var cut = limit
        // Back off so the cut never lands inside a multi-byte character.
        while cut > 0, String(utf8.prefix(cut)) == nil { cut -= 1 }
        return (String(utf8.prefix(cut)) ?? "", true)
    }
}

enum ResponseBodyBytes {
    /// The bytes a "Save body" writes: base64 bodies are decoded, text is UTF-8.
    static func decode(body: String, isBase64: Bool) -> Data? {
        isBase64 ? Data(base64Encoded: body, options: .ignoreUnknownCharacters) : Data(body.utf8)
    }
}

enum StatusTone: Equatable {
    case success, redirect, error

    static func of(_ status: UInt16) -> StatusTone {
        switch status {
        case 200..<300: return .success
        case 300..<400: return .redirect
        default: return .error
        }
    }
}

enum SendCancellation {
    static func isCancellation(_ error: Error) -> Bool {
        error is CancellationError
    }
}
