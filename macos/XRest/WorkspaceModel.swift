import AppKit
import Foundation
import Observation

/// What is open in the detail pane.
enum SidebarSelection: Hashable {
    case endpoint(String)
    case scratch(String)
}

/// Editors that replace the request detail pane.
enum DetailPanel: Equatable {
    case environments
    case settings
    case git
    case history
}

/// State of the last "Test preflight" run.
enum PreflightTestState {
    case idle
    case running
    case done(PreflightTestResult)
    case failed(String)
}

/// Observable wrapper around the Rust core's `XRestWorkspace` (one per window).
///
/// The core owns all behavior (interpolation, auth, unsafe policy, execution); this class only
/// mirrors its state for SwiftUI and holds UI state (selection, the draft being edited).
@MainActor
@Observable
final class WorkspaceModel {
    let core: XRestWorkspace
    let rootURL: URL
    let secrets: KeychainSecretStore

    private(set) var service: ServiceInfo
    private(set) var endpoints: [Endpoint] = []
    private(set) var scratch: [ScratchRequest] = []
    private(set) var environments: [EnvironmentConfig] = []
    private(set) var selectedEnvironment: String?
    private(set) var warnings: [String] = []

    private(set) var selection: SidebarSelection? {
        didSet { scheduleSessionSave() }
    }
    /// The request being edited. Written by the editor views.
    var draft: RequestDraft? {
        didSet { scheduleSessionSave() }
    }
    private var savedDraft: RequestDraft?

    private(set) var response: HttpResponse?
    private(set) var sendError: String?
    /// Keys named by a `MissingSecrets` send error (drives the "Open Secrets" button).
    private(set) var missingSecretKeys: [String] = []
    private(set) var isSending = false
    /// True after the user cancelled the last send (shown as "Cancelled", not as a failure).
    private(set) var wasCancelled = false
    private var sendTask: Task<Void, Never>?
    private var sendGeneration = 0

    // UI state driven from the model so menu commands can reach it.
    /// Set while an action waits behind the unsafe-environment confirmation sheet.
    var pendingConfirmation: PendingConfirmation?
    var showImportCurl = false
    var showSecrets = false
    var showExportCode = false
    var showReimport = false
    /// Environments / service settings editor shown instead of the request.
    private(set) var panel: DetailPanel?
    /// Set by the open panel while it holds unsaved edits.
    var panelDirty = false
    private(set) var preflightTest = PreflightTestState.idle
    /// Key to prefill in the Secrets screen's add form.
    var secretsPrefill: String?
    var pendingDelete: SidebarSelection?
    // Versions panel (endpoints only).
    private(set) var versionRows: [VersionRow] = []
    private(set) var selectedVersion: Int32?
    private(set) var diffRows: [DiffRow] = []
    /// Version awaiting the "Restore this version" confirmation.
    private(set) var pendingRestore: Int32?
    var errorMessage: String?
    // Git panel.
    private(set) var gitStatus: GitStatus?
    private(set) var gitBusy: GitAction?
    private(set) var gitError: String?
    /// Non-blocking notice (e.g. a pull changed the open request while it had unsaved edits).
    var gitNotice: String?
    /// Non-blocking banner for new core warnings such as "auto-commit failed: ...".
    var warningBanner: String?
    private(set) var autoCommit = false
    @ObservationIgnored private(set) var gitRefreshTask: Task<Void, Never>?
    // History panel.
    @ObservationIgnored private var historyStore: HistoryStore?
    private(set) var historyRows: [HistorySummary] = []
    private(set) var historyPaging = HistoryPaging()
    private(set) var historyDetail: HistoryEntry?
    private(set) var historyError: String?
    var historyNotice: String?
    var pendingClearHistory = false
    var historyFilter = HistoryFilterState()
    var historyPageSize: Int {
        get { historyPaging.pageSize }
        set { historyPaging = HistoryPaging(pageSize: newValue) }
    }
    /// Filters the sidebar lists.
    var searchText = ""

    static let maxVersionsKey = "maxVersions"
    static let defaultMaxVersions = 50

    init(
        url: URL, secrets: KeychainSecretStore = KeychainSecretStore(),
        defaults: UserDefaults = .standard
    ) throws {
        let opened = try XRestWorkspace.openWorkspace(path: url.path)
        rootURL = url
        core = opened
        self.secrets = secrets
        // Installed on every open so `{{secret.KEY}}` placeholders resolve from the Keychain.
        opened.setSecretProvider(provider: secrets)
        service = opened.service()
        autoCommit = opened.gitMode() == .auto
        applyPreferences(defaults: defaults)
        reload()
    }

    /// Push the app-wide defaults (HTTP options, retained versions) into the core. Called on open
    /// and whenever the Settings change.
    func applyPreferences(defaults: UserDefaults = .standard) {
        let prefs = AppPreferences(defaults: defaults)
        core.setHttpOptions(options: prefs.httpOptions)
        core.setMaxVersions(n: UInt32(prefs.maxVersions))
    }

    // MARK: Session restore

    /// True once the app is quitting; windows torn down by quit must keep their session.
    static var isTerminating = false

    @ObservationIgnored private var sessionStore: SessionStore?
    @ObservationIgnored var sessionSaveDelay: Duration = .milliseconds(500)
    @ObservationIgnored private var sessionSaveTask: Task<Void, Never>?
    @ObservationIgnored private var terminationObserver: NSObjectProtocol?

    private var sessionPath: String { SessionStore.normalize(rootURL.path) }

    /// Start recording this window's session (selection and unsaved draft) in `store`.
    /// Nothing is written until `restoreSession()` or `persistSessionNow()`.
    func attachSession(_ store: SessionStore) {
        sessionStore = store
        terminationObserver = NotificationCenter.default.addObserver(
            forName: NSApplication.willTerminateNotification, object: nil, queue: .main
        ) { [weak self] _ in
            MainActor.assumeIsolated {
                WorkspaceModel.isTerminating = true
                self?.persistSessionNow()
            }
        }
    }

    func persistSessionNow() {
        guard let sessionStore else { return }
        let state = SessionState(
            workspacePath: sessionPath, selection: selection.map(SessionSelection.init),
            draft: isDirty ? draft.map(DraftSnapshot.init) : nil)
        try? sessionStore.save(state)
    }

    /// Debounced write; call after every selection or draft change.
    func scheduleSessionSave() {
        guard sessionStore != nil else { return }
        sessionSaveTask?.cancel()
        let delay = sessionSaveDelay
        sessionSaveTask = Task { [weak self] in
            try? await Task.sleep(for: delay)
            guard !Task.isCancelled else { return }
            self?.persistSessionNow()
        }
    }

    /// Re-select what was open and bring back the unsaved draft (as an unsaved edit). If the item
    /// was deleted on disk meanwhile, a draft is kept as a new scratch request, with a notice.
    func restoreSession() {
        defer { persistSessionNow() }
        guard let state = sessionStore?.load(path: sessionPath), let ref = state.selection else { return }
        let target = ref.sidebar
        let exists: Bool
        switch target {
        case .endpoint(let id): exists = endpoints.contains { $0.id == id }
        case .scratch(let id): exists = scratch.contains { $0.id == id }
        }
        if exists {
            selection = target
            loadDraft(for: target)
            if let snapshot = state.draft, let base = draft { draft = snapshot.apply(to: base) }
            return
        }
        guard let snapshot = state.draft else { return }
        let recovered = snapshot.apply(to: .blank)
        do {
            let created = try core.createScratch(name: recovered.name, spec: recovered.spec)
            reload()
            selection = .scratch(created.id)
            loadDraft(for: selection)
            warningBanner =
                "\"\(recovered.name)\" no longer exists on disk. Your unsaved edits were kept as a scratch request."
        } catch {
            errorMessage = describe(error)
        }
    }

    /// The window closed. A clean close with nothing unsaved forgets the session; unsaved edits
    /// are kept. Windows torn down by quitting are never forgotten.
    func windowClosed() {
        sessionSaveTask?.cancel()
        if let terminationObserver { NotificationCenter.default.removeObserver(terminationObserver) }
        terminationObserver = nil
        guard let sessionStore, !Self.isTerminating else { return }
        if isDirty { persistSessionNow() } else { sessionStore.remove(path: sessionPath) }
    }

    /// Create a new workspace in `url` (the folder must not contain a service.yaml yet).
    static func create(at url: URL) throws -> WorkspaceModel {
        _ = try XRestWorkspace.createWorkspace(path: url.path, name: url.lastPathComponent)
        return try WorkspaceModel(url: url)
    }

    // MARK: Derived state

    var isDirty: Bool { draft != nil && draft != savedDraft }

    var selectedEnvironmentIsUnsafe: Bool {
        environments.first { $0.name == selectedEnvironment }?.isUnsafe ?? false
    }

    var canPromote: Bool {
        if case .scratch? = selection { return true }
        return false
    }

    var isScratchSelected: Bool { canPromote }

    // MARK: Loading

    func reload() {
        service = core.service()
        endpoints = core.listEndpoints()
        scratch = core.listScratch()
        environments = core.environments()
        selectedEnvironment = core.selectedEnvironment()
        warnings = core.warnings()
    }

    private func loadDraft(for selection: SidebarSelection?) {
        versionRows = []
        selectedVersion = nil
        diffRows = []
        pendingRestore = nil
        response = nil
        wasCancelled = false
        sendError = nil
        guard let selection else {
            draft = nil
            savedDraft = nil
            return
        }
        do {
            switch selection {
            case .endpoint(let id):
                let ep = try core.getEndpoint(id: id)
                draft = RequestDraft(name: ep.name, spec: ep.spec)
            case .scratch(let id):
                let s = try core.getScratch(id: id)
                draft = RequestDraft(name: s.name, spec: s.spec)
            }
            savedDraft = draft
        } catch {
            errorMessage = describe(error)
            draft = nil
            savedDraft = nil
        }
    }

    // MARK: SidebarSelection and saving

    /// Change the selection. Unsaved edits are saved first; if that fails the selection stays.
    func select(_ new: SidebarSelection?) {
        if panel != nil {
            guard !panelDirty else {
                errorMessage = "Save or revert your changes in the editor first."
                return
            }
            closePanel()
        }
        guard new != selection else { return }
        guard flush() else { return }
        selection = new
        loadDraft(for: new)
    }

    /// Save the current draft if it changed. Returns false (and shows an error) on failure.
    @discardableResult
    func flush() -> Bool {
        guard let selection, let draft, draft != savedDraft else { return true }
        do {
            switch selection {
            case .endpoint(let id):
                _ = try core.updateEndpoint(id: id, name: draft.name, spec: draft.spec)
            case .scratch(let id):
                _ = try core.updateScratch(id: id, name: draft.name, spec: draft.spec)
            }
            savedDraft = draft
            endpoints = core.listEndpoints()
            scratch = core.listScratch()
            afterSave()
            scheduleSessionSave()
            return true
        } catch {
            errorMessage = describe(error)
            return false
        }
    }

    func saveCurrent() {
        flush()
    }

    /// Registered by the open panel: saves its edits (no-op when it has none or is invalid).
    @ObservationIgnored private var panelSave: (() -> Void)?

    func registerPanelSave(_ save: (() -> Void)?) {
        panelSave = save
    }

    /// The single Save entry point (menu, ⌘S): the open panel's save, else the request draft.
    func saveActive() {
        if panel != nil {
            panelSave?()
        } else {
            flush()
        }
    }

    // MARK: Git

    enum GitAction: Equatable { case initializing, committing, pulling, pushing, syncing }

    var gitBadgeVisible: Bool { GitBadge.isVisible(gitStatus) }
    var isGitRepo: Bool { gitStatus?.isRepo ?? false }

    /// After any save: surface new core warnings and refresh the git badge.
    private func afterSave() {
        let latest = core.warnings()
        let fresh = WarningTracker.newEntries(old: warnings, new: latest)
        warnings = latest
        if let last = fresh.last { warningBanner = last }
        scheduleGitRefresh()
    }

    /// Fire-and-forget refresh; only runs git when the workspace is (or may be) a repo.
    func scheduleGitRefresh() {
        if let gitStatus, !gitStatus.isRepo { return }
        gitRefreshTask = Task { await refreshGitStatus() }
    }

    func refreshGitStatus() async {
        do {
            gitStatus = try await core.gitStatus()
        } catch {
            gitError = describe(error)
        }
    }

    func setAutoCommit(_ on: Bool) {
        do {
            try core.setGitMode(mode: on ? .auto : .manual)
            autoCommit = on
        } catch {
            gitError = describe(error)
        }
    }

    func gitInit(remote: String?) async {
        let trimmed = remote?.trimmingCharacters(in: .whitespaces)
        await runGit(.initializing) {
            try await $0.gitInit(remote: (trimmed?.isEmpty ?? true) ? nil : trimmed)
        }
    }

    /// An empty `message` uses the generated default.
    func gitCommit(message: String) async {
        let text = message.trimmingCharacters(in: .whitespacesAndNewlines)
        let final = text.isEmpty ? GitFormat.defaultCommitMessage(gitStatus?.uncommitted ?? []) : text
        await runGit(.committing) { _ = try await $0.gitCommit(message: final) }
    }

    func gitPush() async { await runGit(.pushing) { try await $0.gitPush() } }

    func gitPull() async { await runGit(.pulling, reloads: true) { try await $0.gitPull() } }

    func gitSync() async { await runGit(.syncing, reloads: true) { try await $0.gitSync() } }

    private func runGit(
        _ action: GitAction, reloads: Bool = false,
        _ body: (XRestWorkspace) async throws -> Void
    ) async {
        guard gitBusy == nil else { return }
        gitBusy = action
        gitError = nil
        gitNotice = nil
        let before: Endpoint?
        if case .endpoint(let id)? = selection { before = endpoints.first { $0.id == id } } else { before = nil }
        do {
            try await body(core)
            if reloads { applyReloadedState(before: before) }
        } catch {
            gitError = describe(error)
            // A failed pull/sync may still have changed nothing; refresh anyway below.
        }
        gitBusy = nil
        await refreshGitStatus()
    }

    /// The core reloaded from disk (pull/sync); refresh the lists and reconcile the open draft.
    private func applyReloadedState(before: Endpoint?) {
        let wasDirty = isDirty
        reload()
        guard case .endpoint(let id)? = selection else { return }
        let after = endpoints.first { $0.id == id }
        switch GitReload.decide(draftDirty: wasDirty, before: before, after: after) {
        case .keep: break
        case .reloadDraft: loadDraft(for: selection)
        case .closeDeleted:
            selection = nil
            loadDraft(for: nil)
        case .keepEditsWithNotice:
            gitNotice = "This request changed on disk during the pull. Your unsaved edits were kept."
        }
    }

    // MARK: History

    /// Attach the shared history database (called on every workspace open). A failed open is
    /// reported by the History panel; sending keeps working without history.
    func attachHistory(_ store: HistoryStore) {
        historyStore = store
        if let history = store.history { core.setHistory(history: history) }
        historyError = store.error
    }

    /// Reload the first page with the current filter.
    func reloadHistory() {
        historyRows = []
        historyPaging.reset(total: 0)
        guard let history = historyStore?.history else {
            historyError = historyStore?.error ?? "History is not available."
            historyDetail = nil
            return
        }
        historyError = nil
        let filter = historyFilter.filter(serviceId: service.id)
        do {
            historyPaging.reset(total: Int(try history.count(filter: filter)))
            try appendHistoryPage(history, filter)
            if let id = historyDetail?.id, !historyRows.contains(where: { $0.id == id }) {
                historyDetail = nil
            }
        } catch {
            historyError = describe(error)
        }
    }

    func loadMoreHistory() {
        guard historyPaging.hasMore, let history = historyStore?.history else { return }
        do {
            try appendHistoryPage(history, historyFilter.filter(serviceId: service.id))
        } catch {
            historyError = describe(error)
        }
    }

    private func appendHistoryPage(_ history: XRestHistory, _ filter: HistoryFilter) throws {
        let page = try history.list(
            filter: filter, limit: UInt32(historyPaging.pageSize), offset: UInt32(historyPaging.nextOffset))
        historyRows += page
        historyPaging.didLoad(count: page.count)
    }

    func selectHistory(_ id: Int64?) {
        historyNotice = nil
        guard let id else {
            historyDetail = nil
            return
        }
        do {
            historyDetail = try historyStore?.history?.get(id: id)
        } catch {
            historyDetail = nil
            historyError = describe(error)
        }
    }

    func requestClearHistory() { pendingClearHistory = true }

    func confirmClearHistory() {
        pendingClearHistory = false
        do {
            try historyStore?.history?.clear()
            historyDetail = nil
            reloadHistory()
        } catch {
            historyError = describe(error)
        }
    }

    /// Create a scratch request from the selected entry and open it in the editor.
    func openHistoryAsScratch() {
        guard let entry = historyDetail, flush() else { return }
        let converted = HistoryScratch.make(from: entry)
        do {
            let created = try core.createScratch(name: converted.name, spec: converted.spec)
            closePanel()
            reload()
            selection = .scratch(created.id)
            loadDraft(for: selection)
            if let notice = converted.notice { warningBanner = notice }
        } catch {
            errorMessage = describe(error)
        }
    }

    // MARK: Versions

    /// Reload the versions list for the open endpoint (empty for scratch requests).
    func loadVersions() {
        guard case .endpoint(let id)? = selection else {
            versionRows = []
            selectedVersion = nil
            diffRows = []
            return
        }
        do {
            let versions = try core.endpointVersions(id: id, limit: UInt32(Self.defaultMaxVersions))
            versionRows = VersionHistory.rows(versions: versions) { [core] from, to in
                try core.diffVersions(id: id, from: from, to: to)
            }
            if let selectedVersion, versionRows.contains(where: { $0.version == selectedVersion }) {
                selectVersion(selectedVersion)
            } else {
                selectedVersion = nil
                diffRows = []
            }
        } catch {
            errorMessage = describe(error)
        }
    }

    /// Show the diff of `version` against the next older one.
    func selectVersion(_ version: Int32) {
        guard case .endpoint(let id)? = selection,
            let i = versionRows.firstIndex(where: { $0.version == version })
        else { return }
        selectedVersion = version
        guard i + 1 < versionRows.count else {
            diffRows = []
            return
        }
        do {
            diffRows = DiffRow.rows(
                from: try core.diffVersions(id: id, from: versionRows[i + 1].version, to: version))
        } catch {
            diffRows = []
            errorMessage = describe(error)
        }
    }

    func requestRestore(_ version: Int32) {
        guard case .endpoint? = selection else { return }
        pendingRestore = version
    }

    func cancelRestore() {
        pendingRestore = nil
    }

    /// Restore the pending version as a new version and reload the draft. Unsaved edits are
    /// saved first (as their own version) so nothing is lost.
    func confirmRestore() {
        guard case .endpoint(let id)? = selection, let version = pendingRestore else { return }
        pendingRestore = nil
        guard flush() else { return }
        do {
            let restored = try core.restoreEndpointVersion(id: id, version: version)
            endpoints = core.listEndpoints()
            afterSave()
            loadDraft(for: selection)
            loadVersions()
            selectVersion(restored.lastVersion)
        } catch {
            errorMessage = describe(error)
        }
    }

    // MARK: Scratch requests and endpoints

    func newScratch() {
        guard flush() else { return }
        let blank = RequestDraft.blank
        do {
            let created = try core.createScratch(name: blank.name, spec: blank.spec)
            reload()
            selection = .scratch(created.id)
            loadDraft(for: selection)
        } catch {
            errorMessage = describe(error)
        }
    }

    /// Merges an OpenAPI spec into this workspace; refreshes the lists and git status.
    func reimportOpenapi(_ content: String) throws -> ImportSummary {
        let summary = try core.mergeOpenapi(content: content)
        reload()
        afterSave()
        return summary
    }

    /// Code for the current in-memory draft (unsaved edits included).
    func exportCode(language: CodeLanguage, includeSecrets: Bool) throws -> String {
        guard let draft else { throw OpenAPIInputError(message: "Select a request first.") }
        return try core.exportCode(spec: draft.spec, language: language, includeSecrets: includeSecrets)
    }

    func importCurl(_ command: String) {
        guard flush() else { return }
        do {
            let created = try core.importCurlAsScratch(command: command)
            reload()
            selection = .scratch(created.id)
            loadDraft(for: selection)
            showImportCurl = false
        } catch {
            errorMessage = describe(error)
        }
    }

    func promoteSelected() {
        guard case .scratch(let id)? = selection, flush() else { return }
        do {
            let ep = try core.promoteScratch(id: id, name: nil)
            reload()
            afterSave()
            selection = .endpoint(ep.id)
            loadDraft(for: selection)
        } catch {
            errorMessage = describe(error)
        }
    }

    func delete(_ target: SidebarSelection) {
        do {
            switch target {
            case .endpoint(let id): try core.deleteEndpoint(id: id)
            case .scratch(let id): try core.deleteScratch(id: id)
            }
            if selection == target {
                selection = nil
                loadDraft(for: nil)
            }
            reload()
        } catch {
            errorMessage = describe(error)
        }
    }

    // MARK: Environments

    func selectEnvironment(_ name: String) {
        guard name != selectedEnvironment else { return }
        do {
            try core.selectEnvironment(name: name)
            selectedEnvironment = core.selectedEnvironment()
        } catch {
            errorMessage = describe(error)
        }
    }

    // MARK: Environments and service settings

    /// Open an editor in the detail pane. Fails (with a message) if another one has unsaved edits.
    func openPanel(_ new: DetailPanel) {
        guard panel != new else { return }
        if panelDirty {
            errorMessage = "Save or revert your changes in the editor first."
            return
        }
        guard flush() else { return }
        panel = new
        preflightTest = .idle
        if new == .history { reloadHistory() }
    }

    func closePanel() {
        panel = nil
        panelSave = nil
        panelDirty = false
        preflightTest = .idle
    }

    /// Save all environments, then refresh state from the core.
    func saveEnvironments(_ envs: [EnvironmentConfig]) throws {
        try core.saveEnvironments(envs: envs)
        reload()
        afterSave()
    }

    func saveServiceSettings(_ settings: ServiceSettings) throws {
        try core.updateServiceSettings(settings: settings)
        reload()
        afterSave()
    }

    /// Run a preflight once (behind the unsafe confirmation when the environment is unsafe).
    func testPreflight(_ preflight: Preflight) {
        if case .running = preflightTest { return }
        runGuarded(method: preflight.method, url: preflight.url) { [weak self] confirmed in
            self?.runPreflightTest(preflight, confirmed: confirmed)
        }
    }

    private func runPreflightTest(_ preflight: Preflight, confirmed: Bool) {
        preflightTest = .running
        Task {
            do {
                let result = try await core.testPreflight(preflight: preflight, confirmed: confirmed)
                preflightTest = .done(result)
            } catch {
                preflightTest = .failed(describe(error))
            }
        }
    }

    // MARK: Interpolation

    /// Resolved text with secrets masked, for the URL preview and the unsafe confirmation.
    func preview(_ text: String) -> InterpolationPreview {
        core.preview(text: text)
    }

    /// The request URL as it will be sent (path params, query, variables; secrets masked).
    func previewRequestURL(_ spec: RequestSpec) -> InterpolationPreview {
        core.previewRequestUrl(spec: spec)
    }

    /// Names offered after typing `{{`: environment variables, then `secret.<key>` entries.
    func completions(prefix: String) -> [String] {
        CompletionTrigger.candidates(
            prefix: prefix, variables: core.variableNames(), secretKeys: secretKeyChoices())
    }

    // MARK: Secrets

    func openSecrets(prefill: String? = nil) {
        secretsPrefill = prefill
        showSecrets = true
    }

    /// Keys offered by "Link secret": stored keys plus keys the workspace already references.
    func secretKeyChoices() -> [String] {
        Array(Set(secrets.keys()).union(core.secretKeysInUse())).sorted()
    }

    /// Secret keys the current service references, with whether each is stored.
    func neededSecrets() -> [SecretsState.Needed] {
        SecretsState.needed(inUse: core.secretKeysInUse(), stored: secrets.keys())
    }

    // MARK: Sending

    /// Entry point for the Send button and menu command.
    func requestSend() {
        guard panel == nil, draft != nil, !isSending else { return }
        runGuarded { [weak self] confirmed in self?.start(confirmed: confirmed) }
    }

    /// Runs `action` straight away in a safe environment (`confirmed: false`), or behind the
    /// countdown confirmation sheet in an unsafe one (`confirmed: true` once accepted). The core
    /// is the authority on whether the environment is unsafe. Test Preflight uses this too.
    func runGuarded(
        method: String? = nil, url: String? = nil, _ action: @escaping (_ confirmed: Bool) -> Void
    ) {
        if core.sendPolicy().requiresConfirmation {
            presentConfirmation(method: method, url: url) { action(true) }
        } else {
            action(false)
        }
    }

    private func presentConfirmation(
        method: String? = nil, url: String? = nil, _ action: @escaping () -> Void
    ) {
        let rawURL = url ?? draft?.url ?? ""
        // Without an explicit url this is the draft request: show the URL as it will be sent
        // (path params and query merged). Test Preflight passes its own url.
        let resolved: String?
        if rawURL.isEmpty {
            resolved = nil
        } else if url == nil, let draft {
            resolved = core.previewRequestUrl(spec: draft.spec).resolved
        } else {
            resolved = core.preview(text: rawURL).resolved
        }
        pendingConfirmation = PendingConfirmation(
            environment: selectedEnvironment ?? "environment",
            method: method ?? draft?.method ?? "",
            url: rawURL,
            resolvedURL: resolved,
            action: action)
    }

    private func start(confirmed: Bool) {
        guard let draft, !isSending else { return }
        if case .scratch? = selection { flush() }
        sendGeneration += 1
        let generation = sendGeneration
        var endpointId: String?
        if case .endpoint(let id)? = selection { endpointId = id }
        sendTask = Task {
            await perform(
                spec: draft.spec, endpointId: endpointId, confirmed: confirmed, generation: generation)
        }
    }

    /// Cancels the in-flight send; the core future is dropped when the task is cancelled.
    func cancelSend() {
        guard isSending else { return }
        sendGeneration += 1  // any late result from the cancelled task is ignored
        sendTask?.cancel()
        sendTask = nil
        isSending = false
        response = nil
        sendError = nil
        missingSecretKeys = []
        wasCancelled = true
    }

    private func perform(spec: RequestSpec, endpointId: String?, confirmed: Bool, generation: Int) async {
        isSending = true
        wasCancelled = false
        sendError = nil
        missingSecretKeys = []
        defer {
            if generation == sendGeneration {
                isSending = false
                sendTask = nil
            }
            if panel == .history { reloadHistory() }
        }
        do {
            // Saved endpoints are attributed in the execution history; scratch requests are not.
            let result: HttpResponse
            if let endpointId {
                result = try await core.sendForEndpoint(
                    endpointId: endpointId, request: spec, confirmed: confirmed)
            } else {
                result = try await core.send(request: spec, confirmed: confirmed)
            }
            guard generation == sendGeneration, !Task.isCancelled else { return }
            response = result
        } catch {
            guard generation == sendGeneration else { return }
            if SendCancellation.isCancellation(error) || Task.isCancelled {
                response = nil
                wasCancelled = true
                return
            }
            handleSendFailure(error)
        }
    }

    private func handleSendFailure(_ error: Error) {
        if let error = error as? XRestError {
            response = nil
            if case .ConfirmationRequired = error {
                // The environment changed under us; ask again.
                presentConfirmation { [weak self] in self?.start(confirmed: true) }
            } else {
                missingSecretKeys = SecretsState.missingKeys(in: error)
                sendError = error.message
            }
        } else {
            response = nil
            sendError = error.localizedDescription
        }
    }
}
