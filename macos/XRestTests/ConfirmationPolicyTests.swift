import XCTest

@testable import XRest

final class ConfirmationPolicyTests: XCTestCase {
    func testCountdownBlocksUntilZero() {
        XCTAssertFalse(ConfirmationPolicy.canSend(style: .countdown, remaining: 2, typed: "", environment: "PROD"))
        XCTAssertTrue(ConfirmationPolicy.canSend(style: .countdown, remaining: 0, typed: "", environment: "PROD"))
    }

    func testConfirmAllowsOnceAnyCountdownHasRunOut() {
        XCTAssertTrue(ConfirmationPolicy.canSend(style: .confirm, remaining: 0, typed: "", environment: "PROD"))
        // A team minimum runs a countdown even for the confirm style.
        XCTAssertFalse(ConfirmationPolicy.canSend(style: .confirm, remaining: 5, typed: "", environment: "PROD"))
        XCTAssertFalse(ConfirmationPolicy.canSend(style: .typeName, remaining: 2, typed: "PROD", environment: "PROD"))
    }

    func testTeamMinimumIsAFloorForEveryStyle() {
        XCTAssertEqual(ConfirmationPolicy.countdownDuration(style: .countdown, seconds: 3, teamMinimum: 5), 5)
        XCTAssertEqual(ConfirmationPolicy.countdownDuration(style: .countdown, seconds: 8, teamMinimum: 5), 8)
        XCTAssertEqual(ConfirmationPolicy.countdownDuration(style: .confirm, seconds: 8, teamMinimum: 5), 5)
        XCTAssertEqual(ConfirmationPolicy.countdownDuration(style: .typeName, seconds: 8, teamMinimum: 0), 0)
        XCTAssertEqual(ConfirmationPolicy.countdownDuration(style: .confirm, seconds: 8, teamMinimum: -1), 0)
    }

    func testTypeNameRequiresTrimmedExactMatch() {
        XCTAssertTrue(ConfirmationPolicy.canSend(style: .typeName, remaining: 0, typed: " PROD ", environment: "PROD"))
        XCTAssertFalse(ConfirmationPolicy.canSend(style: .typeName, remaining: 0, typed: "prod", environment: "PROD"))
        XCTAssertFalse(ConfirmationPolicy.canSend(style: .typeName, remaining: 0, typed: "", environment: "PROD"))
    }

    func testCountdownDurationOnlyForCountdownStyle() {
        XCTAssertEqual(ConfirmationPolicy.countdownDuration(style: .countdown, seconds: 7), 7)
        XCTAssertEqual(ConfirmationPolicy.countdownDuration(style: .confirm, seconds: 7), 0)
        XCTAssertEqual(ConfirmationPolicy.countdownDuration(style: .typeName, seconds: 7), 0)
    }
}
