import XCTest

@testable import XRest

final class GitSupportTests: XCTestCase {
    private func status(
        isRepo: Bool = true, files: [GitFileStatus] = [], ahead: UInt32 = 0, behind: UInt32 = 0,
        remote: String? = nil, lastSync: UInt64? = nil
    ) -> GitStatus {
        GitStatus(
            isRepo: isRepo, branch: "main", remoteUrl: remote, uncommitted: files, ahead: ahead,
            behind: behind, lastSync: lastSync)
    }

    private func file(_ path: String, _ s: String = "modified") -> GitFileStatus {
        GitFileStatus(path: path, status: s)
    }

    func testBadgeHiddenWhenNoStatusNotARepoOrClean() {
        XCTAssertFalse(GitBadge.isVisible(nil))
        XCTAssertFalse(GitBadge.isVisible(status(isRepo: false, files: [file("a")])))
        XCTAssertFalse(GitBadge.isVisible(status()))
        XCTAssertFalse(GitBadge.isVisible(status(behind: 3)))
    }

    func testBadgeShownForUncommittedOrUnpushed() {
        XCTAssertTrue(GitBadge.isVisible(status(files: [file("a")])))
        XCTAssertTrue(GitBadge.isVisible(status(ahead: 1)))
    }

    func testDefaultCommitMessage() {
        XCTAssertEqual(GitFormat.defaultCommitMessage([]), "Update workspace")
        XCTAssertEqual(GitFormat.defaultCommitMessage([file("endpoints/a.yaml")]), "Update endpoints/a.yaml")
        XCTAssertEqual(
            GitFormat.defaultCommitMessage([file("endpoints/a.yaml", "added")]), "Add endpoints/a.yaml")
        XCTAssertEqual(
            GitFormat.defaultCommitMessage([file("a", "untracked")]), "Add a")
        XCTAssertEqual(
            GitFormat.defaultCommitMessage([file("a", "deleted")]), "Remove a")
        XCTAssertEqual(
            GitFormat.defaultCommitMessage([file("a"), file("b"), file("c")]), "Update 3 files")
        XCTAssertEqual(
            GitFormat.defaultCommitMessage([file("a", "added"), file("b", "added")]), "Add 2 files")
    }

    func testStatusLabelAndSyncSummary() {
        XCTAssertEqual(GitFormat.statusLabel("untracked"), "New")
        XCTAssertEqual(GitFormat.statusLabel("modified"), "Modified")
        XCTAssertEqual(GitFormat.statusLabel("conflicted"), "Conflict")
        XCTAssertEqual(GitFormat.statusLabel("weird"), "weird")
        XCTAssertEqual(GitFormat.syncSummary(ahead: 0, behind: 0), "Up to date")
        XCTAssertEqual(GitFormat.syncSummary(ahead: 2, behind: 0), "2 ahead")
        XCTAssertEqual(GitFormat.syncSummary(ahead: 0, behind: 1), "1 behind")
        XCTAssertEqual(GitFormat.syncSummary(ahead: 2, behind: 1), "2 ahead, 1 behind")
    }

    func testLastSyncText() {
        let now = Date(timeIntervalSince1970: 10_000)
        XCTAssertEqual(GitFormat.lastSync(nil, now: now), "Never")
        XCTAssertEqual(GitFormat.lastSync(9_990, now: now), "Just now")
        XCTAssertEqual(GitFormat.lastSync(10_000 - 300, now: now), "5 min ago")
        XCTAssertEqual(GitFormat.lastSync(10_000 - 7_200, now: now), "2 h ago")
        XCTAssertEqual(GitFormat.lastSync(10_000 + 50, now: now), "Just now")
    }

    func testNewWarnings() {
        XCTAssertEqual(WarningTracker.newEntries(old: ["a"], new: ["a", "auto-commit failed: x"]),
            ["auto-commit failed: x"])
        XCTAssertEqual(WarningTracker.newEntries(old: ["a"], new: ["a"]), [])
        XCTAssertEqual(WarningTracker.newEntries(old: [], new: []), [])
    }

    func testReloadDecision() {
        let spec = RequestDraft.blank.spec
        let a = Endpoint(id: "x", name: "A", spec: spec, lastVersion: 1, lastUpdated: 0)
        var changed = a
        changed.name = "B"
        XCTAssertEqual(GitReload.decide(draftDirty: false, before: a, after: a), .keep)
        XCTAssertEqual(GitReload.decide(draftDirty: false, before: a, after: changed), .reloadDraft)
        XCTAssertEqual(GitReload.decide(draftDirty: true, before: a, after: changed), .keepEditsWithNotice)
        XCTAssertEqual(GitReload.decide(draftDirty: true, before: a, after: a), .keep)
        XCTAssertEqual(GitReload.decide(draftDirty: false, before: a, after: nil), .closeDeleted)
        XCTAssertEqual(GitReload.decide(draftDirty: true, before: a, after: nil), .keepEditsWithNotice)
        XCTAssertEqual(GitReload.decide(draftDirty: false, before: nil, after: nil), .keep)
    }
}
