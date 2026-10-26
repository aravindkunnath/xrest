import Observation
import XCTest

@testable import XRest

final class PreferencesStoreTests: XCTestCase {
    private func makeDefaults() -> UserDefaults {
        let name = "PreferencesStoreTests.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: name)!
        addTeardownBlock { defaults.removePersistentDomain(forName: name) }
        return defaults
    }

    func testReflectsDefaults() {
        let store = PreferencesStore(defaults: makeDefaults())
        XCTAssertEqual(store.editorFontFamily, "")
        XCTAssertEqual(store.editorFontSize, 12)
        XCTAssertTrue(store.wrapLines)
        XCTAssertTrue(store.showLineNumbers)
        XCTAssertEqual(store.density, .regular)
    }

    func testRefreshPicksUpChange() {
        let defaults = makeDefaults()
        let store = PreferencesStore(defaults: defaults)
        let prefs = AppPreferences(defaults: defaults)
        prefs.editorFontFamily = "Menlo"
        prefs.editorFontSize = 16
        prefs.wrapLines = false
        prefs.showLineNumbers = false
        prefs.density = .compact
        store.refresh(from: defaults)
        XCTAssertEqual(store.editorFontFamily, "Menlo")
        XCTAssertEqual(store.editorFontSize, 16)
        XCTAssertFalse(store.wrapLines)
        XCTAssertFalse(store.showLineNumbers)
        XCTAssertEqual(store.density, .compact)
    }

    func testNoRepublishWhenUnchanged() {
        let defaults = makeDefaults()
        let store = PreferencesStore(defaults: defaults)
        var fired = false
        withObservationTracking {
            _ = store.density
        } onChange: {
            fired = true
        }
        store.refresh(from: defaults)
        XCTAssertFalse(fired)
        AppPreferences(defaults: defaults).density = .compact
        store.refresh(from: defaults)
        XCTAssertTrue(fired)
    }
}
