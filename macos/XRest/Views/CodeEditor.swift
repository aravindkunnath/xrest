import AppKit
import SwiftUI

/// Plain monospaced text editor backed by `NSTextView`, with the smart substitutions that corrupt
/// JSON (curly quotes, dashes, auto-correct) turned off. Also used read-only for responses.
/// Text that looks like JSON is syntax highlighted.
struct CodeEditor: NSViewRepresentable {
    @Binding var text: String
    var isEditable = true

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
        JSONHighlighter.apply(to: textView)
        return scroll
    }

    func updateNSView(_ scroll: NSScrollView, context: Context) {
        context.coordinator.parent = self
        guard let textView = scroll.documentView as? NSTextView else { return }
        textView.isEditable = isEditable
        if textView.string != text {
            textView.string = text
            JSONHighlighter.apply(to: textView)
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    final class Coordinator: NSObject, NSTextViewDelegate {
        var parent: CodeEditor

        init(_ parent: CodeEditor) {
            self.parent = parent
        }

        func textDidChange(_ notification: Notification) {
            guard let textView = notification.object as? NSTextView else { return }
            JSONHighlighter.apply(to: textView)
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

    static func apply(to textView: NSTextView) {
        guard let storage = textView.textStorage else { return }
        let full = NSRange(location: 0, length: storage.length)
        let font = NSFont.monospacedSystemFont(ofSize: 12 * CGFloat(AppPreferences.currentZoom), weight: .regular)
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
