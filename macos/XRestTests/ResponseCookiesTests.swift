import XCTest

@testable import XRest

final class ResponseCookiesTests: XCTestCase {
    private func response(cookies: [ResponseCookie]) -> HttpResponse {
        HttpResponse(
            status: 200, statusText: "OK", headers: [], contentType: "", body: "", bodyIsBase64: false,
            prettyBody: nil, sizeBytes: 0, timeMs: 0, finalUrl: "http://x/", environment: nil,
            unresolvedVariables: [], cookies: cookies)
    }

    func testCookiesTabHiddenWhenEmpty() {
        XCTAssertEqual(ResponseTab.visibleCases(for: response(cookies: [])), [.pretty, .raw, .headers])
    }

    func testCookiesTabShownLastWhenPresent() {
        let cookie = ResponseCookie(
            name: "sid", value: "1", domain: nil, path: "/", expires: nil, maxAge: nil, secure: true,
            httpOnly: false, sameSite: nil)
        XCTAssertEqual(
            ResponseTab.visibleCases(for: response(cookies: [cookie])), [.pretty, .raw, .headers, .cookies])
    }
}
