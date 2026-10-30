import XCTest

@testable import XRest

@MainActor
final class OpenTabsTests: XCTestCase {
    private var dir: URL!
    private var sessions: SessionStore!
    private var suite = ""

    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-tabs-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        sessions = SessionStore(directory: dir.appendingPathComponent("sessions"))
        suite = "xrest.tests.\(UUID().uuidString)"
        WorkspaceModel.isTerminating = false
    }

    override func tearDown() {
        WorkspaceModel.isTerminating = false
        try? FileManager.default.removeItem(at: dir)
        UserDefaults.standard.removePersistentDomain(forName: suite)
    }

    private var wsURL: URL { dir.appendingPathComponent("svc") }

    private func open() throws -> WorkspaceModel {
        if !FileManager.default.fileExists(atPath: wsURL.path) {
            try FileManager.default.createDirectory(at: wsURL, withIntermediateDirectories: true)
            _ = try XRestWorkspace.createWorkspace(path: wsURL.path, name: "svc")
        }
        let secrets = KeychainSecretStore(
            service: "dev.xrest.tests.\(UUID().uuidString)", indexURL: dir.appendingPathComponent("idx.json"))
        let m = try WorkspaceModel(url: wsURL, secrets: secrets, defaults: UserDefaults(suiteName: suite)!)
        m.attachSession(sessions)
        return m
    }

    /// Two saved endpoints, nothing open.
    private func modelWithEndpoints() throws -> (WorkspaceModel, SidebarSelection, SidebarSelection) {
        let m = try open()
        m.newScratch()
        m.promoteSelected()
        let a = try XCTUnwrap(m.selection)
        m.newScratch()
        m.promoteSelected()
        let b = try XCTUnwrap(m.selection)
        for tab in m.tabItems { m.closeTabNow(tab.id) }
        XCTAssertTrue(m.tabItems.isEmpty)
        return (m, a, b)
    }

    func testNewScratchAddsATabEachTime() throws {
        let m = try open()
        m.newScratch()
        m.newScratch()
        XCTAssertEqual(m.tabItems.count, 2)
        XCTAssertEqual(m.tabItems.last?.id, m.activeTabID)
    }

    func testSidebarClicksReuseOnePreviewTab() throws {
        let (m, a, b) = try modelWithEndpoints()
        m.select(a)
        m.select(b)
        XCTAssertEqual(m.tabItems.count, 1)
        XCTAssertEqual(m.selection, b)
        XCTAssertTrue(m.tabItems[0].isPreview)
    }

    func testEditingKeepsThePreviewTabSoTheNextClickOpensAnother() throws {
        let (m, a, b) = try modelWithEndpoints()
        m.select(a)
        m.draft?.url = "{{BASE_URL}}/edited"
        XCTAssertFalse(m.tabItems[0].isPreview)
        m.select(b)
        XCTAssertEqual(m.tabItems.count, 2)
        XCTAssertEqual(m.tabItems[0].selection, a)
        XCTAssertTrue(m.tabItems[0].isDirty, "edits stay in the background tab, unsaved")
    }

    func testSelectingAnOpenItemFocusesItsTab() throws {
        let (m, a, b) = try modelWithEndpoints()
        m.select(a)
        m.pinTab(try XCTUnwrap(m.activeTabID))
        m.select(b)
        XCTAssertEqual(m.tabItems.count, 2)
        m.select(a)
        XCTAssertEqual(m.tabItems.count, 2)
        XCTAssertEqual(m.selection, a)
    }

    func testSwitchingTabsKeepsEachDraft() throws {
        let (m, a, b) = try modelWithEndpoints()
        m.select(a)
        m.draft?.url = "{{BASE_URL}}/one"
        let first = try XCTUnwrap(m.activeTabID)
        m.select(b)
        m.draft?.url = "{{BASE_URL}}/two"
        XCTAssertEqual(m.tabItems.count, 2)
        m.activateTab(first)
        XCTAssertEqual(m.draft?.url, "{{BASE_URL}}/one")
        XCTAssertEqual(m.selection, a)
    }

    func testClosingACleanTabClosesImmediatelyAndFocusesANeighbour() throws {
        let m = try open()
        m.newScratch()
        m.newScratch()
        let last = try XCTUnwrap(m.activeTabID)
        m.requestCloseTab(last)
        XCTAssertNil(m.pendingTabClose)
        XCTAssertEqual(m.tabItems.count, 1)
        XCTAssertNotEqual(m.activeTabID, last)
        XCTAssertNotNil(m.draft)
    }

    func testClosingTheLastTabLeavesNoRequest() throws {
        let m = try open()
        m.newScratch()
        m.requestCloseTab(try XCTUnwrap(m.activeTabID))
        XCTAssertTrue(m.tabItems.isEmpty)
        XCTAssertNil(m.draft)
        XCTAssertNil(m.selection)
    }

    func testClosingADirtyTabAsksFirstAndDiscardKeepsDiskUntouched() throws {
        let (m, a, _) = try modelWithEndpoints()
        m.select(a)
        m.draft?.url = "{{BASE_URL}}/unsaved"
        let id = try XCTUnwrap(m.activeTabID)
        m.requestCloseTab(id)
        XCTAssertEqual(m.pendingTabClose, id)
        XCTAssertEqual(m.tabItems.count, 1)
        m.closeTabNow(id)
        guard case .endpoint(let endpoint) = a else { return XCTFail() }
        XCTAssertEqual(try m.core.getEndpoint(id: endpoint).spec.url, "{{BASE_URL}}/")
    }

    func testSaveAndCloseWritesTheDraft() throws {
        let (m, a, _) = try modelWithEndpoints()
        m.select(a)
        m.draft?.url = "{{BASE_URL}}/saved"
        m.saveAndCloseTab(try XCTUnwrap(m.activeTabID))
        XCTAssertTrue(m.tabItems.isEmpty)
        guard case .endpoint(let endpoint) = a else { return XCTFail() }
        XCTAssertEqual(try m.core.getEndpoint(id: endpoint).spec.url, "{{BASE_URL}}/saved")
    }

    func testDeletingAnItemClosesItsTabs() throws {
        let (m, a, b) = try modelWithEndpoints()
        m.select(a)
        m.draft?.url = "{{BASE_URL}}/x"
        m.select(b)
        XCTAssertEqual(m.tabItems.count, 2)
        m.delete(a)
        XCTAssertEqual(m.tabItems.count, 1)
        XCTAssertEqual(m.selection, b)
    }

    func testTabsAndTheirDraftsSurviveARestart() throws {
        let (first, a, b) = try modelWithEndpoints()
        first.select(a)
        first.draft?.url = "{{BASE_URL}}/kept"
        first.select(b)
        first.persistSessionNow()

        let second = try open()
        second.restoreSession()
        XCTAssertEqual(second.tabItems.map(\.selection), [a, b])
        XCTAssertEqual(second.selection, b)
        XCTAssertEqual(second.tabItems[0].draft?.url, "{{BASE_URL}}/kept")
        XCTAssertTrue(second.tabItems[0].isDirty)
        XCTAssertFalse(second.tabItems[1].isDirty)
    }

    func testOldSingleSelectionSessionsStillRestore() throws {
        let (first, a, _) = try modelWithEndpoints()
        first.select(a)
        first.persistSessionNow()
        var state = try XCTUnwrap(sessions.load(path: SessionStore.normalize(wsURL.path)))
        state.tabs = nil
        state.activeTab = nil
        try sessions.save(state)

        let second = try open()
        second.restoreSession()
        XCTAssertEqual(second.tabItems.count, 1)
        XCTAssertEqual(second.selection, a)
    }
}
