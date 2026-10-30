import XCTest

@testable import XRest

final class SubmitGateTests: XCTestCase {
    func testAcceptSuppressesNextSubmitOnce() {
        var gate = SubmitGate()
        gate.markAccepted()
        XCTAssertFalse(gate.shouldSubmit())
        XCTAssertTrue(gate.shouldSubmit())
    }

    func testSubmitWithoutAcceptGoesThrough() {
        var gate = SubmitGate()
        XCTAssertTrue(gate.shouldSubmit())
    }

    func testClearDropsStaleAccept() {
        var gate = SubmitGate()
        gate.markAccepted()
        gate.clear()
        XCTAssertTrue(gate.shouldSubmit())
    }
}
