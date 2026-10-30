import XCTest

@testable import XRest

final class ResponseBodyTests: XCTestCase {
    func testResponseKindTable() {
        let cases: [(String, Bool, ResponseKind)] = [
            ("application/json", false, .json),
            ("application/json; charset=utf-8", false, .json),
            ("Application/Problem+JSON", false, .json),
            ("text/html; charset=utf-8", false, .html),
            ("application/xhtml+xml", false, .html),
            ("image/png", true, .image),
            ("image/svg+xml", false, .text),
            ("application/xml", false, .text),
            ("text/xml", false, .text),
            ("text/plain", false, .text),
            ("", false, .text),
            ("application/octet-stream", true, .binary),
            ("application/json", true, .binary),
            ("text/html", true, .binary),
            ("", true, .binary),
        ]
        for (type, b64, expected) in cases {
            XCTAssertEqual(ResponseKind.classify(contentType: type, bodyIsBase64: b64), expected, "\(type) b64=\(b64)")
        }
    }

    func testHexDumpFormatting() {
        let data = Data("Hello, hex dump!\u{0}A".utf8)  // 18 bytes
        let lines = HexDump.format(data).split(separator: "\n").map(String.init)
        XCTAssertEqual(lines.count, 2)
        XCTAssertEqual(
            lines[0], "00000000  48 65 6c 6c 6f 2c 20 68  65 78 20 64 75 6d 70 21  |Hello, hex dump!|")
        XCTAssertEqual(lines[1], "00000010  00 41" + String(repeating: " ", count: 45) + "|.A|")
    }

    func testHexDumpLimitsToFirst64KiB() {
        let data = Data(repeating: 0xAB, count: HexDump.limit + 100)
        let out = HexDump.format(data)
        XCTAssertEqual(out.split(separator: "\n").count, HexDump.limit / 16 + 1)
        XCTAssertTrue(out.hasSuffix("(showing first 65536 of 65636 bytes)"))
    }

    func testTruncationThreshold() {
        let limit = BodyTruncation.limit
        XCTAssertEqual(limit, 2 * 1024 * 1024)
        let exact = String(repeating: "a", count: limit)
        XCTAssertFalse(BodyTruncation.apply(exact).truncated)
        let over = exact + "a"
        let r = BodyTruncation.apply(over)
        XCTAssertTrue(r.truncated)
        XCTAssertEqual(r.text.utf8.count, limit)
    }

    func testTruncationKeepsCharacterBoundary() {
        let s = "aé" + String(repeating: "b", count: 10)  // é is 2 bytes
        let r = BodyTruncation.apply(s, limit: 2)  // cuts inside é
        XCTAssertTrue(r.truncated)
        XCTAssertEqual(r.text, "a")
    }

    func testBase64DecodeForSave() {
        XCTAssertEqual(ResponseBodyBytes.decode(body: "AAEC/w==", isBase64: true), Data([0, 1, 2, 255]))
        XCTAssertEqual(ResponseBodyBytes.decode(body: "héllo", isBase64: false), Data("héllo".utf8))
        XCTAssertNil(ResponseBodyBytes.decode(body: "!!!not base64", isBase64: true))
    }

    func testStatusTone() {
        XCTAssertEqual(StatusTone.of(204), .success)
        XCTAssertEqual(StatusTone.of(301), .redirect)
        XCTAssertEqual(StatusTone.of(404), .error)
        XCTAssertEqual(StatusTone.of(503), .error)
    }

    func testCancellationDetection() {
        XCTAssertTrue(SendCancellation.isCancellation(CancellationError()))
        XCTAssertFalse(SendCancellation.isCancellation(NSError(domain: "x", code: 1)))
    }
}
