import XCTest
@testable import XRest

final class DensityMetricsTests: XCTestCase {
    func testRegularMatchesCurrentLayout() {
        let m = DensityMetrics(.regular)
        XCTAssertEqual(m.rowHeight, 24)
        XCTAssertEqual(m.rowPadding, 4)
        XCTAssertEqual(m.tabHeight, 36)
        XCTAssertEqual(m.sidebarRowPadding, 6)
    }

    func testCompactIsStrictlySmaller() {
        let r = DensityMetrics(.regular)
        let c = DensityMetrics(.compact)
        XCTAssertEqual(c.rowHeight, 20)
        XCTAssertEqual(c.rowPadding, 2)
        XCTAssertEqual(c.tabHeight, 30)
        XCTAssertEqual(c.sidebarRowPadding, 4)
        XCTAssertLessThan(c.rowHeight, r.rowHeight)
        XCTAssertLessThan(c.rowPadding, r.rowPadding)
        XCTAssertLessThan(c.tabHeight, r.tabHeight)
        XCTAssertLessThan(c.sidebarRowPadding, r.sidebarRowPadding)
    }
}
