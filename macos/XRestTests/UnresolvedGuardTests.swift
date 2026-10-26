import XCTest
@testable import XRest

final class UnresolvedGuardTests: XCTestCase {
    func testAsksOnlyWhenEnabledAndNonEmpty() {
        XCTAssertTrue(UnresolvedGuard.shouldAsk(unresolved: ["a"], enabled: true))
        XCTAssertFalse(UnresolvedGuard.shouldAsk(unresolved: ["a"], enabled: false))
        XCTAssertFalse(UnresolvedGuard.shouldAsk(unresolved: [], enabled: true))
        XCTAssertFalse(UnresolvedGuard.shouldAsk(unresolved: [], enabled: false))
    }
}
