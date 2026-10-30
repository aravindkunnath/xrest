import XCTest

@testable import XRest

@MainActor
final class HistoryModelTests: XCTestCase {
    private var dir: URL!

    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-hist-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    }

    override func tearDown() {
        try? FileManager.default.removeItem(at: dir)
    }

    private func makeModel(_ name: String = "svc", store: HistoryStore? = nil) throws -> WorkspaceModel {
        let ws = dir.appendingPathComponent(name)
        try FileManager.default.createDirectory(at: ws, withIntermediateDirectories: true)
        _ = try XRestWorkspace.createWorkspace(path: ws.path, name: name)
        let secrets = KeychainSecretStore(
            service: "dev.xrest.tests.\(UUID().uuidString)", indexURL: dir.appendingPathComponent("idx-\(name).json"))
        let model = try WorkspaceModel(url: ws, secrets: secrets)
        model.attachHistory(store ?? HistoryStore(path: dir.appendingPathComponent("history.sqlite")))
        return model
    }

    private func sendUnreachable(_ model: WorkspaceModel, path: String = "") async {
        model.newScratch()
        model.draft?.url = "http://127.0.0.1:1/\(path)"
        model.requestSend()
        for _ in 0..<200 where model.isSending || model.response == nil && model.sendError == nil {
            try? await Task.sleep(for: .milliseconds(25))
        }
    }

    func testStoreOpensInCreatedDirectoryAndReportsFailure() {
        let nested = dir.appendingPathComponent("a/b/history.sqlite")
        let ok = HistoryStore(path: nested)
        XCTAssertNotNil(ok.history)
        XCTAssertNil(ok.error)
        XCTAssertTrue(FileManager.default.fileExists(atPath: nested.path))

        let bad = HistoryStore(path: dir)  // a directory is not a database
        XCTAssertNil(bad.history)
        XCTAssertNotNil(bad.error)
    }

    func testFailedOpenKeepsAppWorkingAndPanelShowsError() throws {
        let model = try makeModel(store: HistoryStore(path: dir))
        model.openPanel(.history)
        XCTAssertNotNil(model.historyError)
        XCTAssertTrue(model.historyRows.isEmpty)
        model.newScratch()
        XCTAssertNotNil(model.draft)
    }

    func testSendRecordsAndPanelListsAndOpensAsScratch() async throws {
        let model = try makeModel()
        await sendUnreachable(model)
        model.openPanel(.history)
        XCTAssertEqual(model.historyRows.count, 1)
        XCTAssertEqual(model.historyRows[0].url, "http://127.0.0.1:1/")
        XCTAssertNil(model.historyRows[0].status)

        model.selectHistory(model.historyRows[0].id)
        XCTAssertEqual(model.historyDetail?.method, "GET")

        let before = model.scratch.count
        model.openHistoryAsScratch()
        XCTAssertEqual(model.scratch.count, before + 1)
        XCTAssertNil(model.panel)
        XCTAssertEqual(model.draft?.url, "http://127.0.0.1:1/")
        guard case .scratch? = model.selection else { return XCTFail("scratch not selected") }
    }

    func testListRefreshesWhenSendFinishesWhilePanelOpen() async throws {
        let model = try makeModel()
        model.newScratch()
        model.draft?.url = "http://127.0.0.1:1/late"
        model.requestSend()
        model.openPanel(.history)
        XCTAssertEqual(model.historyRows.count, 0)
        for _ in 0..<200 where model.historyRows.isEmpty {
            try? await Task.sleep(for: .milliseconds(25))
        }
        XCTAssertEqual(model.historyRows.count, 1)
    }

    func testPagingFilterAndClear() async throws {
        let model = try makeModel()
        for i in 0..<3 { await sendUnreachable(model, path: "\(i)") }
        model.historyPageSize = 2
        model.openPanel(.history)
        XCTAssertEqual(model.historyRows.count, 2)
        XCTAssertTrue(model.historyPaging.hasMore)
        model.loadMoreHistory()
        XCTAssertEqual(model.historyRows.count, 3)
        XCTAssertFalse(model.historyPaging.hasMore)
        XCTAssertEqual(model.historyRows.first?.url, "http://127.0.0.1:1/2", "newest first")

        model.historyFilter.method = "POST"
        model.reloadHistory()
        XCTAssertEqual(model.historyRows.count, 0)
        model.historyFilter.method = nil

        model.requestClearHistory()
        XCTAssertTrue(model.pendingClearHistory)
        model.confirmClearHistory()
        XCTAssertEqual(model.historyRows.count, 0)
        XCTAssertNil(model.historyDetail)
    }

    func testSecondWorkspaceSharesHistoryAndThisServiceFilter() async throws {
        let store = HistoryStore(path: dir.appendingPathComponent("shared.sqlite"))
        let a = try makeModel("a", store: store)
        let b = try makeModel("b", store: store)
        await sendUnreachable(a)
        b.openPanel(.history)
        XCTAssertEqual(b.historyRows.count, 0, "defaults to this service")
        b.historyFilter.thisServiceOnly = false
        b.reloadHistory()
        XCTAssertEqual(b.historyRows.count, 1)
    }
}
