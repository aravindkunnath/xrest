import Foundation

/// One line of the versions list.
struct VersionRow: Identifiable, Equatable {
    let version: Int32
    /// Unix time (seconds) of the save.
    let timestamp: UInt64
    let summary: String
    var id: Int32 { version }
}

enum VersionSummary {
    static let maxFields = 3

    /// "url, header:Accept +2 more"; the field names are the core's.
    static func summarize(_ changes: [FieldChange]) -> String {
        guard !changes.isEmpty else { return "No changes" }
        let shown = changes.prefix(maxFields).map(\.field).joined(separator: ", ")
        let rest = changes.count - maxFields
        return rest > 0 ? "\(shown) +\(rest) more" : shown
    }
}

/// A field-level difference prepared for display.
struct DiffRow: Identifiable, Equatable {
    enum Kind: Equatable { case added, removed, changed }

    let id: Int
    let field: String
    let kind: Kind
    let before: String?
    let after: String?

    static func rows(from changes: [FieldChange]) -> [DiffRow] {
        changes.enumerated().map { i, c in
            let kind: Kind =
                switch c.change {
                case "added": .added
                case "removed": .removed
                default: .changed
                }
            return DiffRow(id: i, field: c.field, kind: kind, before: c.before, after: c.after)
        }
    }
}

enum VersionHistory {
    /// Rows newest first; each summarises the diff against the next older version.
    static func rows(
        versions: [EndpointVersion], diff: (_ from: Int32, _ to: Int32) throws -> [FieldChange]
    ) -> [VersionRow] {
        versions.enumerated().map { i, v in
            let summary: String
            if i + 1 < versions.count,
                let changes = try? diff(versions[i + 1].version, v.version)
            {
                summary = VersionSummary.summarize(changes)
            } else {
                summary = "Initial version"
            }
            return VersionRow(version: v.version, timestamp: v.lastUpdated, summary: summary)
        }
    }
}
