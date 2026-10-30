import CryptoKit
import Foundation

/// Which sidebar item was open.
struct SessionSelection: Codable, Equatable {
    enum Kind: String, Codable { case endpoint, scratch }
    var kind: Kind
    var id: String

    init(kind: Kind, id: String) {
        self.kind = kind
        self.id = id
    }

    init(_ selection: SidebarSelection) {
        switch selection {
        case .endpoint(let id): self = SessionSelection(kind: .endpoint, id: id)
        case .scratch(let id): self = SessionSelection(kind: .scratch, id: id)
        }
    }

    var sidebar: SidebarSelection {
        switch kind {
        case .endpoint: return .endpoint(id)
        case .scratch: return .scratch(id)
        }
    }
}

/// Codable copy of an editable request draft.
///
/// SECURITY NOTE: drafts are stored exactly as typed, in Application Support (never in the
/// workspace or git). Secrets should be typed as `{{secret.KEY}}` placeholders; a plain secret
/// value typed into an unsaved draft IS persisted to local disk until the draft is saved or
/// discarded. That is acceptable for a local, per-user file.
struct DraftSnapshot: Codable, Equatable {
    struct Row: Codable, Equatable {
        var name: String
        var value: String
        var enabled: Bool
        var secretKey: String?
        var kind: String

        init(_ nv: NameValue) {
            name = nv.name
            value = nv.value
            enabled = nv.enabled
            secretKey = nv.secretKey
            kind = nv.kind
        }

        var nameValue: NameValue {
            NameValue(name: name, value: value, enabled: enabled, secretKey: secretKey, kind: kind)
        }
    }

    var name: String
    var method: String
    var url: String
    var authenticated: Bool
    var params: [Row]
    var headers: [Row]
    var pathParams: [Row]
    var bodyKind: String
    var bodyText: String
    var bodyContentType: String
    var formFields: [Row]
    var multipartFields: [Row]
    var binaryPath: String
    var timeoutMs: UInt64?
    var followRedirects: Bool?
    var verifyTls: Bool?
    var proxy: String?

    init(_ d: RequestDraft) {
        name = d.name
        method = d.method
        url = d.url
        authenticated = d.authenticated
        params = d.params.map { Row($0.nameValue) }
        headers = d.headers.map { Row($0.nameValue) }
        pathParams = d.pathParams.map(Row.init)
        bodyKind = d.bodyKind.rawValue
        bodyText = d.bodyText
        bodyContentType = d.bodyContentType
        formFields = d.formFields.map { Row($0.nameValue) }
        multipartFields = d.multipartFields.map { Row($0.nameValue) }
        binaryPath = d.binaryPath
        timeoutMs = d.options.timeoutMs
        followRedirects = d.options.followRedirects
        verifyTls = d.options.verifyTls
        proxy = d.options.proxy
    }

    /// The edits laid over `base`. The preflight config is not editable in the draft, so it is
    /// kept from `base` (the item as it is on disk now).
    func apply(to base: RequestDraft) -> RequestDraft {
        var d = base
        d.name = name
        d.method = method
        d.url = url
        d.pathParams = pathParams.map(\.nameValue)  // after `url`, whose observer re-syncs rows
        d.authenticated = authenticated
        d.params = params.map { KeyValueRow($0.nameValue) }
        d.headers = headers.map { KeyValueRow($0.nameValue) }
        d.bodyKind = BodyKind(rawValue: bodyKind) ?? .none
        d.bodyText = bodyText
        d.bodyContentType = bodyContentType
        d.formFields = formFields.map { KeyValueRow($0.nameValue) }
        d.multipartFields = multipartFields.map { KeyValueRow($0.nameValue) }
        d.binaryPath = binaryPath
        d.options = RequestOptions(
            timeoutMs: timeoutMs, followRedirects: followRedirects, verifyTls: verifyTls, proxy: proxy)
        return d
    }
}

/// What one window had open: the workspace, its tabs and any unsaved drafts.
struct SessionState: Codable, Equatable {
    var workspacePath: String
    /// The active tab, kept for sessions written before tabs existed.
    var selection: SessionSelection?
    var draft: DraftSnapshot?
    var tabs: [TabSnapshot]?
    var activeTab: Int?
}

/// One JSON file per workspace (named by a hash of its path) in `directory`
/// (macOS: `~/Library/Application Support/XRest/sessions/`).
struct SessionStore {
    let directory: URL

    static var standard: SessionStore {
        let base =
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.temporaryDirectory
        return SessionStore(directory: base.appendingPathComponent("XRest/sessions", isDirectory: true))
    }

    static func normalize(_ path: String) -> String {
        URL(fileURLWithPath: path).standardizedFileURL.path
    }

    private func file(for path: String) -> URL {
        let digest = SHA256.hash(data: Data(Self.normalize(path).utf8))
        let name = digest.map { String(format: "%02x", $0) }.joined()
        return directory.appendingPathComponent("\(name).json")
    }

    func save(_ state: SessionState) throws {
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        try encoder.encode(state).write(to: file(for: state.workspacePath), options: .atomic)
    }

    func load(path: String) -> SessionState? {
        guard let data = try? Data(contentsOf: file(for: path)) else { return nil }
        return try? JSONDecoder().decode(SessionState.self, from: data)
    }

    func remove(path: String) {
        try? FileManager.default.removeItem(at: file(for: path))
    }

    /// Every readable session; unreadable files are skipped.
    func all() -> [SessionState] {
        let files =
            (try? FileManager.default.contentsOfDirectory(at: directory, includingPropertiesForKeys: nil)) ?? []
        return files.filter { $0.pathExtension == "json" }.compactMap {
            (try? Data(contentsOf: $0)).flatMap { try? JSONDecoder().decode(SessionState.self, from: $0) }
        }
        .sorted { $0.workspacePath < $1.workspacePath }
    }

    /// Workspaces to reopen at launch. Sessions whose folder is gone are deleted.
    func workspacesToReopen(exists: (URL) -> Bool = RecentWorkspaces.directoryExists) -> [URL] {
        var urls: [URL] = []
        for state in all() {
            let url = URL(fileURLWithPath: state.workspacePath)
            if exists(url) {
                urls.append(url)
            } else {
                remove(path: state.workspacePath)
            }
        }
        return urls
    }
}
