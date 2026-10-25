import Foundation
import Observation
import SwiftUI

/// `{name}` path tokens in a URL. The rules mirror `substitute_path_params` in
/// core/xrest-core/src/resolve.rs: only the path portion (after `://host`, before `?` or `#`)
/// counts, `{{…}}` placeholders are skipped, and a name is any non-empty text without braces.
///
/// One deliberate widening: the core runs on the URL after variable interpolation, so a URL that
/// starts with a variable (`{{BASE_URL}}/users/{id}`) is treated as if that variable were
/// `scheme://host`.
enum PathTokens {
    static func extract(from url: String) -> [String] {
        guard let path = pathPortion(of: url) else { return [] }
        var names: [String] = []
        var rest = Substring(path)
        while let open = rest.firstIndex(of: "{") {
            rest = rest[open...]
            if rest.hasPrefix("{{") {
                if let end = rest.range(of: "}}") {
                    rest = rest[end.upperBound...]
                } else {
                    rest = rest.dropFirst(2)
                }
                continue
            }
            guard let close = rest.firstIndex(of: "}") else { break }
            let name = rest[rest.index(after: rest.startIndex)..<close]
            if name.isEmpty || name.contains("{") {
                rest = rest.dropFirst()
                continue
            }
            let token = String(name)
            if !names.contains(token) { names.append(token) }
            rest = rest[rest.index(after: close)...]
        }
        return names
    }

    private static func pathPortion(of url: String) -> String? {
        let afterHost: Substring
        if let scheme = url.range(of: "://") {
            afterHost = url[scheme.upperBound...]
        } else if url.hasPrefix("{{"), let end = url.range(of: "}}") {
            afterHost = url[end.upperBound...]
        } else {
            return nil
        }
        let delimiters: Set<Character> = ["/", "?", "#"]
        guard let start = afterHost.firstIndex(where: delimiters.contains) else { return "" }
        let fromPath = afterHost[start...]
        let end = fromPath.firstIndex(where: { $0 == "?" || $0 == "#" }) ?? fromPath.endIndex
        return String(fromPath[..<end])
    }

    /// Rows for exactly `tokens`, in token order. Existing rows keep their value and enabled flag;
    /// rows whose token left the URL are dropped.
    static func synced(_ existing: [NameValue], tokens: [String]) -> [NameValue] {
        tokens.map { token in
            existing.first { $0.name == token }
                ?? NameValue(name: token, value: "", enabled: true, secretKey: nil, kind: "plain")
        }
    }
}

/// An unfinished `{{name` before the caret in a text field, and how to complete it.
struct CompletionTrigger: Equatable {
    /// From the `{{` through the caret (UTF-16 offsets, as NSTextView reports them).
    let range: NSRange
    /// What was typed after `{{`.
    let prefix: String

    static func detect(in text: String, caret: Int) -> CompletionTrigger? {
        let ns = text as NSString
        guard caret >= 2, caret <= ns.length else { return nil }
        let before = ns.substring(to: caret)
        guard let open = (before as NSString).range(of: "{{", options: .backwards) as NSRange?,
            open.location != NSNotFound
        else { return nil }
        let prefix = (before as NSString).substring(from: open.location + 2)
        let forbidden = CharacterSet(charactersIn: "{}").union(.whitespacesAndNewlines)
        guard prefix.rangeOfCharacter(from: forbidden) == nil else { return nil }
        return CompletionTrigger(
            range: NSRange(location: open.location, length: caret - open.location), prefix: prefix)
    }

    /// Variables then `secret.<key>` entries matching `prefix`: prefix matches first, then other
    /// substring matches, case-insensitive.
    static func candidates(prefix: String, variables: [String], secretKeys: [String]) -> [String] {
        let all = variables + secretKeys.map { "secret.\($0)" }
        guard !prefix.isEmpty else { return all }
        let p = prefix.lowercased()
        let starts = all.filter { $0.lowercased().hasPrefix(p) }
        let contains = all.filter { !$0.lowercased().hasPrefix(p) && $0.lowercased().contains(p) }
        return starts + contains
    }

    /// Replace the typed prefix with `{{name}}`, also consuming the rest of the name and the
    /// closing braces if they already follow the caret. Returns the new text and caret.
    func insert(_ name: String, into text: String) -> (text: String, caret: Int) {
        let ns = text as NSString
        var end = range.location + range.length
        let nameChars = CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "_.-"))
        while end < ns.length {
            let ch = ns.substring(with: NSRange(location: end, length: 1))
            if ch.rangeOfCharacter(from: nameChars) == nil { break }
            end += 1
        }
        if end + 2 <= ns.length, ns.substring(with: NSRange(location: end, length: 2)) == "}}" {
            end += 2
        }
        let replacement = "{{\(name)}}"
        let result = ns.replacingCharacters(
            in: NSRange(location: range.location, length: end - range.location), with: replacement)
        return (result, range.location + (replacement as NSString).length)
    }
}

/// Splits resolved preview text into plain runs and leftover `{{…}}` placeholders (unresolved).
enum PreviewSegments {
    struct Segment: Equatable {
        let text: String
        let isUnresolved: Bool
    }

    static func split(_ text: String) -> [Segment] {
        guard !text.isEmpty else { return [] }
        let regex = try! NSRegularExpression(pattern: #"\{\{[^}]*\}\}"#)
        let ns = text as NSString
        var out: [Segment] = []
        var cursor = 0
        for m in regex.matches(in: text, range: NSRange(location: 0, length: ns.length)) {
            if m.range.location > cursor {
                out.append(
                    Segment(
                        text: ns.substring(with: NSRange(location: cursor, length: m.range.location - cursor)),
                        isUnresolved: false))
            }
            out.append(Segment(text: ns.substring(with: m.range), isUnresolved: true))
            cursor = m.range.location + m.range.length
        }
        if cursor < ns.length {
            out.append(Segment(text: ns.substring(from: cursor), isUnresolved: false))
        }
        return out
    }
}

/// The last opened workspaces (most recent first), persisted as paths in UserDefaults.
struct RecentWorkspaces {
    static let defaultKey = "recentWorkspaces"

    let defaults: UserDefaults
    let key: String
    let limit: Int
    let exists: (URL) -> Bool

    init(
        defaults: UserDefaults = .standard, key: String = RecentWorkspaces.defaultKey,
        limit: Int = 10,
        exists: @escaping (URL) -> Bool = RecentWorkspaces.directoryExists
    ) {
        self.defaults = defaults
        self.key = key
        self.limit = limit
        self.exists = exists
    }

    static func directoryExists(_ url: URL) -> Bool {
        var isDir: ObjCBool = false
        return FileManager.default.fileExists(atPath: url.path, isDirectory: &isDir) && isDir.boolValue
    }

    private static func normalized(_ url: URL) -> String {
        url.standardizedFileURL.path
    }

    private var stored: [String] { defaults.stringArray(forKey: key) ?? [] }

    func record(_ url: URL) {
        let path = Self.normalized(url)
        var paths = stored.filter { $0 != path }
        paths.insert(path, at: 0)
        defaults.set(Array(paths.prefix(limit)), forKey: key)
    }

    /// Existing folders only; missing ones are removed from the stored list.
    func list() -> [URL] {
        let all = stored
        let alive = all.filter { exists(URL(fileURLWithPath: $0)) }
        if alive.count != all.count { defaults.set(alive, forKey: key) }
        return alive.prefix(limit).map { URL(fileURLWithPath: $0) }
    }

    func clear() {
        defaults.removeObject(forKey: key)
    }
}

/// Observable wrapper so File > Open Recent and the welcome screen refresh when the list changes.
@MainActor
@Observable
final class RecentStore {
    static let shared = RecentStore()

    private let recents = RecentWorkspaces()
    private(set) var items: [URL]

    private init() { items = recents.list() }

    func record(_ url: URL) {
        recents.record(url)
        items = recents.list()
    }

    func refresh() { items = recents.list() }

    func clear() {
        recents.clear()
        items = []
    }
}

/// A completed `{{name}}` token under a position in a text field, for the hover popup.
enum VariableToken {
    /// The trimmed name of the `{{…}}` token containing UTF-16 `offset`, if any.
    static func name(at offset: Int, in text: String) -> (name: String, range: NSRange)? {
        guard let regex = try? NSRegularExpression(pattern: #"\{\{\s*([^{}]+?)\s*\}\}"#) else {
            return nil
        }
        let ns = text as NSString
        for match in regex.matches(in: text, range: NSRange(location: 0, length: ns.length))
        where NSLocationInRange(offset, match.range) {
            return (ns.substring(with: match.range(at: 1)), match.range)
        }
        return nil
    }

    /// Character index under horizontal position `x` in a monospaced single-line field.
    static func offset(atX x: CGFloat, charWidth: CGFloat) -> Int {
        guard charWidth > 0, x >= 0 else { return -1 }
        return Int(x / charWidth)
    }
}

/// The hover popup's content for one variable.
struct VariableInfo: Equatable {
    var text: String
    var isResolved: Bool
}

private struct VariableLookupKey: EnvironmentKey {
    static let defaultValue: ((String) -> VariableInfo)? = nil
}

extension EnvironmentValues {
    /// Resolves a variable name for the hover popup; nil where no workspace is available.
    var variableLookup: ((String) -> VariableInfo)? {
        get { self[VariableLookupKey.self] }
        set { self[VariableLookupKey.self] = newValue }
    }
}
