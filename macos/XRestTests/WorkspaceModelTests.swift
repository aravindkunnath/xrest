import XCTest

@testable import XRest

@MainActor
final class WorkspaceModelTests: XCTestCase {
    private var dir: URL!

    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-model-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    }

    private var suiteNames: [String] = []

    override func tearDown() {
        for name in suiteNames { UserDefaults().removePersistentDomain(forName: name) }
        suiteNames = []
        try? FileManager.default.removeItem(at: dir)
    }

    private func makePreferences() -> AppPreferences {
        let name = "xrest-model-prefs-\(UUID().uuidString)"
        suiteNames.append(name)
        return AppPreferences(defaults: UserDefaults(suiteName: name)!)
    }

    private func makeModel(preferences: AppPreferences? = nil) throws -> WorkspaceModel {
        let ws = dir.appendingPathComponent("svc")
        try FileManager.default.createDirectory(at: ws, withIntermediateDirectories: true)
        _ = try XRestWorkspace.createWorkspace(path: ws.path, name: "svc")
        let store = KeychainSecretStore(
            service: "dev.xrest.tests.\(UUID().uuidString)", indexURL: dir.appendingPathComponent("idx.json"))
        return try WorkspaceModel(url: ws, secrets: store, preferences: preferences ?? makePreferences())
    }

    private func unsafeSendPending(_ model: WorkspaceModel) throws -> PendingConfirmation {
        // The blank draft URL is {{BASE_URL}}/, so define it to keep the unresolved prompt out of the way.
        try model.saveEnvironments([
            EnvironmentConfig(
                name: "prod", isUnsafe: true,
                variables: [NameValue(name: "BASE_URL", value: "https://h", enabled: true, secretKey: nil, kind: "plain")])
        ])
        model.selectEnvironment("prod")
        model.newScratch()
        model.requestSend()
        return try XCTUnwrap(model.pendingConfirmation)
    }

    func testConfirmationUsesPreferenceStyleAndLength() throws {
        let prefs = makePreferences()
        prefs.confirmStyle = .countdown
        prefs.confirmCountdownSeconds = 5
        prefs.returnCancels = false
        let pending = try unsafeSendPending(try makeModel(preferences: prefs))
        XCTAssertEqual(pending.countdown.duration, 5)
        XCTAssertEqual(pending.style, .countdown)
        XCTAssertFalse(pending.returnCancels)
    }

    func testConfirmStyleHasZeroDelay() throws {
        let prefs = makePreferences()
        prefs.confirmStyle = .confirm
        let pending = try unsafeSendPending(try makeModel(preferences: prefs))
        XCTAssertEqual(pending.countdown.duration, 0)
        XCTAssertTrue(pending.countdown.isReady)
        XCTAssertEqual(pending.style, .confirm)
    }

    func testSaveActiveRoutesToOpenPanel() throws {
        let model = try makeModel()
        model.newScratch()
        var panelSaves = 0
        model.openPanel(.settings)
        model.draft?.name = "edited"
        XCTAssertTrue(model.isDirty)
        model.registerPanelSave { panelSaves += 1 }
        model.saveActive()
        XCTAssertEqual(panelSaves, 1)
        XCTAssertTrue(model.isDirty, "draft must not be saved while a panel is open")
    }

    func testSaveActiveSavesDraftWithoutPanel() throws {
        let model = try makeModel()
        model.newScratch()
        model.draft?.name = "edited"
        model.saveActive()
        XCTAssertFalse(model.isDirty)
    }

    func testRequestSendIsNoOpWhilePanelOpen() throws {
        let model = try makeModel()
        // Unsafe env: an effective send would surface synchronously as a confirmation.
        try model.saveEnvironments([EnvironmentConfig(name: "prod", isUnsafe: true, variables: [])])
        model.selectEnvironment("prod")
        model.newScratch()
        model.openPanel(.environments)
        model.requestSend()
        XCTAssertNil(model.pendingConfirmation)
        XCTAssertFalse(model.isSending)
    }

    func testConfirmationShowsRealURL() throws {
        let model = try makeModel()
        try model.saveEnvironments([
            EnvironmentConfig(
                name: "prod", isUnsafe: true,
                variables: [NameValue(name: "BASE", value: "https://h", enabled: true, secretKey: nil, kind: "plain")])
        ])
        model.selectEnvironment("prod")
        model.newScratch()
        model.draft?.url = "{{BASE}}/users/{id}"
        model.draft?.pathParams = [
            NameValue(name: "id", value: "42", enabled: true, secretKey: nil, kind: "plain")
        ]
        model.draft?.params = [
            KeyValueRow(NameValue(name: "limit", value: "5", enabled: true, secretKey: nil, kind: "plain"))
        ]
        model.requestSend()
        let pending = try XCTUnwrap(model.pendingConfirmation)
        XCTAssertEqual(pending.resolvedURL, "https://h/users/42?limit=5")
        XCTAssertEqual(pending.url, "{{BASE}}/users/{id}")
    }

    private func runDraft(_ model: WorkspaceModel, url: String) -> () -> Bool {
        model.newScratch()
        model.draft?.url = url
        var ran = false
        model.runGuarded { _ in ran = true }
        return { ran }
    }

    func testUnresolvedVariablePromptsWhenEnabled() throws {
        let prefs = makePreferences()
        prefs.warnUnresolved = true
        let model = try makeModel(preferences: prefs)
        let ran = runDraft(model, url: "{{MISSING}}/x")
        let pending = try XCTUnwrap(model.pendingConfirmation)
        XCTAssertEqual(pending.reason, .unresolved(["MISSING"]))
        XCTAssertEqual(pending.style, .confirm)
        XCTAssertTrue(pending.countdown.isReady)
        XCTAssertFalse(ran())
        pending.accept()
        XCTAssertTrue(ran())
    }

    func testUnresolvedDoesNotPromptWhenDisabled() throws {
        let prefs = makePreferences()
        prefs.warnUnresolved = false
        let model = try makeModel(preferences: prefs)
        let ran = runDraft(model, url: "{{MISSING}}/x")
        XCTAssertNil(model.pendingConfirmation)
        XCTAssertTrue(ran())
    }

    func testNoPromptWhenAllResolved() throws {
        let prefs = makePreferences()
        prefs.warnUnresolved = true
        let model = try makeModel(preferences: prefs)
        try model.saveEnvironments([
            EnvironmentConfig(
                name: "dev", isUnsafe: false,
                variables: [NameValue(name: "BASE", value: "https://h", enabled: true, secretKey: nil, kind: "plain")])
        ])
        model.selectEnvironment("dev")
        let ran = runDraft(model, url: "{{BASE}}/x")
        XCTAssertNil(model.pendingConfirmation)
        XCTAssertTrue(ran())
    }

    func testAcceptingUnresolvedThenShowsUnsafeConfirmation() throws {
        let prefs = makePreferences()
        prefs.warnUnresolved = true
        let model = try makeModel(preferences: prefs)
        try model.saveEnvironments([EnvironmentConfig(name: "prod", isUnsafe: true, variables: [])])
        model.selectEnvironment("prod")
        let ran = runDraft(model, url: "{{MISSING}}/x")
        let first = try XCTUnwrap(model.pendingConfirmation)
        XCTAssertEqual(first.reason, .unresolved(["MISSING"]))
        first.accept()
        let second = try XCTUnwrap(model.pendingConfirmation)
        XCTAssertNotEqual(second.id, first.id)
        XCTAssertEqual(second.reason, .unsafeEnvironment)
        XCTAssertFalse(ran())
        second.accept()
        XCTAssertTrue(ran())
    }

    func testDismissAfterAcceptKeepsChainedUnsafeConfirmation() throws {
        let prefs = makePreferences()
        prefs.warnUnresolved = true
        let model = try makeModel(preferences: prefs)
        try model.saveEnvironments([EnvironmentConfig(name: "prod", isUnsafe: true, variables: [])])
        model.selectEnvironment("prod")
        _ = runDraft(model, url: "{{MISSING}}/x")
        let first = try XCTUnwrap(model.pendingConfirmation)
        first.accept()
        model.dismissConfirmation(first)
        let second = try XCTUnwrap(model.pendingConfirmation)
        XCTAssertEqual(second.reason, .unsafeEnvironment)
    }

    func testDismissConfirmationClearsOnlyTheShownPrompt() throws {
        let model = try makeModel(preferences: makePreferences())
        let a = PendingConfirmation(environment: "A", method: "GET", url: "u", action: {})
        let b = PendingConfirmation(environment: "B", method: "GET", url: "u", action: {})
        model.pendingConfirmation = b
        model.dismissConfirmation(a)
        XCTAssertTrue(model.pendingConfirmation === b)
        model.dismissConfirmation(b)
        XCTAssertNil(model.pendingConfirmation)
    }

    func testExplicitURLIsNotUnresolvedGuarded() throws {
        let prefs = makePreferences()
        prefs.warnUnresolved = true
        let model = try makeModel(preferences: prefs)
        model.newScratch()
        var ran = false
        model.runGuarded(url: "{{MISSING}}/x") { _ in ran = true }
        XCTAssertNil(model.pendingConfirmation)
        XCTAssertTrue(ran)
    }
}
