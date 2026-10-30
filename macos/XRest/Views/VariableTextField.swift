import AppKit
import SwiftUI

/// Names offered after typing `{{`, given what was typed so far.
typealias CompletionProvider = (_ prefix: String) -> [String]

/// A single-line text field that opens a completion list when the caret follows an unfinished
/// `{{`. Detection and insertion live in `CompletionTrigger`; this view only wires up the keys.
/// Return or Tab inserts the highlighted name; Up/Down move; Escape closes.
struct VariableTextField: View {
    let placeholder: String
    @Binding var text: String
    var completions: CompletionProvider?
    var font: Font = .mono(12)
    /// Point size behind `font`, used to place the hover popup over the right characters.
    var fontSize: CGFloat = 12
    var textColor: Color = Theme.textSoft
    var accessibilityLabel: String? = nil
    /// Tints `{{…}}` tokens by drawing a styled copy of the text under transparent field text.
    /// Off for callers that draw their own copy (the URL bar).
    var highlightsVariables = true
    var onSubmit: (() -> Void)? = nil

    @State private var trigger: CompletionTrigger?
    @State private var items: [String] = []
    @State private var selected = 0
    @State private var submitGate = SubmitGate()
    @State private var hovered: (info: VariableInfo, name: String, x: CGFloat)?
    @FocusState private var focused: Bool
    @Environment(\.variableLookup) private var lookup

    private var isOpen: Bool { trigger != nil }

    var body: some View {
        ZStack(alignment: .leading) {
            if highlightsVariables {
                Text(VariableStyle.attributed(text, color: textColor))
                    .font(font)
                    .lineLimit(1)
                    .truncationMode(.tail)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .allowsHitTesting(false)
                    .accessibilityHidden(true)
            }
            field
        }
    }

    private var field: some View {
        TextField(placeholder, text: $text)
            .textFieldStyle(.plain)
            .font(font)
            .foregroundStyle(highlightsVariables ? Color.clear : textColor)
            .tint(Theme.accent)
            .focused($focused)
            .onSubmit { if submitGate.shouldSubmit() { onSubmit?() } }
            .onChange(of: text) { refresh() }
            .onChange(of: focused) { if !focused { close() } }
            .onKeyPress(.downArrow) { move(1) }
            .onKeyPress(.upArrow) { move(-1) }
            .onKeyPress(.return) { accept() }
            .onKeyPress(.tab) { accept() }
            .onKeyPress(.escape) {
                guard isOpen else { return .ignored }
                close()
                return .handled
            }
            .accessibilityLabel(accessibilityLabel ?? placeholder)
            .onContinuousHover { phase in hover(phase) }
            .floatingPopup(
                isPresented: isOpen || hovered != nil, x: isOpen ? 0 : (hovered?.x ?? 0),
                interactive: isOpen
            ) {
                if isOpen {
                    list
                } else if let hovered {
                    VariableChip(name: hovered.name, info: hovered.info)
                }
            }
    }

    private var list: some View {
        VStack(alignment: .leading, spacing: 0) {
            if items.isEmpty {
                Text("No variables or secrets in this environment")
                    .font(.mono(12))
                    .foregroundStyle(Theme.muted)
                    .padding(.horizontal, 8)
                    .frame(minHeight: 24, alignment: .leading)
            }
            ForEach(Array(items.prefix(8).enumerated()), id: \.offset) { index, name in
                Button {
                    insert(name)
                } label: {
                    Text(name)
                        .font(.mono(12))
                        .foregroundStyle(name.hasPrefix("secret.") ? Theme.green : Theme.amberSoft)
                        .lineLimit(1)
                        .padding(.horizontal, 8)
                        .frame(minWidth: 220, maxWidth: .infinity, minHeight: 24, alignment: .leading)
                        .background(index == selected ? Theme.selected : .clear)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
        }
        .fixedSize()
        .padding(.vertical, 3)
        .chrome(fill: Theme.raised, stroke: Theme.borderStrong, radius: 6)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Variable suggestions")
    }

    // MARK: Behavior

    /// Width of one character of the field's monospaced face.
    private var charWidth: CGFloat {
        let points = fontSize * CGFloat(ZoomStore.shared.zoom)
        let face = NSFont(name: "JetBrains Mono", size: points)
            ?? NSFont.monospacedSystemFont(ofSize: points, weight: .regular)
        return face.maximumAdvancement.width
    }

    private func hover(_ phase: HoverPhase) {
        guard case .active(let point) = phase, !isOpen, let lookup else {
            hovered = nil
            return
        }
        let offset = VariableToken.offset(atX: point.x, charWidth: charWidth)
        guard let token = VariableToken.name(at: offset, in: text) else {
            hovered = nil
            return
        }
        hovered = (lookup(token.name), token.name, CGFloat(token.range.location) * charWidth)
    }

    /// UTF-16 caret of the active field editor; end of text when it cannot be read.
    private func caret() -> Int {
        if let editor = NSApp.keyWindow?.firstResponder as? NSTextView {
            return editor.selectedRange().location
        }
        return (text as NSString).length
    }

    private func refresh() {
        guard focused, let completions,
            let found = CompletionTrigger.detect(in: text, caret: caret())
        else { return close() }
        let names = completions(found.prefix)
        trigger = found
        items = names
        selected = min(selected, max(names.count - 1, 0))
    }

    private func close() {
        trigger = nil
        items = []
        selected = 0
    }

    private func move(_ delta: Int) -> KeyPress.Result {
        guard isOpen, !items.isEmpty else { return .ignored }
        let count = min(items.count, 8)
        selected = (selected + delta + count) % count
        return .handled
    }

    private func accept() -> KeyPress.Result {
        guard isOpen, items.indices.contains(selected) else { return .ignored }
        insert(items[selected])
        // Return also fires `onSubmit`; swallow that one, and drop the flag if it never comes.
        submitGate.markAccepted()
        DispatchQueue.main.async { submitGate.clear() }
        return .handled
    }

    private func insert(_ name: String) {
        guard let trigger else { return }
        let result = trigger.insert(name, into: text)
        close()
        text = result.text
        // The binding update resets the field editor; put the caret after the inserted braces.
        DispatchQueue.main.async {
            if let editor = NSApp.keyWindow?.firstResponder as? NSTextView {
                editor.setSelectedRange(NSRange(location: result.caret, length: 0))
            }
        }
    }
}

enum VariableStyle {
    /// `text` in `color`, with each `{{…}}` token amber on a soft amber fill.
    static func attributed(_ text: String, color: Color) -> AttributedString {
        var result = AttributedString(text.isEmpty ? " " : text)
        result.foregroundColor = color
        guard let regex = try? NSRegularExpression(pattern: #"\{\{[^}]*\}\}"#) else { return result }
        for match in regex.matches(in: text, range: NSRange(location: 0, length: (text as NSString).length)) {
            guard let range = Range(match.range, in: text),
                let lower = AttributedString.Index(range.lowerBound, within: result),
                let upper = AttributedString.Index(range.upperBound, within: result)
            else { continue }
            result[lower..<upper].foregroundColor = Theme.amber
            result[lower..<upper].backgroundColor = Theme.amberFill
        }
        return result
    }
}

/// The hover popup: a variable's name and what it resolves to (secrets masked).
struct VariableChip: View {
    let name: String
    let info: VariableInfo

    var body: some View {
        HStack(spacing: 6) {
            Text(name).foregroundStyle(name.hasPrefix("secret.") ? Theme.green : Theme.amberSoft)
            Text(info.isResolved ? "=" : "—").foregroundStyle(Theme.muted)
            Text(info.text).foregroundStyle(info.isResolved ? Theme.text : Theme.amber)
        }
        .font(.mono(12))
        .lineLimit(1)
        .fixedSize()
        .padding(.horizontal, 8)
        .frame(minHeight: 24)
        .chrome(fill: Theme.raised, stroke: Theme.borderStrong, radius: 6)
        .accessibilityLabel("\(name) \(info.text)")
    }
}
