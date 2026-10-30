import AppKit
import SwiftUI

/// Plain monospaced text editor backed by `NSTextView`, with the smart substitutions that corrupt
/// JSON (curly quotes, dashes, auto-correct) turned off. Also used read-only for responses.
/// Text that looks like JSON is syntax highlighted.
struct CodeEditor: NSViewRepresentable {
    @Binding var text: String
    var isEditable = true
    /// Read from the observable store while the parent builds this view, so a zoom change makes
    /// SwiftUI call `updateNSView`.
    var zoom = ZoomStore.shared.zoom
    /// Editor preferences, read from the store the same way as `zoom`.
    var fontFamily = PreferencesStore.shared.editorFontFamily
    var baseFontSize = PreferencesStore.shared.editorFontSize
    var wrapLines = PreferencesStore.shared.wrapLines
    /// When non-nil, overrides the wrap preference (e.g. hex dumps must not wrap).
    var wrapOverride: Bool? = nil
    var showLineNumbers = PreferencesStore.shared.showLineNumbers
    /// When set, hovering a `{{name}}` shows its value in a small popup.
    var variableLookup: ((String) -> VariableInfo)? = nil

    static func fontSize(zoom: Double, base: Int) -> CGFloat { CGFloat(base) * CGFloat(zoom) }

    /// The named family when it is installed and fixed-pitch, else the system monospaced face.
    static func font(family: String, size: CGFloat) -> NSFont {
        if !family.isEmpty, let named = NSFont(name: family, size: size), named.isFixedPitch {
            return named
        }
        if !family.isEmpty,
            let regular = NSFontManager.shared.font(withFamily: family, traits: [], weight: 5, size: size),
            regular.isFixedPitch
        {
            return regular
        }
        return NSFont.monospacedSystemFont(ofSize: size, weight: .regular)
    }

    /// `NSTextView` keeps a tracking container narrower than the view by the horizontal insets.
    static func wrapContainerWidth(contentWidth: CGFloat, inset: CGFloat) -> CGFloat {
        max(0, contentWidth - 2 * inset)
    }

    private var editorFont: NSFont {
        Self.font(family: fontFamily, size: Self.fontSize(zoom: zoom, base: baseFontSize))
    }

    private func highlight(_ textView: NSTextView) {
        JSONHighlighter.apply(to: textView, zoom: zoom, family: fontFamily, base: baseFontSize)
    }

    private func applyLayout(to scroll: NSScrollView, textView: NSTextView, coordinator: Coordinator) {
        guard let container = textView.textContainer else { return }
        // Geometry is only touched when wrapping flips: resetting it on every update would
        // invalidate the layout of the whole document.
        let wrap = wrapOverride ?? wrapLines
        if coordinator.appliedWrap != wrap {
            coordinator.appliedWrap = wrap
            if wrap {
                textView.isHorizontallyResizable = false
                container.widthTracksTextView = true
                let width = scroll.contentSize.width
                if width > 0 {
                    container.size = NSSize(
                        width: Self.wrapContainerWidth(contentWidth: width, inset: textView.textContainerInset.width),
                        height: CGFloat.greatestFiniteMagnitude)
                    textView.setFrameSize(NSSize(width: width, height: textView.frame.height))
                }
                scroll.hasHorizontalScroller = false
            } else {
                textView.isHorizontallyResizable = true
                container.widthTracksTextView = false
                container.size = NSSize(width: 1e7, height: CGFloat.greatestFiniteMagnitude)
                scroll.hasHorizontalScroller = true
            }
        }
        scroll.rulersVisible = showLineNumbers
        (scroll.verticalRulerView as? LineNumberRuler)?.font = editorFont
        textView.needsDisplay = true
    }

    func makeNSView(context: Context) -> NSScrollView {
        let scroll = NSTextView.scrollableTextView()
        scroll.hasVerticalScroller = true
        scroll.drawsBackground = false
        guard let textView = scroll.documentView as? NSTextView else { return scroll }
        textView.delegate = context.coordinator
        textView.isRichText = false
        textView.allowsUndo = true
        textView.isAutomaticQuoteSubstitutionEnabled = false
        textView.isAutomaticDashSubstitutionEnabled = false
        textView.isAutomaticTextReplacementEnabled = false
        textView.isAutomaticSpellingCorrectionEnabled = false
        textView.isAutomaticLinkDetectionEnabled = false
        textView.isContinuousSpellCheckingEnabled = false
        textView.drawsBackground = false
        textView.insertionPointColor = NSColor.dynamic(dark: 0xF2A93B, light: 0xB26A00)
        textView.selectedTextAttributes = [.backgroundColor: NSColor.dynamic(dark: 0x3A3F4A, light: 0xBBD3F2)]
        textView.textContainerInset = NSSize(width: 10, height: 12)
        textView.isEditable = isEditable
        textView.usesFindBar = true
        textView.isIncrementalSearchingEnabled = true
        textView.string = text
        highlight(textView)
        let ruler = LineNumberRuler(scrollView: scroll, textView: textView)
        scroll.hasVerticalRuler = true
        scroll.verticalRulerView = ruler
        applyLayout(to: scroll, textView: textView, coordinator: context.coordinator)
        if variableLookup != nil {
            context.coordinator.textView = textView
            textView.addTrackingArea(
                NSTrackingArea(
                    rect: .zero, options: [.mouseMoved, .mouseEnteredAndExited, .activeInKeyWindow, .inVisibleRect],
                    owner: context.coordinator, userInfo: nil))
        }
        return scroll
    }

    func updateNSView(_ scroll: NSScrollView, context: Context) {
        context.coordinator.parent = self
        guard let textView = scroll.documentView as? NSTextView else { return }
        textView.isEditable = isEditable
        if textView.string != text {
            textView.string = text
            highlight(textView)
            (scroll.verticalRulerView as? LineNumberRuler)?.textReplaced()
        } else if textView.font != editorFont {
            highlight(textView)
        }
        applyLayout(to: scroll, textView: textView, coordinator: context.coordinator)
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    @MainActor final class Coordinator: NSObject, NSTextViewDelegate {
        var parent: CodeEditor
        var appliedWrap: Bool?

        weak var textView: NSTextView?
        private lazy var popup = FloatingPopupController()

        init(_ parent: CodeEditor) {
            self.parent = parent
        }

        @objc func mouseMoved(with event: NSEvent) {
            guard let textView, let lookup = parent.variableLookup, let layout = textView.layoutManager,
                let container = textView.textContainer, let window = textView.window
            else { return }
            let point = textView.convert(event.locationInWindow, from: nil)
            let inset = textView.textContainerOrigin
            let local = NSPoint(x: point.x - inset.x, y: point.y - inset.y)
            let glyphBox = layout.usedRect(for: container)
            guard glyphBox.insetBy(dx: -4, dy: -4).contains(local) else { return popup.hide() }
            let index = layout.characterIndex(
                for: local, in: container, fractionOfDistanceBetweenInsertionPoints: nil)
            guard let token = VariableToken.name(at: index, in: textView.string) else { return popup.hide() }

            let glyphs = layout.glyphRange(forCharacterRange: token.range, actualCharacterRange: nil)
            var box = layout.boundingRect(forGlyphRange: glyphs, in: container)
            box.origin.x += inset.x
            box.origin.y += inset.y
            let screen = window.convertToScreen(textView.convert(box, to: nil))
            let info = lookup(token.name)
            popup.show(AnyView(VariableChip(name: token.name, info: info)), in: window, below: screen)
        }

        @objc func mouseExited(with event: NSEvent) { popup.hide() }

        func textDidChange(_ notification: Notification) {
            guard let textView = notification.object as? NSTextView else { return }
            parent.highlight(textView)
            if parent.text != textView.string {
                parent.text = textView.string
            }
        }
    }
}

/// Colors the tokens of JSON text: keys blue, strings green, numbers and literals amber.
enum JSONHighlighter {
    private static let token = try! NSRegularExpression(
        pattern:
            #"("(?:\\.|[^"\\])*")(\s*:)?|(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)|\b(true|false|null)\b"#)

    static func apply(
        to textView: NSTextView, zoom: Double = ZoomStore.shared.zoom,
        family: String = PreferencesStore.shared.editorFontFamily,
        base: Int = PreferencesStore.shared.editorFontSize
    ) {
        guard let storage = textView.textStorage else { return }
        let full = NSRange(location: 0, length: storage.length)
        let font = CodeEditor.font(family: family, size: CodeEditor.fontSize(zoom: zoom, base: base))
        let paragraph = NSMutableParagraphStyle()
        paragraph.lineHeightMultiple = 1.35

        storage.beginEditing()
        storage.setAttributes(
            [
                .font: font, .foregroundColor: NSColor.dynamic(dark: 0xC9CDD4, light: 0x3A404B),
                .paragraphStyle: paragraph,
            ], range: full)
        textView.typingAttributes = [
            .font: font, .foregroundColor: NSColor.dynamic(dark: 0xC9CDD4, light: 0x3A404B), .paragraphStyle: paragraph,
        ]

        let string = storage.string
        // Skip token coloring on big bodies so the main thread never stalls.
        if storage.length <= 256 * 1024, let first = string.first(where: { !$0.isWhitespace }), first == "{" || first == "[" {
            token.enumerateMatches(in: string, range: full) { match, _, _ in
                guard let match else { return }
                if match.range(at: 1).location != NSNotFound {
                    let isKey = match.range(at: 2).location != NSNotFound
                    storage.addAttribute(
                        .foregroundColor, value: (isKey ? NSColor.dynamic(dark: 0x6CB6FF, light: 0x1C6AC0) : NSColor.dynamic(dark: 0xA6DB8A, light: 0x3B8A27)),
                        range: match.range(at: 1))
                } else {
                    let group = match.range(at: 3).location != NSNotFound ? 3 : 4
                    storage.addAttribute(
                        .foregroundColor, value: NSColor.dynamic(dark: 0xF2A93B, light: 0xB26A00),
                        range: match.range(at: group))
                }
            }
        }
        storage.endEditing()
    }
}
