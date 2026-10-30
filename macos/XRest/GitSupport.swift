import Foundation

/// Rail badge: shown when a repo has uncommitted or unpushed changes.
enum GitBadge {
    static func isVisible(_ status: GitStatus?) -> Bool {
        guard let status, status.isRepo else { return false }
        return !status.uncommitted.isEmpty || status.ahead > 0
    }
}

/// Display text for the Git panel.
enum GitFormat {
    /// "Update endpoints/a.yaml", "Add 2 files", ...
    static func defaultCommitMessage(_ files: [GitFileStatus]) -> String {
        guard !files.isEmpty else { return "Update workspace" }
        let verbs = Set(files.map { verb($0.status) })
        let v = verbs.count == 1 ? verbs.first! : "Update"
        if files.count == 1 { return "\(v) \(files[0].path)" }
        return "\(v) \(files.count) files"
    }

    private static func verb(_ status: String) -> String {
        switch status {
        case "added", "untracked": return "Add"
        case "deleted": return "Remove"
        default: return "Update"
        }
    }

    static func statusLabel(_ status: String) -> String {
        switch status {
        case "modified": return "Modified"
        case "added": return "Added"
        case "deleted": return "Deleted"
        case "renamed": return "Renamed"
        case "untracked": return "New"
        case "conflicted": return "Conflict"
        default: return status
        }
    }

    static func syncSummary(ahead: UInt32, behind: UInt32) -> String {
        var parts: [String] = []
        if ahead > 0 { parts.append("\(ahead) ahead") }
        if behind > 0 { parts.append("\(behind) behind") }
        return parts.isEmpty ? "Up to date" : parts.joined(separator: ", ")
    }

    static func lastSync(_ timestamp: UInt64?, now: Date = Date()) -> String {
        guard let timestamp else { return "Never" }
        let seconds = now.timeIntervalSince1970 - Double(timestamp)
        if seconds < 60 { return "Just now" }
        if seconds < 3600 { return "\(Int(seconds / 60)) min ago" }
        if seconds < 86400 { return "\(Int(seconds / 3600)) h ago" }
        return "\(Int(seconds / 86400)) d ago"
    }
}

enum WarningTracker {
    /// Entries in `new` that were not in `old`.
    static func newEntries(old: [String], new: [String]) -> [String] {
        let seen = Set(old)
        return new.filter { !seen.contains($0) }
    }
}

/// What to do with the open endpoint draft after the core reloaded from disk.
enum GitReload {
    enum Action: Equatable {
        case keep
        case reloadDraft
        case closeDeleted
        case keepEditsWithNotice
    }

    static func decide(draftDirty: Bool, before: Endpoint?, after: Endpoint?) -> Action {
        guard let before else { return .keep }
        guard let after else { return draftDirty ? .keepEditsWithNotice : .closeDeleted }
        if before.name == after.name && before.spec == after.spec { return .keep }
        return draftDirty ? .keepEditsWithNotice : .reloadDraft
    }
}
