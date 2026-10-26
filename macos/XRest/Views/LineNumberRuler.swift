import AppKit

/// Vertical ruler that numbers the logical lines of an `NSTextView`. It only asks the layout
/// manager about the lines currently visible, so a huge unwrapped line is never laid out in full.
final class LineNumberRuler: NSRulerView {
    private weak var textView: NSTextView?
    private var lineStarts: [Int] = [0]
    var font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular) {
        didSet { updateThickness(); needsDisplay = true }
    }

    init(scrollView: NSScrollView, textView: NSTextView) {
        self.textView = textView
        super.init(scrollView: scrollView, orientation: .verticalRuler)
        reservedThicknessForMarkers = 0
        clientView = textView
        rebuild()
        let center = NotificationCenter.default
        center.addObserver(
            self, selector: #selector(textChanged), name: NSText.didChangeNotification, object: textView)
        scrollView.contentView.postsBoundsChangedNotifications = true
        center.addObserver(
            self, selector: #selector(scrolled), name: NSView.boundsDidChangeNotification,
            object: scrollView.contentView)
    }

    required init(coder: NSCoder) { fatalError("init(coder:) is not used") }

    deinit { NotificationCenter.default.removeObserver(self) }

    /// UTF-16 offsets where each line starts; always contains at least offset 0.
    static func lineStarts(in text: String) -> [Int] {
        var starts = [0]
        var offset = 0
        for unit in text.utf16 {
            offset += 1
            if unit == 0x0A { starts.append(offset) }
        }
        return starts
    }

    static func lineStarts(in text: NSString) -> [Int] {
        var starts = [0]
        let length = text.length
        var index = 0
        while index < length {
            let range = text.range(of: "\n", options: .literal, range: NSRange(location: index, length: length - index))
            if range.location == NSNotFound { break }
            index = NSMaxRange(range)
            starts.append(index)
        }
        return starts
    }

    static func lineCount(in text: String) -> Int { lineStarts(in: text).count }

    @objc private func textChanged(_ note: Notification) { textReplaced() }

    /// Also called after programmatic `string` assignment, which posts no change notification.
    func textReplaced() {
        rebuild()
        needsDisplay = true
    }

    @objc private func scrolled(_ note: Notification) { needsDisplay = true }

    private func rebuild() {
        let text = (textView?.textStorage?.string ?? "") as NSString
        lineStarts = Self.lineStarts(in: text)
        updateThickness()
    }

    private func updateThickness() {
        let digits = max(2, String(lineStarts.count).count)
        let width = ("0" as NSString).size(withAttributes: [.font: numberFont]).width
        let thickness = ceil(width * CGFloat(digits)) + 16
        if abs(ruleThickness - thickness) > 0.5 { ruleThickness = thickness }
    }

    private var numberFont: NSFont {
        NSFont.monospacedDigitSystemFont(ofSize: max(font.pointSize * 0.85, 8), weight: .regular)
    }

    override func drawHashMarksAndLabels(in rect: NSRect) {
        guard let textView, let layout = textView.layoutManager, let container = textView.textContainer
        else { return }
        let visible = textView.visibleRect
        let origin = textView.textContainerOrigin
        let attrs: [NSAttributedString.Key: Any] = [
            .font: numberFont, .foregroundColor: NSColor.dynamic(dark: 0x6B7280, light: 0x9AA0AA),
        ]
        let length = textView.textStorage?.length ?? 0

        func draw(number: Int, fragment: NSRect) {
            let y = convert(NSPoint(x: 0, y: fragment.minY + origin.y), from: textView).y
            let text = String(number) as NSString
            let size = text.size(withAttributes: attrs)
            let lineHeight = fragment.height
            // Center on the line fragment (it is taller than the glyphs because of the line spacing).
            let top = y + (isFlipped ? (lineHeight - size.height) / 2 : -(lineHeight + size.height) / 2)
            text.draw(at: NSPoint(x: bounds.width - size.width - 8, y: top), withAttributes: attrs)
        }

        if length == 0 {
            let fragment = layout.extraLineFragmentRect
            let height = fragment.isEmpty ? (font.ascender - font.descender) * 1.35 : fragment.height
            draw(number: 1, fragment: NSRect(x: 0, y: 0, width: 1, height: height))
            return
        }

        let glyphs = layout.glyphRange(
            forBoundingRect: NSRect(
                x: 0, y: visible.minY - origin.y, width: container.size.width, height: visible.height),
            in: container)
        let chars = layout.characterRange(forGlyphRange: glyphs, actualGlyphRange: nil)

        // First line start at or after the visible range's first character.
        var index = lineStarts.partitioningIndex { $0 >= chars.location }
        while index < lineStarts.count, lineStarts[index] <= NSMaxRange(chars) {
            let start = lineStarts[index]
            if start >= length {
                let fragment = layout.extraLineFragmentRect
                if !fragment.isEmpty { draw(number: index + 1, fragment: fragment) }
                break
            }
            let glyph = layout.glyphIndexForCharacter(at: start)
            let fragment = layout.lineFragmentRect(forGlyphAt: glyph, effectiveRange: nil)
            draw(number: index + 1, fragment: fragment)
            index += 1
        }
    }
}

private extension Array where Element == Int {
    /// Index of the first element satisfying `predicate`, for an array partitioned by it.
    func partitioningIndex(where predicate: (Int) -> Bool) -> Int {
        var low = 0
        var high = count
        while low < high {
            let mid = (low + high) / 2
            if predicate(self[mid]) { high = mid } else { low = mid + 1 }
        }
        return low
    }
}
