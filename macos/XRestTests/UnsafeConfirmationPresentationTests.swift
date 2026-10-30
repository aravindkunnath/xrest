import XCTest
@testable import XRest

final class UnsafeConfirmationPresentationTests: XCTestCase {
    private func make(
        reason: PendingConfirmation.Reason = .unsafeEnvironment,
        style: ConfirmStyle = .countdown, returnCancels: Bool = true,
        remaining: Int = 3, typed: String = "", teamMinimum: Int = 0
    ) -> UnsafeConfirmationPresentation {
        UnsafeConfirmationPresentation(
            reason: reason, style: style, environment: "PROD",
            returnCancels: returnCancels, remaining: remaining, typed: typed,
            teamMinimum: teamMinimum)
    }

    func testTeamMinimumRunsACountdownForTheConfirmStyle() {
        let running = make(style: .confirm, remaining: 4, teamMinimum: 5)
        XCTAssertTrue(running.showsCountdown)
        XCTAssertFalse(running.sendEnabled)
        XCTAssertEqual(running.sendLabel, "Send to PROD (4)")
        XCTAssertEqual(running.teamMinimumNote, "This workspace waits at least 5s before sending to PROD.")
        XCTAssertTrue(make(style: .confirm, remaining: 0, teamMinimum: 5).sendEnabled)
        XCTAssertNil(make(style: .countdown, remaining: 0).teamMinimumNote)
    }

    func testUnresolvedPromptIgnoresTeamMinimum() {
        let p = make(reason: .unresolved(["A"]), style: .confirm, remaining: 0, teamMinimum: 5)
        XCTAssertFalse(p.showsCountdown)
        XCTAssertNil(p.teamMinimumNote)
        XCTAssertTrue(p.sendEnabled)
    }

    func testCountdownRunningLocksAndShowsRemaining() {
        let p = make(style: .countdown, remaining: 3)
        XCTAssertEqual(p.title, "Send to PROD?")
        XCTAssertEqual(p.sendLabel, "Send to PROD (3)")
        XCTAssertFalse(p.sendEnabled)
        XCTAssertTrue(p.showsCountdown)
        XCTAssertFalse(p.showsTypeField)
    }

    func testCountdownReadyUnlocksWithoutZeroInLabel() {
        let p = make(style: .countdown, remaining: 0)
        XCTAssertEqual(p.sendLabel, "Send to PROD")
        XCTAssertTrue(p.sendEnabled)
        XCTAssertEqual(p.countdownCaption, "Ready")
        XCTAssertEqual(make(style: .countdown, remaining: 2).countdownCaption, "Unlocks in 2s")
    }

    func testConfirmIsImmediate() {
        let p = make(style: .confirm, remaining: 0)
        XCTAssertEqual(p.sendLabel, "Send to PROD")
        XCTAssertTrue(p.sendEnabled)
        XCTAssertFalse(p.showsCountdown)
    }

    func testTypeNameRequiresExactName() {
        XCTAssertFalse(make(style: .typeName, remaining: 0, typed: "").sendEnabled)
        XCTAssertFalse(make(style: .typeName, remaining: 0, typed: "prod").sendEnabled)
        XCTAssertFalse(make(style: .typeName, remaining: 0, typed: "PRO").sendEnabled)
        XCTAssertTrue(make(style: .typeName, remaining: 0, typed: "PROD").sendEnabled)
        XCTAssertTrue(make(style: .typeName, remaining: 0, typed: "  PROD ").sendEnabled)
        let p = make(style: .typeName, remaining: 0)
        XCTAssertTrue(p.showsTypeField)
        XCTAssertEqual(p.typePrompt, "Type PROD to confirm")
        XCTAssertEqual(p.sendLabel, "Send to PROD")
    }

    func testUnresolvedReasonIsZeroDelayConfirm() {
        let p = make(reason: .unresolved(["a", "b"]), style: .typeName, remaining: 5)
        XCTAssertEqual(p.title, "Send with unresolved variables?")
        XCTAssertTrue(p.sendEnabled)
        XCTAssertFalse(p.showsCountdown)
        XCTAssertFalse(p.showsTypeField)
        XCTAssertEqual(p.sendLabel, "Send Anyway")
    }

    func testReturnKeyDefaultButton() {
        XCTAssertEqual(make(returnCancels: true).defaultButton, .cancel)
        XCTAssertEqual(make(returnCancels: false).defaultButton, .send)
    }

    func testReturnCancelsHintFollowsSwitch() {
        XCTAssertEqual(
            UnsafeConfirmationPresentation.returnCancelsHint(returnCancels: true),
            "Pressing Return in the dialog never sends.")
        XCTAssertEqual(
            UnsafeConfirmationPresentation.returnCancelsHint(returnCancels: false),
            "Pressing Return sends once Send is unlocked.")
    }
}
