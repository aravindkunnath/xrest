import XCTest

@testable import XRest

final class EnvironmentGridTests: XCTestCase {
    private func nv(
        _ name: String, _ value: String = "", enabled: Bool = true, secret: String? = nil
    ) -> NameValue {
        NameValue(
            name: name, value: value, enabled: enabled, secretKey: secret,
            kind: secret == nil ? "plain" : "secret")
    }

    private var sample: [EnvironmentConfig] {
        [
            EnvironmentConfig(
                name: "dev", isUnsafe: false,
                variables: [nv("BASE_URL", "http://dev"), nv("TOKEN", secret: "dev-token"), nv("OFF", "x", enabled: false)]),
            EnvironmentConfig(
                name: "prod", isUnsafe: true,
                variables: [nv("BASE_URL", "https://prod"), nv("EXTRA", "1")]),
        ]
    }

    func testRoundTripPreservesOrderSecretKeyEnabledAndUnsafe() {
        let grid = EnvironmentGrid(environments: sample)
        XCTAssertEqual(grid.toConfigs(), sample)
    }

    func testRulesAndColorSurviveRoundTripRenameAndEdits() {
        var envs = sample
        envs[1].color = "red"
        envs[1].confirm = .changes
        envs[1].minCountdownSeconds = 5
        var grid = EnvironmentGrid(environments: envs)
        XCTAssertEqual(grid.toConfigs(), envs)
        XCTAssertFalse(grid.isDirty(comparedTo: envs))

        grid.renameColumn(from: "prod", to: "production")
        let renamed = grid.toConfigs()[1]
        XCTAssertEqual(renamed.color, "red")
        XCTAssertEqual(renamed.confirm, .changes)
        XCTAssertEqual(renamed.minCountdownSeconds, 5)

        grid.setColor(env: "production", "purple")
        grid.setConfirm(env: "production", .every)
        grid.setMinCountdown(env: "production", 99)
        let edited = grid.toConfigs()[1]
        XCTAssertEqual(edited.color, "purple")
        XCTAssertEqual(edited.confirm, .every)
        XCTAssertEqual(edited.minCountdownSeconds, 30, "clamped to the core's maximum")
        grid.setMinCountdown(env: "production", -4)
        XCTAssertEqual(grid.toConfigs()[1].minCountdownSeconds, 0)
        XCTAssertTrue(grid.isDirty(comparedTo: envs))
    }

    func testNewColumnHasDefaultRules() {
        var grid = EnvironmentGrid(environments: sample)
        grid.addColumn(name: "qa")
        let qa = grid.toConfigs().last!
        XCTAssertNil(qa.color)
        XCTAssertEqual(qa.confirm, .every)
        XCTAssertEqual(qa.minCountdownSeconds, 0)
    }

    func testRowsAreUnionInFirstSeenOrder() {
        let grid = EnvironmentGrid(environments: sample)
        XCTAssertEqual(grid.rows.map(\.name), ["BASE_URL", "TOKEN", "OFF", "EXTRA"])
        XCTAssertNil(grid.rows[1].cells["prod"])
    }

    func testRemovingVariableRemovesFromEveryEnvironment() {
        var grid = EnvironmentGrid(environments: sample)
        grid.removeRow(id: grid.rows[0].id)
        let out = grid.toConfigs()
        XCTAssertFalse(out.contains { $0.variables.contains { $0.name == "BASE_URL" } })
        XCTAssertEqual(out[0].variables.map(\.name), ["TOKEN", "OFF"])
        XCTAssertEqual(out[1].variables.map(\.name), ["EXTRA"])
    }

    func testLinkAndUnlinkSecret() {
        var grid = EnvironmentGrid(environments: sample)
        let id = grid.rows[0].id
        grid.linkSecret(rowID: id, env: "dev", key: "k1")
        var cell = grid.rows[0].cells["dev"]!
        XCTAssertEqual(cell.secretKey, "k1")
        XCTAssertEqual(cell.kind, "secret")
        XCTAssertEqual(cell.value, "")
        grid.unlinkSecret(rowID: id, env: "dev")
        cell = grid.rows[0].cells["dev"]!
        XCTAssertNil(cell.secretKey)
        XCTAssertEqual(cell.kind, "plain")
    }

    func testSettingValueCreatesCellAndEmptyCellsAreOmitted() {
        var grid = EnvironmentGrid(environments: sample)
        grid.addRow()
        let id = grid.rows.last!.id
        grid.setName(rowID: id, "NEW")
        XCTAssertEqual(grid.toConfigs().flatMap(\.variables).filter { $0.name == "NEW" }.count, 0)
        grid.setValue(rowID: id, env: "prod", "v")
        let prod = grid.toConfigs()[1]
        XCTAssertEqual(prod.variables.last, nv("NEW", "v"))
    }

    func testValidationFlagsEmptyAndDuplicateNames() {
        var grid = EnvironmentGrid(environments: sample)
        XCTAssertTrue(grid.validationErrors.isEmpty)
        grid.addRow()
        XCTAssertFalse(grid.validationErrors.isEmpty)
        grid.setName(rowID: grid.rows.last!.id, "BASE_URL")
        XCTAssertTrue(grid.validationErrors.contains { $0.contains("BASE_URL") })
    }

    func testUnsafeToggleAndDirtiness() {
        var grid = EnvironmentGrid(environments: sample)
        XCTAssertFalse(grid.isDirty(comparedTo: sample))
        grid.setUnsafe(env: "dev", true)
        XCTAssertTrue(grid.isDirty(comparedTo: sample))
        XCTAssertTrue(grid.toConfigs()[0].isUnsafe)
    }

    func testPerEnvironmentOrderIsPreserved() {
        let envs = [
            EnvironmentConfig(name: "dev", isUnsafe: false, variables: [nv("A", "1"), nv("B", "2")]),
            EnvironmentConfig(name: "prod", isUnsafe: false, variables: [nv("B", "3"), nv("A", "4")]),
        ]
        let grid = EnvironmentGrid(environments: envs)
        XCTAssertEqual(grid.toConfigs(), envs)
        XCTAssertFalse(grid.isDirty(comparedTo: envs))
    }

    func testDuplicateNameInOneEnvironmentIsKeptAndRejected() {
        let envs = [
            EnvironmentConfig(name: "dev", isUnsafe: false, variables: [nv("A", "1"), nv("A", "2")])
        ]
        let grid = EnvironmentGrid(environments: envs)
        XCTAssertEqual(grid.toConfigs(), envs, "nothing may be silently dropped")
        XCTAssertFalse(grid.validationErrors.isEmpty)
    }

    func testNamesAreTrimmedOnSaveAndLoadIsNotDirty() {
        let envs = [EnvironmentConfig(name: "dev", isUnsafe: false, variables: [nv("X ", "1")])]
        var grid = EnvironmentGrid(environments: envs)
        XCTAssertFalse(grid.isDirty(comparedTo: envs))
        XCTAssertEqual(grid.toConfigs()[0].variables.map(\.name), ["X"])
        grid.setName(rowID: grid.rows[0].id, "  Y  ")
        XCTAssertEqual(grid.toConfigs()[0].variables.map(\.name), ["Y"])
    }

    func testAddColumnAppendsSafeEmptyEnvironment() {
        var grid = EnvironmentGrid(environments: sample)
        grid.addColumn(name: "  qa  ")
        let last = grid.toConfigs().last!
        XCTAssertEqual(last.name, "qa")
        XCTAssertFalse(last.isUnsafe)
        XCTAssertTrue(last.variables.isEmpty)
    }

    func testRenameColumnKeepsValuesRanksSecretsAndUnsafe() {
        var grid = EnvironmentGrid(environments: sample)
        grid.renameColumn(from: "prod", to: "live")
        var expected = sample
        expected[1].name = "live"
        XCTAssertEqual(grid.toConfigs(), expected)
        grid.renameColumn(from: "dev", to: "local")
        expected[0].name = "local"
        XCTAssertEqual(grid.toConfigs(), expected)
        XCTAssertTrue(grid.validationErrors.isEmpty)
    }

    func testRenameToDuplicateOrEmptyIsAValidationError() {
        var grid = EnvironmentGrid(environments: sample)
        grid.renameColumn(from: "prod", to: "dev")
        XCTAssertTrue(grid.validationErrors.contains { $0.contains("dev") && $0.contains("more than once") })
        var empty = EnvironmentGrid(environments: sample)
        empty.renameColumn(from: "prod", to: "  ")
        XCTAssertTrue(empty.validationErrors.contains("An environment has no name."))
    }

    func testRemoveColumnDropsItsCellsOnly() {
        var grid = EnvironmentGrid(environments: sample)
        grid.removeColumn(name: "prod")
        XCTAssertEqual(grid.toConfigs(), [sample[0]])
        XCTAssertNil(grid.rows.first { $0.name == "BASE_URL" }?.cells["prod"])
    }

    func testRemovingLastColumnIsAValidationError() {
        var grid = EnvironmentGrid(environments: sample)
        grid.removeColumn(name: "dev")
        grid.removeColumn(name: "prod")
        XCTAssertTrue(grid.validationErrors.contains("At least one environment is required."))
    }

    func testMoveColumnReordersToConfigs() {
        var grid = EnvironmentGrid(environments: sample)
        grid.moveColumn(from: IndexSet(integer: 1), to: 0)
        XCTAssertEqual(grid.toConfigs().map(\.name), ["prod", "dev"])
    }

    func testIsDirtyAfterAddAndRename() {
        var grid = EnvironmentGrid(environments: sample)
        grid.addColumn(name: "qa")
        XCTAssertTrue(grid.isDirty(comparedTo: sample))
        var renamed = EnvironmentGrid(environments: sample)
        renamed.renameColumn(from: "dev", to: "local")
        XCTAssertTrue(renamed.isDirty(comparedTo: sample))
    }
}
