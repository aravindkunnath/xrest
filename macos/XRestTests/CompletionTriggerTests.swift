import XCTest

@testable import XRest

final class CompletionTriggerTests: XCTestCase {
    private func detect(_ text: String, caret: Int? = nil) -> CompletionTrigger? {
        CompletionTrigger.detect(in: text, caret: caret ?? (text as NSString).length)
    }

    func testDetectsOpenBraces() {
        let t = detect("{{")
        XCTAssertEqual(t?.prefix, "")
        XCTAssertEqual(t?.range, NSRange(location: 0, length: 2))
        let u = detect("https://{{BA")
        XCTAssertEqual(u?.prefix, "BA")
        XCTAssertEqual(u?.range, NSRange(location: 8, length: 4))
    }

    func testNoTriggerWhenClosedOrNotOpen() {
        XCTAssertNil(detect(""))
        XCTAssertNil(detect("{"))
        XCTAssertNil(detect("{ {"))
        XCTAssertNil(detect("{{A}}"))
        XCTAssertNil(detect("{{A}} tail"))
        XCTAssertNil(detect("{{A b"))
        XCTAssertNil(detect("{{A\nB"))
        XCTAssertNil(detect("plain"))
    }

    func testUsesTheLastOpenBeforeTheCaret() {
        XCTAssertEqual(detect("{{A}}/{{B")?.prefix, "B")
        XCTAssertEqual(detect("{{A}}/{{B")?.range, NSRange(location: 6, length: 3))
    }

    func testCaretInTheMiddle() {
        // caret after "{{A" in "{{AB}} tail"
        let t = detect("{{AB}} tail", caret: 3)
        XCTAssertEqual(t?.prefix, "A")
        // caret after the closing braces: nothing open
        XCTAssertNil(detect("{{AB}} tail", caret: 6))
        // caret before the braces
        XCTAssertNil(detect("x{{AB", caret: 1))
    }

    func testCandidatesFilterAndIncludeSecrets() {
        let vars = ["BASE_URL", "TOKEN", "my_base"]
        let secrets = ["api", "tok"]
        XCTAssertEqual(
            CompletionTrigger.candidates(prefix: "", variables: vars, secretKeys: secrets),
            ["BASE_URL", "TOKEN", "my_base", "secret.api", "secret.tok"])
        // prefix matches first (case-insensitive), then substring matches
        XCTAssertEqual(
            CompletionTrigger.candidates(prefix: "base", variables: vars, secretKeys: secrets),
            ["BASE_URL", "my_base"])
        XCTAssertEqual(
            CompletionTrigger.candidates(prefix: "tok", variables: vars, secretKeys: secrets),
            ["TOKEN", "secret.tok"])
        XCTAssertEqual(
            CompletionTrigger.candidates(prefix: "secret.", variables: vars, secretKeys: secrets),
            ["secret.api", "secret.tok"])
        XCTAssertEqual(
            CompletionTrigger.candidates(prefix: "zzz", variables: vars, secretKeys: secrets), [])
    }

    func testInsertionReplacesPrefixAndMovesCaretPastBraces() {
        let text = "https://{{BA"
        let t = detect(text)!
        let r = t.insert("BASE_URL", into: text)
        XCTAssertEqual(r.text, "https://{{BASE_URL}}")
        XCTAssertEqual(r.caret, (r.text as NSString).length)
    }

    func testInsertionConsumesExistingTailAndKeepsTheRest() {
        let text = "{{AB}}/x"
        let t = detect(text, caret: 3)!
        let r = t.insert("API", into: text)
        XCTAssertEqual(r.text, "{{API}}/x")
        XCTAssertEqual(r.caret, 7)
        // bare "{{" followed by unrelated text is kept
        let t2 = detect("{{/x", caret: 2)!
        XCTAssertEqual(t2.insert("A", into: "{{/x").text, "{{A}}/x")
    }

    func testInsertionWithNonBmpText() {
        let text = "😀{{B"
        let t = detect(text)!
        XCTAssertEqual(t.insert("BASE", into: text).text, "😀{{BASE}}")
    }
}

final class PreviewSegmentsTests: XCTestCase {
    func testMarksLeftoverPlaceholdersAsUnresolved() {
        let segs = PreviewSegments.split("http://{{A}}/x?q={{secret.k}}")
        XCTAssertEqual(segs.map(\.text), ["http://", "{{A}}", "/x?q=", "{{secret.k}}"])
        XCTAssertEqual(segs.map(\.isUnresolved), [false, true, false, true])
    }

    func testPlainTextIsOneSegment() {
        XCTAssertEqual(PreviewSegments.split("http://h.io/x").count, 1)
        XCTAssertTrue(PreviewSegments.split("").isEmpty)
    }
}

final class VariableTokenTests: XCTestCase {
    func testFindsTokenUnderOffset() {
        let text = "{{BASE_URL}}/todos?k={{ secret.key }}"
        XCTAssertEqual(VariableToken.name(at: 0, in: text)?.name, "BASE_URL")
        XCTAssertEqual(VariableToken.name(at: 11, in: text)?.name, "BASE_URL")
        XCTAssertEqual(VariableToken.name(at: 25, in: text)?.name, "secret.key")
        XCTAssertEqual(VariableToken.name(at: 11, in: text)?.range, NSRange(location: 0, length: 12))
    }

    func testNoTokenOutsideOrWhenUnclosed() {
        XCTAssertNil(VariableToken.name(at: 12, in: "{{A}}/x"))
        XCTAssertNil(VariableToken.name(at: 2, in: "{{A"))
        XCTAssertNil(VariableToken.name(at: 0, in: ""))
    }

    func testOffsetFromX() {
        XCTAssertEqual(VariableToken.offset(atX: 0, charWidth: 7), 0)
        XCTAssertEqual(VariableToken.offset(atX: 15, charWidth: 7), 2)
        XCTAssertEqual(VariableToken.offset(atX: -1, charWidth: 7), -1)
    }
}
