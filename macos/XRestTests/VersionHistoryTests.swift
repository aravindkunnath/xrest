import XCTest

@testable import XRest

final class VersionHistoryTests: XCTestCase {
    private func change(_ field: String, _ kind: String, _ before: String? = nil, _ after: String? = nil)
        -> FieldChange
    {
        FieldChange(field: field, change: kind, before: before, after: after)
    }

    func testSummaryListsFieldsAndCollapsesTheRest() {
        XCTAssertEqual(VersionSummary.summarize([]), "No changes")
        XCTAssertEqual(VersionSummary.summarize([change("url", "changed")]), "url")
        XCTAssertEqual(
            VersionSummary.summarize([
                change("url", "changed"), change("header:Accept", "added"), change("method", "changed"),
            ]), "url, header:Accept, method")
        XCTAssertEqual(
            VersionSummary.summarize([
                change("url", "changed"), change("header:Accept", "added"), change("method", "changed"),
                change("body.content", "changed"), change("param:q", "removed"),
            ]), "url, header:Accept, method +2 more")
    }

    func testDiffRowKinds() {
        let rows = DiffRow.rows(from: [
            change("header:X", "added", nil, "1"),
            change("param:q", "removed", "a", nil),
            change("url", "changed", "a", "b"),
        ])
        XCTAssertEqual(rows.map(\.kind), [.added, .removed, .changed])
        XCTAssertEqual(rows[0].after, "1")
        XCTAssertNil(rows[0].before)
        XCTAssertEqual(rows[2].before, "a")
        XCTAssertEqual(rows[2].after, "b")
        XCTAssertEqual(rows[2].field, "url")
    }

    func testRowsPairEachVersionWithItsPredecessor() {
        func v(_ n: Int32, _ t: UInt64) -> EndpointVersion {
            EndpointVersion(version: n, lastUpdated: t, spec: RequestDraft.blank.spec)
        }
        var asked: [String] = []
        let rows = VersionHistory.rows(versions: [v(3, 30), v(2, 20), v(1, 10)]) { from, to in
            asked.append("\(from)->\(to)")
            return [self.change("url", "changed")]
        }
        XCTAssertEqual(rows.map(\.version), [3, 2, 1])
        XCTAssertEqual(asked, ["2->3", "1->2"])
        XCTAssertEqual(rows[0].summary, "url")
        XCTAssertEqual(rows[2].summary, "Initial version")
        XCTAssertEqual(rows[2].timestamp, 10)
    }
}

@MainActor
final class VersionHistoryModelTests: XCTestCase {
    private var dir: URL!
    private var suite = ""

    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-vh-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        suite = "xrest.tests.\(UUID().uuidString)"
    }

    override func tearDown() {
        try? FileManager.default.removeItem(at: dir)
        UserDefaults.standard.removePersistentDomain(forName: suite)
    }

    private func makeModel(maxVersions: Int? = nil) throws -> WorkspaceModel {
        let ws = dir.appendingPathComponent("svc")
        try FileManager.default.createDirectory(at: ws, withIntermediateDirectories: true)
        _ = try XRestWorkspace.createWorkspace(path: ws.path, name: "svc")
        let store = KeychainSecretStore(
            service: "dev.xrest.tests.\(UUID().uuidString)", indexURL: dir.appendingPathComponent("idx.json"))
        let defaults = UserDefaults(suiteName: suite)!
        if let maxVersions { defaults.set(maxVersions, forKey: "maxVersions") }
        return try WorkspaceModel(url: ws, secrets: store, defaults: defaults)
    }

    /// A promoted endpoint saved with the given URLs: version 1 = promote, then one per URL.
    private func endpointModel(urls: [String], maxVersions: Int? = nil) throws -> WorkspaceModel {
        let model = try makeModel(maxVersions: maxVersions)
        model.newScratch()
        model.promoteSelected()
        for url in urls {
            model.draft?.url = url
            model.saveCurrent()
        }
        return model
    }

    func testLoadVersionsNewestFirstWithSummaries() throws {
        let model = try endpointModel(urls: ["{{BASE_URL}}/a", "{{BASE_URL}}/b"])
        model.loadVersions()
        XCTAssertEqual(model.versionRows.map(\.version), [3, 2, 1])
        XCTAssertEqual(model.versionRows[0].summary, "url")
        XCTAssertEqual(model.versionRows[2].summary, "Initial version")
    }

    func testSelectingVersionShowsDiffAgainstPredecessor() throws {
        let model = try endpointModel(urls: ["{{BASE_URL}}/a"])
        model.loadVersions()
        model.selectVersion(2)
        XCTAssertEqual(model.selectedVersion, 2)
        XCTAssertEqual(model.diffRows.map(\.field), ["url"])
        XCTAssertEqual(model.diffRows.first?.kind, .changed)
        model.selectVersion(1)
        XCTAssertTrue(model.diffRows.isEmpty)
    }

    func testRestoreCreatesNewVersionAndReloadsDraft() throws {
        let model = try endpointModel(urls: ["{{BASE_URL}}/a", "{{BASE_URL}}/b"])
        model.loadVersions()
        model.requestRestore(2)
        XCTAssertEqual(model.pendingRestore, 2)
        model.confirmRestore()
        XCTAssertNil(model.pendingRestore)
        XCTAssertEqual(model.draft?.url, "{{BASE_URL}}/a")
        XCTAssertFalse(model.isDirty)
        XCTAssertEqual(model.versionRows.map(\.version), [4, 3, 2, 1])
        XCTAssertEqual(model.selectedVersion, 4)
    }

    func testCancelRestoreChangesNothing() throws {
        let model = try endpointModel(urls: ["{{BASE_URL}}/a"])
        model.loadVersions()
        model.requestRestore(1)
        model.cancelRestore()
        XCTAssertNil(model.pendingRestore)
        XCTAssertEqual(model.versionRows.count, 2)
    }

    func testRestoreSavesUnsavedEditsFirst() throws {
        let model = try endpointModel(urls: ["{{BASE_URL}}/a"])
        model.draft?.url = "{{BASE_URL}}/unsaved"
        model.loadVersions()
        model.requestRestore(1)
        model.confirmRestore()
        // unsaved edit became v3, restore of v1 became v4
        XCTAssertEqual(model.versionRows.map(\.version), [4, 3, 2, 1])
        XCTAssertEqual(model.draft?.url, "{{BASE_URL}}/")
    }

    func testVersionsOnlyForEndpoints() throws {
        let model = try makeModel()
        model.newScratch()
        model.loadVersions()
        XCTAssertTrue(model.versionRows.isEmpty)
    }

    func testStoredMaxVersionsAppliedOnOpen() throws {
        let model = try endpointModel(
            urls: (1...7).map { "{{BASE_URL}}/\($0)" }, maxVersions: 5)
        model.loadVersions()
        XCTAssertEqual(model.versionRows.count, 5)
        XCTAssertEqual(model.versionRows.first?.version, 8)
    }

    func testEndpointSendIsAttributedInHistory() async throws {
        let model = try makeModel()
        let history = try XRestHistory.open(path: dir.appendingPathComponent("h.sqlite").path)
        model.core.setHistory(history: history)
        model.newScratch()
        model.promoteSelected()
        guard case .endpoint(let id)? = model.selection else { return XCTFail("no endpoint") }
        model.draft?.url = "http://127.0.0.1:1/"
        model.requestSend()
        for _ in 0..<100 where model.isSending || model.sendError == nil && model.response == nil {
            try await Task.sleep(for: .milliseconds(50))
        }
        let all = try history.list(
            filter: HistoryFilter(serviceId: nil, endpointId: nil, method: nil, statusClass: nil), limit: 10,
            offset: 0)
        XCTAssertEqual(all.count, 1)
        XCTAssertEqual(all.first?.endpointId, id)
    }
}
