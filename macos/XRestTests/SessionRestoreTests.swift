import XCTest

@testable import XRest

final class SessionStoreTests: XCTestCase {
    private var dir: URL!

    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-sess-\(UUID().uuidString)")
    }

    override func tearDown() { try? FileManager.default.removeItem(at: dir) }

    private func richDraft() -> RequestDraft {
        var d = RequestDraft.blank
        d.name = "Create user"
        d.method = "POST"
        d.url = "{{BASE_URL}}/users/{id}"
        d.setPathParam("id", to: "7")
        var h = KeyValueRow()
        h.name = "X-Token"
        h.value = "{{secret.token}}"
        h.secretKey = "token"
        d.headers = [h]
        var q = KeyValueRow()
        q.name = "q"
        q.value = "1"
        q.enabled = false
        d.params = [q]
        d.bodyKind = .json
        d.bodyText = #"{"a":1}"#
        d.formFields = [q]
        d.options = RequestOptions(timeoutMs: 1500, followRedirects: false, verifyTls: nil, proxy: "")
        return d
    }

    func testDraftSnapshotRoundTripsThroughJSON() throws {
        let draft = richDraft()
        let data = try JSONEncoder().encode(DraftSnapshot(draft))
        let back = try JSONDecoder().decode(DraftSnapshot.self, from: data)
        XCTAssertEqual(back.apply(to: RequestDraft.blank), draft)
    }

    func testSaveLoadRemoveAndDirectoryIsCreated() throws {
        let store = SessionStore(directory: dir)
        let state = SessionState(
            workspacePath: "/w/a", selection: SessionSelection(kind: .scratch, id: "s1"),
            draft: DraftSnapshot(richDraft()))
        XCTAssertNil(store.load(path: "/w/a"))
        try store.save(state)
        XCTAssertEqual(store.load(path: "/w/a"), state)
        XCTAssertNil(store.load(path: "/w/b"), "sessions are keyed by workspace path")
        store.remove(path: "/w/a")
        XCTAssertNil(store.load(path: "/w/a"))
    }

    func testAllReturnsEverySessionAndSkipsCorruptFiles() throws {
        let store = SessionStore(directory: dir)
        try store.save(SessionState(workspacePath: "/w/a", selection: nil, draft: nil))
        try store.save(SessionState(workspacePath: "/w/b", selection: nil, draft: nil))
        try Data("not json".utf8).write(to: dir.appendingPathComponent("junk.json"))
        XCTAssertEqual(Set(store.all().map(\.workspacePath)), ["/w/a", "/w/b"])
    }

    func testLaunchPlanDropsMissingWorkspacesAndDeletesTheirFiles() throws {
        let store = SessionStore(directory: dir)
        let alive = dir.appendingPathComponent("alive")
        try FileManager.default.createDirectory(at: alive, withIntermediateDirectories: true)
        try store.save(SessionState(workspacePath: alive.path, selection: nil, draft: nil))
        try store.save(SessionState(workspacePath: "/definitely/missing", selection: nil, draft: nil))
        let urls = store.workspacesToReopen()
        XCTAssertEqual(urls.map(\.path), [alive.path])
        XCTAssertNil(store.load(path: "/definitely/missing"))
    }
}

@MainActor
final class SessionRestoreModelTests: XCTestCase {
    private var dir: URL!
    private var sessions: SessionStore!
    private var suite = ""

    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-sr-\(UUID().uuidString)")
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

    private func endpointModel() throws -> WorkspaceModel {
        let m = try open()
        m.newScratch()
        m.promoteSelected()
        return m
    }

    func testQuitWithUnsavedDraftRestoresItAsUnsaved() throws {
        let first = try endpointModel()
        first.draft?.url = "{{BASE_URL}}/typed"
        XCTAssertTrue(first.isDirty)
        let selection = first.selection
        first.persistSessionNow()

        let second = try open()
        second.restoreSession()
        XCTAssertEqual(second.selection, selection)
        XCTAssertEqual(second.draft?.url, "{{BASE_URL}}/typed")
        XCTAssertTrue(second.isDirty, "restored drafts are unsaved edits, not written to the workspace")
        // The workspace on disk still has the old URL.
        guard case .endpoint(let id)? = selection else { return XCTFail() }
        XCTAssertEqual(try second.core.getEndpoint(id: id).spec.url, "{{BASE_URL}}/")
    }

    func testRestoresSelectionWithoutDraft() throws {
        let first = try endpointModel()
        let selection = first.selection
        first.persistSessionNow()
        let second = try open()
        second.restoreSession()
        XCTAssertEqual(second.selection, selection)
        XCTAssertFalse(second.isDirty)
    }

    func testDraftOfDeletedEndpointIsKeptAsScratchWithNotice() throws {
        let first = try endpointModel()
        first.draft?.name = "Typed name"
        first.draft?.url = "{{BASE_URL}}/typed"
        first.persistSessionNow()
        let deleted = try XCTUnwrap(first.selection)
        first.delete(deleted)

        let second = try open()
        second.restoreSession()
        guard case .scratch? = second.selection else { return XCTFail("\(String(describing: second.selection))") }
        XCTAssertEqual(second.draft?.url, "{{BASE_URL}}/typed")
        XCTAssertEqual(second.draft?.name, "Typed name")
        XCTAssertNotNil(second.warningBanner)
    }

    func testDeletedSelectionWithoutDraftIsJustDropped() throws {
        let first = try endpointModel()
        first.persistSessionNow()
        first.delete(try XCTUnwrap(first.selection))
        let second = try open()
        second.restoreSession()
        XCTAssertNil(second.selection)
        XCTAssertNil(second.draft)
    }

    func testCleanCloseWithoutDraftRemovesSessionFile() throws {
        let m = try endpointModel()
        m.persistSessionNow()
        XCTAssertNotNil(sessions.load(path: wsURL.path))
        m.windowClosed()
        XCTAssertNil(sessions.load(path: wsURL.path))
    }

    func testCloseWithDraftKeepsItAndTerminationNeverDeletes() throws {
        let m = try endpointModel()
        m.draft?.url = "{{BASE_URL}}/x"
        m.windowClosed()
        XCTAssertNotNil(sessions.load(path: wsURL.path)?.draft)

        let clean = try endpointModel()
        clean.persistSessionNow()
        WorkspaceModel.isTerminating = true
        clean.windowClosed()
        XCTAssertNotNil(sessions.load(path: wsURL.path), "windows torn down by quit stay in the session")
    }

    func testDebouncedWriteHappensAfterDelay() async throws {
        let m = try endpointModel()
        m.sessionSaveDelay = .milliseconds(20)
        m.draft?.url = "{{BASE_URL}}/debounced"
        m.scheduleSessionSave()
        m.draft?.url = "{{BASE_URL}}/debounced2"
        m.scheduleSessionSave()
        try await Task.sleep(for: .milliseconds(300))
        XCTAssertEqual(sessions.load(path: wsURL.path)?.draft.map { $0.apply(to: .blank).url }, "{{BASE_URL}}/debounced2")
    }
}
