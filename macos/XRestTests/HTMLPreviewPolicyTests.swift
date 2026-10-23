import XCTest

@testable import XRest

final class HTMLPreviewPolicyTests: XCTestCase {
    func testWaitsUntilRuleListIsReady() {
        XCTAssertEqual(HTMLPreviewPolicy.decision(for: .pending), .wait)
    }

    func testLoadsHTMLOnlyWhenRuleListIsInstalled() {
        XCTAssertEqual(HTMLPreviewPolicy.decision(for: .ready), .loadHTML)
    }

    func testFailsClosedToSourceWhenRuleListFails() {
        XCTAssertEqual(HTMLPreviewPolicy.decision(for: .failed), .showSource)
    }
}
