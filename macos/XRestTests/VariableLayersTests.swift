import XCTest

@testable import XRest

final class VariableLayersTests: XCTestCase {
    private func value(_ layer: VariableLayer, _ v: String, enabled: Bool = true, secret: String? = nil) -> LayerValue {
        LayerValue(layer: layer, value: v, secretKey: secret, enabled: enabled)
    }

    func testEveryCorePaletteColorAndIconHasAnAppMapping() {
        XCTAssertFalse(PaletteColor.names.isEmpty)
        for name in PaletteColor.names { XCTAssertNotNil(PaletteColor.color(name), name) }
        XCTAssertNil(PaletteColor.color("neon"))
        XCTAssertNil(PaletteColor.color(nil))
        XCTAssertFalse(WorkspaceIcon.names.isEmpty)
        for name in WorkspaceIcon.names { XCTAssertNotNil(WorkspaceIcon.symbol(name), name) }
        XCTAssertEqual(PaletteColor.title("graphite"), "Graphite")
    }

    func testRowShowsWinnerAndTheValueItHides() {
        let row = VariableRow(
            VariableReport(
                name: "USER",
                values: [value(.local, "7"), value(.environment, "42")],
                winner: .local))
        XCTAssertEqual(row.display, "7")
        XCTAssertEqual(row.source, .local)
        XCTAssertEqual(row.shadowed, "42")
        XCTAssertEqual(row.localOverride?.value, "7")
    }

    func testDisabledLayersAreSkippedForShadowAndWinner() {
        let row = VariableRow(
            VariableReport(
                name: "A",
                values: [value(.local, "x", enabled: false), value(.workspace, "w"), value(.global, "g", enabled: false)],
                winner: .workspace))
        XCTAssertEqual(row.display, "w")
        XCTAssertNil(row.shadowed)
        let allOff = VariableRow(
            VariableReport(name: "B", values: [value(.global, "g", enabled: false)], winner: nil))
        XCTAssertEqual(allOff.display, "Not set")
        XCTAssertNil(allOff.source)
    }

    func testSecretsAreMaskedAndNeverShownAsShadowed() {
        let row = VariableRow(
            VariableReport(
                name: "TOKEN",
                values: [value(.environment, "", secret: "prod-token"), value(.global, "plain")],
                winner: .environment))
        XCTAssertTrue(row.isSecret)
        XCTAssertEqual(row.display, "•••••• (Keychain: prod-token)")
        XCTAssertNil(row.shadowed, "a hidden value under a secret is not interesting")
        XCTAssertEqual(VariableRow.show(value(.global, "")), "(empty)")
    }

    func testInspectorListsEveryLayerAndMarksTheOneInUse() {
        let row = VariableRow(
            VariableReport(
                name: "REGION",
                values: [value(.environment, "eu"), value(.workspace, "us")],
                winner: .environment))
        let entries = VariableInspector.entries(for: row)
        XCTAssertEqual(entries.map(\.layer), [.local, .environment, .workspace, .global])
        XCTAssertEqual(entries.map(\.inUse), [false, true, false, false])
        XCTAssertEqual(entries.map(\.overridden), [false, false, true, false])
        XCTAssertNil(entries[0].value)
    }

    func testLayerLabels() {
        XCTAssertEqual(VariableLayer.local.title, "Local override")
        XCTAssertTrue(VariableLayer.workspace.scope.contains("variables.yaml"))
    }
}

@MainActor
final class WorkspaceVariablesModelTests: XCTestCase {
    private var dir: URL!
    private var suiteNames: [String] = []

    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-vars-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    }

    override func tearDown() {
        for name in suiteNames { UserDefaults().removePersistentDomain(forName: name) }
        suiteNames = []
        try? FileManager.default.removeItem(at: dir)
    }

    private func makePreferences() -> AppPreferences {
        let name = "xrest-vars-prefs-\(UUID().uuidString)"
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

    /// PROD is unsafe, confirms changes only, and requires at least `minimum` seconds.
    private func prodChangesOnly(_ model: WorkspaceModel, minimum: UInt32) throws {
        try model.saveEnvironments([
            EnvironmentConfig(
                name: "prod", isUnsafe: true,
                variables: [.plain("BASE_URL", "https://h")],
                color: "red", confirm: .changes, minCountdownSeconds: minimum)
        ])
        model.selectEnvironment("prod")
        model.newScratch()
    }

    func testReadsSkipConfirmationInChangesOnlyEnvironment() throws {
        let model = try makeModel()
        try prodChangesOnly(model, minimum: 5)
        model.draft?.method = "GET"
        model.requestSend()
        XCTAssertNil(model.pendingConfirmation)
        model.cancelSend()
        XCTAssertEqual(model.selectedEnvironmentColor, "red")
    }

    func testWritesUseTheTeamMinimumEvenWithConfirmStyle() throws {
        let prefs = makePreferences()
        prefs.confirmStyle = .confirm
        let model = try makeModel(preferences: prefs)
        try prodChangesOnly(model, minimum: 7)
        model.draft?.method = "DELETE"
        model.requestSend()
        let pending = try XCTUnwrap(model.pendingConfirmation)
        XCTAssertEqual(pending.countdown.duration, 7)
        XCTAssertEqual(pending.teamMinimum, 7)
        XCTAssertEqual(pending.style, .confirm)
    }

    func testUserCountdownWinsWhenLongerThanTheTeamMinimum() throws {
        let prefs = makePreferences()
        prefs.confirmStyle = .countdown
        prefs.confirmCountdownSeconds = 9
        let model = try makeModel(preferences: prefs)
        try prodChangesOnly(model, minimum: 4)
        model.draft?.method = "POST"
        model.requestSend()
        XCTAssertEqual(try XCTUnwrap(model.pendingConfirmation).countdown.duration, 9)
    }

    func testSharedVariablesOverridesAndReport() throws {
        let model = try makeModel()
        try model.saveEnvironments([
            EnvironmentConfig(name: "dev", isUnsafe: false, variables: [.plain("USER", "42")])
        ])
        try model.saveWorkspaceVariables([.plain("REGION", "us-east-1"), .plain("  ", "dropped")])
        XCTAssertEqual(model.workspaceVariables.map(\.name), ["REGION"])

        try model.setLocalOverride(environment: "dev", name: "USER", value: "7")
        var rows = model.variableRows()
        let user = try XCTUnwrap(rows.first { $0.name == "USER" })
        XCTAssertEqual(user.source, .local)
        XCTAssertEqual(user.display, "7")
        XCTAssertEqual(user.shadowed, "42")
        XCTAssertEqual(rows.first { $0.name == "REGION" }?.source, .workspace)

        try model.setLocalOverride(environment: "dev", name: "USER", value: nil)
        rows = model.variableRows(environment: "dev")
        XCTAssertEqual(rows.first { $0.name == "USER" }?.source, .environment)
        XCTAssertTrue(model.localOverrides(environment: "dev").isEmpty)
    }

    func testGlobalsAttachAndBuiltinsHaveHoverText() throws {
        let model = try makeModel()
        let store = GlobalVariablesStore(path: dir.appendingPathComponent("app/variables.yaml"))
        try store.save([.plain("EMAIL", "me@example.com")])
        model.attachGlobals(store)
        XCTAssertEqual(model.preview("{{EMAIL}}").resolved, "me@example.com")
        XCTAssertEqual(model.variableRows().first { $0.name == "EMAIL" }?.source, .global)
        let info = model.variableInfo("$uuid")
        XCTAssertTrue(info.isResolved)
        XCTAssertTrue(info.text.hasPrefix("built-in:"))
        XCTAssertTrue(model.completions(prefix: "$").contains("$uuid"))
    }

    func testAppearanceIsValidatedAndSaved() throws {
        let model = try makeModel()
        try model.setAppearance(WorkspaceAppearance(color: "teal", icon: "server"))
        XCTAssertEqual(model.appearance.color, "teal")
        XCTAssertThrowsError(try model.setAppearance(WorkspaceAppearance(color: "neon", icon: nil)))
        XCTAssertEqual(model.appearance.icon, "server")
    }
}
