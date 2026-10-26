import XCTest

@testable import XRest

@MainActor
final class ConfirmationCountdownTests: XCTestCase {
    func testStartsRunningAtTen() {
        let c = ConfirmationCountdown()
        XCTAssertEqual(c.state, .running(remaining: 10))
        XCTAssertEqual(c.progress, 1.0, accuracy: 0.0001)
    }

    func testReachingZeroUnlocksWithoutCancelling() {
        let c = ConfirmationCountdown(duration: 3)
        c.tick(); c.tick()
        XCTAssertEqual(c.state, .running(remaining: 1))
        XCTAssertFalse(c.isReady)
        c.tick()
        XCTAssertEqual(c.state, .ready)
        XCTAssertTrue(c.isReady)
        XCTAssertFalse(c.isRunning)
        XCTAssertTrue(c.accept())
    }

    func testZeroDurationStartsReady() {
        let c = ConfirmationCountdown(duration: 0)
        XCTAssertEqual(c.state, .ready)
        XCTAssertTrue(c.isReady)
        XCTAssertTrue(c.accept())
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

    func testPendingCancelAfterReadyNeverRunsAction() {
        var runs = 0
        let p = PendingConfirmation(
            environment: "prod", method: "GET", url: "u", countdown: ConfirmationCountdown(duration: 2)
        ) { runs += 1 }
        p.countdown.tick(); p.countdown.tick()
        XCTAssertEqual(p.countdown.state, .ready)
        p.cancel(); p.accept()
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

@MainActor
final class PendingConfirmationURLTests: XCTestCase {
    func testCarriesResolvedURLAlongsideRaw() {
        let p = PendingConfirmation(
            environment: "prod", method: "GET", url: "{{BASE}}/x",
            resolvedURL: "https://api.io/x", action: {})
        XCTAssertEqual(p.url, "{{BASE}}/x")
        XCTAssertEqual(p.resolvedURL, "https://api.io/x")
        let q = PendingConfirmation(environment: "prod", method: "GET", url: "u", action: {})
        XCTAssertNil(q.resolvedURL)
    }
}
