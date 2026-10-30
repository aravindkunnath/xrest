import XCTest

@testable import XRest

final class SettingsSectionTests: XCTestCase {
    func testOrderAndTitles() {
        XCTAssertEqual(
            SettingsSection.allCases.map(\.title),
            ["Appearance", "Requests", "Variables", "Versions", "Safety"])
    }

    func testUnknownStoredTabFallsBackToAppearance() {
        XCTAssertEqual(SettingsSection(rawValue: "general") ?? .appearance, .appearance)
    }
}
