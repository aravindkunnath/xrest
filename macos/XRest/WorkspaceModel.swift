import Foundation
import Observation

/// What is open in the detail pane.
enum SidebarSelection: Hashable {
    case endpoint(String)
    case scratch(String)
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

    private(set) var selection: SidebarSelection?
    /// The request being edited. Written by the editor views.
    var draft: RequestDraft?
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
    /// Key to prefill in the Secrets screen's add form.
    var secretsPrefill: String?
    var pendingDelete: SidebarSelection?
    var errorMessage: String?
    /// Filters the sidebar lists.
    var searchText = ""

    init(url: URL, secrets: KeychainSecretStore = KeychainSecretStore()) throws {
        let opened = try XRestWorkspace.openWorkspace(path: url.path)
        rootURL = url
        core = opened
        self.secrets = secrets
        // Installed on every open so `{{secret.KEY}}` placeholders resolve from the Keychain.
        opened.setSecretProvider(provider: secrets)
        service = opened.service()
        reload()
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
            return true
        } catch {
            errorMessage = describe(error)
            return false
        }
    }

    func saveCurrent() {
        flush()
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

    // MARK: Secrets

    func openSecrets(prefill: String? = nil) {
        secretsPrefill = prefill
        showSecrets = true
    }

    /// Secret keys the current service references, with whether each is stored.
    func neededSecrets() -> [SecretsState.Needed] {
        SecretsState.needed(inUse: core.secretKeysInUse(), stored: secrets.keys())
    }

    // MARK: Sending

    /// Entry point for the Send button and menu command.
    func requestSend() {
        guard draft != nil, !isSending else { return }
        runGuarded { [weak self] confirmed in self?.start(confirmed: confirmed) }
    }

    /// Runs `action` straight away in a safe environment (`confirmed: false`), or behind the
    /// countdown confirmation sheet in an unsafe one (`confirmed: true` once accepted). The core
    /// is the authority on whether the environment is unsafe. Test Preflight uses this too.
    func runGuarded(_ action: @escaping (_ confirmed: Bool) -> Void) {
        if core.sendPolicy().requiresConfirmation {
            presentConfirmation { action(true) }
        } else {
            action(false)
        }
    }

    private func presentConfirmation(_ action: @escaping () -> Void) {
        pendingConfirmation = PendingConfirmation(
            environment: selectedEnvironment ?? "environment",
            method: draft?.method ?? "",
            url: draft?.url ?? "",
            action: action)
    }

    private func start(confirmed: Bool) {
        guard let draft, !isSending else { return }
        if case .scratch? = selection { flush() }
        sendGeneration += 1
        let generation = sendGeneration
        sendTask = Task { await perform(spec: draft.spec, confirmed: confirmed, generation: generation) }
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

    private func perform(spec: RequestSpec, confirmed: Bool, generation: Int) async {
        isSending = true
        wasCancelled = false
        sendError = nil
        missingSecretKeys = []
        defer {
            if generation == sendGeneration {
                isSending = false
                sendTask = nil
            }
        }
        do {
            let result = try await core.send(request: spec, confirmed: confirmed)
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
