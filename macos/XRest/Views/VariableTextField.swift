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
    var textColor: Color = Theme.textSoft
    var accessibilityLabel: String? = nil
    var onSubmit: (() -> Void)? = nil

    @State private var trigger: CompletionTrigger?
    @State private var items: [String] = []
    @State private var selected = 0
    @State private var submitGate = SubmitGate()
    @FocusState private var focused: Bool

    private var isOpen: Bool { trigger != nil && !items.isEmpty }

    var body: some View {
        TextField(placeholder, text: $text)
            .textFieldStyle(.plain)
            .font(font)
            .foregroundStyle(textColor)
            .tint(Theme.amber)
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
            .overlay(alignment: .topLeading) {
                if isOpen { list.offset(y: 22) }
            }
            .zIndex(isOpen ? 10 : 0)
    }

    private var list: some View {
        VStack(alignment: .leading, spacing: 0) {
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
        .shadow(color: .black.opacity(0.4), radius: 8, y: 3)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Variable suggestions")
    }

    // MARK: Behavior

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
        if names.isEmpty { trigger = nil }
    }

    private func close() {
        trigger = nil
        items = []
        selected = 0
    }

    private func move(_ delta: Int) -> KeyPress.Result {
        guard isOpen else { return .ignored }
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
