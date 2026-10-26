import AppKit
import XCTest

@testable import XRest

final class CodeEditorFontTests: XCTestCase {
    func testUnknownFamilyFallsBack() {
        XCTAssertTrue(CodeEditor.font(family: "No Such Font", size: 12).isFixedPitch)
    }

    func testEmptyFamilyIsSystemMono() {
        XCTAssertTrue(CodeEditor.font(family: "", size: 12).isFixedPitch)
    }

    func testNonMonospacedFamilyFallsBack() {
        XCTAssertTrue(CodeEditor.font(family: "Helvetica", size: 12).isFixedPitch)
    }

    func testSizeScalesWithZoom() {
        XCTAssertEqual(CodeEditor.fontSize(zoom: 1.5, base: 12), 18)
    }

    func testBaseMatchesPreviousDefault() {
        XCTAssertEqual(CodeEditor.fontSize(zoom: 1, base: 12), 12)
    }

    func testLineCount() {
        XCTAssertEqual(LineNumberRuler.lineCount(in: ""), 1)
        XCTAssertEqual(LineNumberRuler.lineCount(in: "a"), 1)
        XCTAssertEqual(LineNumberRuler.lineCount(in: "a\n"), 2)
        XCTAssertEqual(LineNumberRuler.lineCount(in: "a\nb\nc"), 3)
        XCTAssertEqual(LineNumberRuler.lineCount(in: "\n\n"), 3)
    }
}
