import AppKit
import Observation
import SwiftUI
import XCTest

@testable import XRest

@MainActor
final class ZoomTests: XCTestCase {
    private var suite = ""
    private var defaults: UserDefaults!
    private var dir: URL!

    override func setUp() {
        suite = "xrest.tests.zoom.\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suite)!
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-zoom-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    }

    override func tearDown() {
        ZoomStore.shared.refresh(from: .standard)
        defaults.removePersistentDomain(forName: suite)
        try? FileManager.default.removeItem(at: dir)
    }

    private func makeModel() throws -> WorkspaceModel {
        let ws = dir.appendingPathComponent("svc")
        try FileManager.default.createDirectory(at: ws, withIntermediateDirectories: true)
        _ = try XRestWorkspace.createWorkspace(path: ws.path, name: "svc")
        let store = KeychainSecretStore(
            service: "dev.xrest.tests.\(UUID().uuidString)", indexURL: dir.appendingPathComponent("idx.json"))
        return try WorkspaceModel(url: ws, secrets: store)
    }

    func testChangingZoomDoesNotResetPanelStateOrEdits() throws {
        let model = try makeModel()
        model.openPanel(.environments)
        model.panelDirty = true
        model.newScratch()  // refused while a dirty panel is open, so nothing else changes either

        AppPreferences(defaults: defaults).zoom = 1.5
        ZoomStore.shared.refresh(from: defaults)
        model.applyPreferences(defaults: defaults)

        XCTAssertEqual(ZoomStore.shared.zoom, 1.5)
        XCTAssertEqual(model.panel, .environments, "zoom must not tear down the open panel")
        XCTAssertTrue(model.panelDirty, "zoom must not discard unsaved panel edits")
    }

    func testZoomStoreIsObservableAndFontsReadIt() {
        var changed = false
        withObservationTracking {
            _ = Font.mono(12)
        } onChange: {
            changed = true
        }
        AppPreferences(defaults: defaults).zoom = 1.25
        ZoomStore.shared.refresh(from: defaults)
        XCTAssertTrue(changed, "views using Font.mono must re-render when zoom changes")
    }

    func testRefreshWithSameZoomDoesNotInvalidate() {
        var changed = false
        withObservationTracking {
            _ = ZoomStore.shared.zoom
        } onChange: {
            changed = true
        }
        ZoomStore.shared.refresh(from: defaults)  // still 1.0
        XCTAssertFalse(changed)
    }

    func testEditorFontSizeIsDerivedFromZoom() {
        XCTAssertEqual(CodeEditor.fontSize(zoom: 1.0, base: 12), 12)
        XCTAssertEqual(CodeEditor.fontSize(zoom: 1.5, base: 12), 18)
        let tv = NSTextView()
        tv.string = "{\"a\": 1}"
        JSONHighlighter.apply(to: tv, zoom: 2.0)
        XCTAssertEqual((tv.textStorage?.attribute(.font, at: 0, effectiveRange: nil) as? NSFont)?.pointSize, 24)
        JSONHighlighter.apply(to: tv, zoom: 0.8)
        XCTAssertEqual((tv.textStorage?.attribute(.font, at: 0, effectiveRange: nil) as? NSFont)?.pointSize ?? 0, 9.6, accuracy: 0.01)
    }
}
