import AppKit
import SwiftUI
import XCTest

@testable import XRest

final class AppPreferencesTests: XCTestCase {
    private var suite = ""
    private var defaults: UserDefaults!

    override func setUp() {
        suite = "xrest.tests.prefs.\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suite)!
    }

    override func tearDown() {
        defaults.removePersistentDomain(forName: suite)
    }

    func testDefaults() {
        let p = AppPreferences(defaults: defaults)
        XCTAssertEqual(p.appearance, .system)
        XCTAssertEqual(p.zoom, 1.0)
        XCTAssertEqual(p.timeoutSeconds, 30)
        XCTAssertTrue(p.followRedirects)
        XCTAssertTrue(p.verifyTLS)
        XCTAssertEqual(p.proxyMode, .system)
        XCTAssertEqual(p.maxVersions, 50)
    }

    func testRoundTripAndClamping() {
        let p = AppPreferences(defaults: defaults)
        p.appearance = .light
        p.zoom = 1.25
        p.timeoutSeconds = 5
        p.followRedirects = false
        p.verifyTLS = false
        p.proxyMode = .custom
        p.proxyURL = "http://proxy:8080"
        p.maxVersions = 10
        let q = AppPreferences(defaults: defaults)
        XCTAssertEqual(q.appearance, .light)
        XCTAssertEqual(q.zoom, 1.25)
        XCTAssertEqual(q.timeoutSeconds, 5)
        XCTAssertFalse(q.followRedirects)
        XCTAssertFalse(q.verifyTLS)
        XCTAssertEqual(q.proxyMode, .custom)
        XCTAssertEqual(q.proxyURL, "http://proxy:8080")
        XCTAssertEqual(q.maxVersions, 10)

        p.zoom = 9
        XCTAssertEqual(AppPreferences(defaults: defaults).zoom, 2.0)
        p.zoom = 0.1
        XCTAssertEqual(AppPreferences(defaults: defaults).zoom, 0.8)
        p.maxVersions = 1
        XCTAssertEqual(AppPreferences(defaults: defaults).maxVersions, 5)
        p.maxVersions = 500
        XCTAssertEqual(AppPreferences(defaults: defaults).maxVersions, 50)
        defaults.set("garbage", forKey: AppPreferences.Key.appearance)
        XCTAssertEqual(AppPreferences(defaults: defaults).appearance, .system)
        defaults.set(-3.0, forKey: AppPreferences.Key.timeoutSeconds)
        XCTAssertEqual(AppPreferences(defaults: defaults).timeoutSeconds, 30)
    }

    func testNewDefaults() {
        let p = AppPreferences(defaults: defaults)
        XCTAssertEqual(p.editorFontFamily, "")
        XCTAssertEqual(p.editorFontSize, 12)
        XCTAssertTrue(p.wrapLines)
        XCTAssertTrue(p.showLineNumbers)
        XCTAssertEqual(p.density, .regular)
        XCTAssertEqual(p.confirmStyle, .countdown)
        XCTAssertEqual(p.confirmCountdownSeconds, 3)
        XCTAssertTrue(p.returnCancels)
        XCTAssertTrue(p.warnUnresolved)
    }

    func testNewRoundTrip() {
        let p = AppPreferences(defaults: defaults)
        p.editorFontFamily = "Menlo"
        p.editorFontSize = 15
        p.wrapLines = false
        p.showLineNumbers = false
        p.density = .compact
        p.confirmStyle = .typeName
        p.confirmCountdownSeconds = 7
        p.returnCancels = false
        p.warnUnresolved = false
        let q = AppPreferences(defaults: defaults)
        XCTAssertEqual(q.editorFontFamily, "Menlo")
        XCTAssertEqual(q.editorFontSize, 15)
        XCTAssertFalse(q.wrapLines)
        XCTAssertFalse(q.showLineNumbers)
        XCTAssertEqual(q.density, .compact)
        XCTAssertEqual(q.confirmStyle, .typeName)
        XCTAssertEqual(q.confirmCountdownSeconds, 7)
        XCTAssertFalse(q.returnCancels)
        XCTAssertFalse(q.warnUnresolved)
    }

    func testNewClamping() {
        let p = AppPreferences(defaults: defaults)
        p.editorFontSize = 999
        XCTAssertEqual(p.editorFontSize, 24)
        p.editorFontSize = 1
        XCTAssertEqual(p.editorFontSize, 9)
        p.confirmCountdownSeconds = 0
        XCTAssertEqual(p.confirmCountdownSeconds, 1)
        p.confirmCountdownSeconds = 99
        XCTAssertEqual(p.confirmCountdownSeconds, 10)
    }

    func testBogusEnumFallsBack() {
        defaults.set("nonsense", forKey: AppPreferences.Key.density)
        defaults.set("nonsense", forKey: AppPreferences.Key.confirmStyle)
        let p = AppPreferences(defaults: defaults)
        XCTAssertEqual(p.density, .regular)
        XCTAssertEqual(p.confirmStyle, .countdown)
    }

    func testHttpOptionsMapping() {
        let p = AppPreferences(defaults: defaults)
        var o = p.httpOptions
        XCTAssertEqual(o.timeoutMs, 30_000)
        XCTAssertTrue(o.followRedirects)
        XCTAssertTrue(o.verifyTls)
        XCTAssertNil(o.proxy)

        p.timeoutSeconds = 2.5
        p.followRedirects = false
        p.verifyTLS = false
        p.proxyMode = .direct
        o = p.httpOptions
        XCTAssertEqual(o.timeoutMs, 2500)
        XCTAssertFalse(o.followRedirects)
        XCTAssertFalse(o.verifyTls)
        XCTAssertEqual(o.proxy, "")

        p.proxyMode = .custom
        p.proxyURL = "  http://proxy:8080 "
        XCTAssertEqual(p.httpOptions.proxy, "http://proxy:8080")
        p.proxyURL = "http://"
        XCTAssertNil(p.httpOptions.proxy, "an unfinished custom proxy falls back to the system proxy")
    }

    func testAppearanceMapsToColorScheme() {
        XCTAssertNil(AppearanceMode.system.colorScheme)
        XCTAssertEqual(AppearanceMode.light.colorScheme, .light)
        XCTAssertEqual(AppearanceMode.dark.colorScheme, .dark)
    }

    func testDynamicColorResolvesPerAppearance() throws {
        let color = NSColor.dynamic(dark: 0x000000, light: 0xFFFFFF)
        func brightness(_ name: NSAppearance.Name) throws -> CGFloat {
            var value: CGFloat = -1
            try XCTUnwrap(NSAppearance(named: name)).performAsCurrentDrawingAppearance {
                value = color.usingColorSpace(.sRGB)?.brightnessComponent ?? -1
            }
            return value
        }
        XCTAssertEqual(try brightness(.darkAqua), 0, accuracy: 0.01)
        XCTAssertEqual(try brightness(.aqua), 1, accuracy: 0.01)
    }

    @MainActor
    func testNewModelAppliesStoredMaxVersions() throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-prefs-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }
        _ = try XRestWorkspace.createWorkspace(path: dir.path, name: "svc")
        let store = KeychainSecretStore(
            service: "dev.xrest.tests.\(UUID().uuidString)", indexURL: dir.appendingPathComponent("idx.json"))
        let p = AppPreferences(defaults: defaults)
        p.maxVersions = 5
        let model = try WorkspaceModel(url: dir, secrets: store, defaults: defaults)
        model.newScratch()
        model.promoteSelected()
        for i in 1...8 {
            model.draft?.url = "{{BASE_URL}}/\(i)"
            model.saveCurrent()
        }
        model.loadVersions()
        XCTAssertEqual(model.versionRows.count, 5)
        // Lowering the setting while the window is open applies to later saves.
        p.maxVersions = 5
        model.applyPreferences(defaults: defaults)
    }
}
