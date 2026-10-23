import XCTest

@testable import XRest

/// Drives the model against real git repositories in a temp dir. The process environment is
/// isolated (no user/system git config, fixed identity) for the duration of each test.
@MainActor
final class GitModelTests: XCTestCase {
    private var dir: URL!
    private var saved: [String: String?] = [:]

    private static let env = [
        "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_AUTHOR_NAME": "Test", "GIT_AUTHOR_EMAIL": "t@example.com",
        "GIT_COMMITTER_NAME": "Test", "GIT_COMMITTER_EMAIL": "t@example.com",
        "GIT_TERMINAL_PROMPT": "0",
    ]

    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-git-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        for (k, v) in Self.env {
            saved[k] = ProcessInfo.processInfo.environment[k]
            setenv(k, v, 1)
        }
    }

    override func tearDown() {
        for (k, v) in saved {
            if let v { setenv(k, v, 1) } else { unsetenv(k) }
        }
        try? FileManager.default.removeItem(at: dir)
    }

    private func makeModel(_ name: String = "svc") throws -> WorkspaceModel {
        let ws = dir.appendingPathComponent(name)
        try FileManager.default.createDirectory(at: ws, withIntermediateDirectories: true)
        _ = try XRestWorkspace.createWorkspace(path: ws.path, name: name)
        let store = KeychainSecretStore(
            service: "dev.xrest.tests.\(UUID().uuidString)",
            indexURL: dir.appendingPathComponent("idx-\(name).json"))
        return try WorkspaceModel(url: ws, secrets: store)
    }

    private func bareRemote() throws -> String {
        let bare = dir.appendingPathComponent("remote.git")
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "/usr/bin/git")
        p.arguments = ["init", "--bare", "-b", "main", bare.path]
        try p.run()
        p.waitUntilExit()
        XCTAssertEqual(p.terminationStatus, 0)
        return bare.path
    }

    func testNonRepoStatusAndInit() async throws {
        let model = try makeModel()
        await model.refreshGitStatus()
        XCTAssertEqual(model.gitStatus?.isRepo, false)
        XCTAssertFalse(model.gitBadgeVisible)
        await model.gitInit(remote: nil)
        XCTAssertNil(model.gitError)
        XCTAssertEqual(model.gitStatus?.isRepo, true)
        XCTAssertTrue(model.gitBadgeVisible, "fresh workspace has uncommitted files")
    }

    func testCommitClearsBadge() async throws {
        let model = try makeModel()
        await model.gitInit(remote: nil)
        await model.gitCommit(message: "")
        XCTAssertNil(model.gitError)
        XCTAssertEqual(model.gitStatus?.uncommitted.count, 0)
        XCTAssertFalse(model.gitBadgeVisible)
    }

    func testSaveRefreshesStatus() async throws {
        let model = try makeModel()
        await model.gitInit(remote: nil)
        await model.gitCommit(message: "init")
        model.newScratch()
        model.promoteSelected()
        await model.refreshGitStatus()
        XCTAssertTrue(model.gitBadgeVisible)
        await model.gitCommit(message: "add")
        model.draft?.name = "renamed"
        model.saveCurrent()
        await model.gitRefreshTask?.value
        XCTAssertEqual(model.gitStatus?.uncommitted.count, 1)
    }

    func testAutoCommitToggle() async throws {
        let model = try makeModel()
        await model.gitInit(remote: nil)
        XCTAssertFalse(model.autoCommit)
        model.setAutoCommit(true)
        XCTAssertTrue(model.autoCommit)
        model.setAutoCommit(false)
        XCTAssertFalse(model.autoCommit)
    }

    func testPullReloadsCleanDraftAndKeepsDirtyEdits() async throws {
        let remote = try bareRemote()
        let a = try makeModel("a")
        a.newScratch()
        a.promoteSelected()
        guard case .endpoint(let id)? = a.selection else { return XCTFail("no endpoint") }
        await a.gitInit(remote: remote)
        await a.gitCommit(message: "init")
        let branch = a.gitStatus?.branch ?? "main"
        try await a.core.gitPush()

        // A second clone edits the endpoint and pushes.
        let clone = dir.appendingPathComponent("b")
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "/usr/bin/git")
        p.arguments = ["clone", "-b", branch, remote, clone.path]
        try p.run()
        p.waitUntilExit()
        XCTAssertEqual(p.terminationStatus, 0)
        let b = try XRestWorkspace.openWorkspace(path: clone.path)
        let current = try b.getEndpoint(id: id)
        _ = try b.updateEndpoint(id: id, name: "from remote", spec: current.spec)
        _ = try await b.gitCommit(message: "remote edit")
        try await b.gitPush()

        await a.gitPull()
        XCTAssertNil(a.gitError)
        XCTAssertEqual(a.draft?.name, "from remote")
        XCTAssertEqual(a.endpoints.first?.name, "from remote")
        XCTAssertNil(a.gitNotice)

        // Second round: unsaved local edits survive and a notice is shown.
        let again = try b.getEndpoint(id: id)
        _ = try b.updateEndpoint(id: id, name: "remote two", spec: again.spec)
        _ = try await b.gitCommit(message: "remote edit 2")
        try await b.gitPush()
        a.draft?.name = "local unsaved"
        await a.gitPull()
        XCTAssertEqual(a.draft?.name, "local unsaved")
        XCTAssertEqual(a.endpoints.first?.name, "remote two")
        XCTAssertNotNil(a.gitNotice)
    }

    func testGitFailureSurfacesStderr() async throws {
        let model = try makeModel()
        await model.gitInit(remote: nil)
        await model.gitPush()  // no remote configured
        XCTAssertNotNil(model.gitError)
        XCTAssertNil(model.gitBusy)
    }
}
