import XCTest

@testable import XRest

/// The token rules mirror `substitute_path_params` in core/xrest-core/src/resolve.rs.
final class PathTokensTests: XCTestCase {
    func testExtractionTable() {
        let cases: [(String, [String])] = [
            ("http://h.io/users/{id}/posts", ["id"]),
            ("http://h.io/{a}/{b}", ["a", "b"]),
            // duplicates collapse, first occurrence order kept
            ("http://h.io/{a}/{b}/{a}", ["a", "b"]),
            // `{{…}}` placeholders are not path tokens
            ("http://h.io/{{NOPE}}/{id}", ["id"]),
            ("http://h.io/{{A}}", []),
            // query, fragment and host braces are ignored
            ("http://h.io/p/{id}?x={x}#f={f}", ["id"]),
            ("http://h.io/?x={x}", []),
            ("http://{host}/p", []),
            ("http://h.io#{frag}", []),
            // empty / unclosed / nested-open names
            ("http://h.io/{}", []),
            ("http://h.io/{unclosed", []),
            ("http://h.io/{a{b}", ["b"]),
            // the core allows any other characters in a name
            ("http://h.io/{a b}", ["a b"]),
            // unclosed `{{` swallows only the two braces, like the core
            ("http://h.io/{{a}/{b}", ["b"]),
            // no scheme: the core only substitutes after `://`
            ("/users/{id}", []),
            ("users/{id}", []),
            ("", []),
            // a leading variable stands for scheme://host, resolved before substitution
            ("{{BASE_URL}}/users/{id}", ["id"]),
            ("{{BASE_URL}}/users/{{ID}}", []),
            ("{{BASE_URL}}?x={x}", []),
            ("{{BASE_URL}}{id}", []),
            ("{{BASE_URL", []),
        ]
        for (url, expected) in cases {
            XCTAssertEqual(PathTokens.extract(from: url), expected, "url: \(url)")
        }
    }

    private func nv(_ name: String, _ value: String, enabled: Bool = true) -> NameValue {
        NameValue(name: name, value: value, enabled: enabled, secretKey: nil, kind: "plain")
    }

    func testSyncKeepsValuesOrdersByTokenAndDropsRemoved() {
        let existing = [nv("b", "2"), nv("gone", "x"), nv("a", "1", enabled: false)]
        let synced = PathTokens.synced(existing, tokens: ["a", "b", "c"])
        XCTAssertEqual(synced.map(\.name), ["a", "b", "c"])
        XCTAssertEqual(synced.map(\.value), ["1", "2", ""])
        XCTAssertEqual(synced.map(\.enabled), [false, true, true])
    }

    func testDraftDerivesPathParamsFromUrlEdits() {
        var draft = RequestDraft.blank
        draft.url = "http://h.io/u/{id}/p/{pid}"
        XCTAssertEqual(draft.pathParams.map(\.name), ["id", "pid"])
        draft.setPathParam("id", to: "42")
        XCTAssertEqual(draft.pathParamValue("id"), "42")
        XCTAssertEqual(draft.spec.pathParams.first?.value, "42")
        draft.url = "http://h.io/u/{id}"
        XCTAssertEqual(draft.pathParams.map(\.name), ["id"])
        XCTAssertEqual(draft.pathParamValue("id"), "42")
        draft.url = "http://h.io/u"
        XCTAssertTrue(draft.pathParams.isEmpty)
    }

    func testSettingValueForUnknownTokenIsIgnored() {
        var draft = RequestDraft.blank
        draft.url = "http://h.io/{id}"
        draft.setPathParam("zzz", to: "1")
        XCTAssertEqual(draft.pathParams.map(\.name), ["id"])
    }
}
