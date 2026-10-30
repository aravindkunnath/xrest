import Foundation

/// One open request in the tab strip.
///
/// The model keeps the active tab's live state in its own properties (`draft`, `response`, …) and
/// stores every other tab here; `WorkspaceModel.tabItems` overlays the live state on the active one.
struct OpenTab: Identifiable, Equatable {
    let id: UUID
    var selection: SidebarSelection?
    var draft: RequestDraft?
    var savedDraft: RequestDraft?
    var response: HttpResponse?
    var sendError: String?
    var missingSecretKeys: [String] = []
    var wasCancelled = false
    /// A preview tab is replaced by the next sidebar click; editing or double-clicking keeps it.
    var isPreview = false

    init(id: UUID = UUID(), isPreview: Bool = false) {
        self.id = id
        self.isPreview = isPreview
    }

    var isDirty: Bool { draft != nil && draft != savedDraft }

    var title: String {
        guard let name = draft?.name, !name.isEmpty else { return "Untitled" }
        return name
    }

    var isScratch: Bool {
        if case .scratch? = selection { return true }
        return false
    }
}

/// What a session file remembers about one tab.
struct TabSnapshot: Codable, Equatable {
    var selection: SessionSelection
    /// Only present while the tab has unsaved edits.
    var draft: DraftSnapshot?
    var isPreview: Bool
}
