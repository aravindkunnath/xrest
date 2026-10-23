import XCTest

@testable import XRest

final class RecentWorkspacesTests: XCTestCase {
    private var defaults: UserDefaults!
    private var suite: String!
    private var existing: Set<String> = []

    override func setUp() {
        suite = "xrest.tests.\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suite)
        existing = []
    }

    override func tearDown() {
        defaults.removePersistentDomain(forName: suite)
    }

    private func url(_ n: Int) -> URL { URL(fileURLWithPath: "/ws/\(n)") }

    private func make(limit: Int = 10) -> RecentWorkspaces {
        RecentWorkspaces(defaults: defaults, limit: limit, exists: { [unowned self] in
            self.existing.contains($0.path)
        })
    }

    private func add(_ r: RecentWorkspaces, _ n: Int) {
        existing.insert(url(n).path)
        r.record(url(n))
    }

    func testMostRecentFirstAndDedup() {
        let r = make()
        add(r, 1); add(r, 2); add(r, 3); add(r, 1)
        XCTAssertEqual(r.list(), [url(1), url(3), url(2)])
    }

    func testCapsAtLimit() {
        let r = make(limit: 10)
        for n in 1...12 { add(r, n) }
        let list = r.list()
        XCTAssertEqual(list.count, 10)
        XCTAssertEqual(list.first, url(12))
        XCTAssertEqual(list.last, url(3))
    }

    func testPrunesMissingFoldersAndPersistsThePrune() {
        let r = make()
        add(r, 1); add(r, 2); add(r, 3)
        existing.remove(url(2).path)
        XCTAssertEqual(r.list(), [url(3), url(1)])
        // still gone even if the folder reappears: the prune was persisted
        existing.insert(url(2).path)
        XCTAssertEqual(r.list(), [url(3), url(1)])
    }

    func testPersistsAcrossInstancesAndClear() {
        add(make(), 1)
        let again = make()
        XCTAssertEqual(again.list(), [url(1)])
        again.clear()
        XCTAssertTrue(make().list().isEmpty)
    }

    func testEquivalentPathsDedup() {
        let r = make()
        existing.insert("/ws/1")
        r.record(URL(fileURLWithPath: "/ws/1/"))
        r.record(URL(fileURLWithPath: "/ws/./1"))
        XCTAssertEqual(r.list().count, 1)
    }
}
