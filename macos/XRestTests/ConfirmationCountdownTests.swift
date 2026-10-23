import XCTest

@testable import XRest

@MainActor
final class ConfirmationCountdownTests: XCTestCase {
    func testStartsRunningAtTen() {
        let c = ConfirmationCountdown()
        XCTAssertEqual(c.state, .running(remaining: 10))
        XCTAssertEqual(c.progress, 1.0, accuracy: 0.0001)
    }

    func testReachingZeroCancels() {
        let c = ConfirmationCountdown(duration: 3)
        c.tick(); c.tick()
        XCTAssertEqual(c.state, .running(remaining: 1))
        c.tick()
        XCTAssertEqual(c.state, .cancelled)
        XCTAssertFalse(c.accept())
        XCTAssertEqual(c.state, .cancelled)
    }

    func testAcceptWhileRunningOnlyOnce() {
        let c = ConfirmationCountdown()
        c.tick()
        XCTAssertTrue(c.accept())
        XCTAssertEqual(c.state, .accepted)
        XCTAssertFalse(c.accept())
        c.tick()
        XCTAssertEqual(c.state, .accepted)
    }

    func testCancelThenAcceptFails() {
        let c = ConfirmationCountdown()
        c.cancel()
        XCTAssertEqual(c.state, .cancelled)
        XCTAssertFalse(c.accept())
    }

    func testResetRestoresTen() {
        let c = ConfirmationCountdown()
        c.tick(); c.tick()
        c.cancel()
        c.reset()
        XCTAssertEqual(c.state, .running(remaining: 10))
    }

    func testPendingRunsActionOnceOnAccept() {
        var runs = 0
        let p = PendingConfirmation(environment: "prod", method: "POST", url: "https://x/y") { runs += 1 }
        p.accept(); p.accept()
        XCTAssertEqual(runs, 1)
    }

    func testPendingExpiryNeverRunsAction() {
        var runs = 0
        let p = PendingConfirmation(
            environment: "prod", method: "GET", url: "u", countdown: ConfirmationCountdown(duration: 2)
        ) { runs += 1 }
        p.countdown.tick(); p.countdown.tick()
        p.accept()
        XCTAssertEqual(runs, 0)
        XCTAssertEqual(p.countdown.state, .cancelled)
    }

    func testPendingCancelNeverRunsAction() {
        var runs = 0
        let p = PendingConfirmation(environment: "prod", method: "GET", url: "u") { runs += 1 }
        p.cancel(); p.accept()
        XCTAssertEqual(runs, 0)
    }

    func testReopeningGivesFreshTen() {
        let a = PendingConfirmation(environment: "prod", method: "GET", url: "u") {}
        a.countdown.tick(); a.countdown.tick()
        let b = PendingConfirmation(environment: "prod", method: "GET", url: "u") {}
        XCTAssertEqual(b.countdown.state, .running(remaining: 10))
    }
}
