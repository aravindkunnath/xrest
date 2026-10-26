import XCTest

@testable import XRest

final class ConfirmationPolicyTests: XCTestCase {
    func testCountdownBlocksUntilZero() {
        XCTAssertFalse(ConfirmationPolicy.canSend(style: .countdown, remaining: 2, typed: "", environment: "PROD"))
        XCTAssertTrue(ConfirmationPolicy.canSend(style: .countdown, remaining: 0, typed: "", environment: "PROD"))
    }

    func testConfirmAlwaysAllows() {
        XCTAssertTrue(ConfirmationPolicy.canSend(style: .confirm, remaining: 5, typed: "", environment: "PROD"))
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
