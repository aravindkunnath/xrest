import AppKit
import SwiftUI

extension View {
    /// Shows `content` in its own borderless window just under this view, so no ancestor
    /// (scroll view, clip, sibling stacking order) can cover or cut it off. It never takes focus.
    func floatingPopup<Content: View>(
        isPresented: Bool, x: CGFloat = 0, interactive: Bool = false,
        @ViewBuilder content: () -> Content
    ) -> some View {
        background(
            PopupAnchor(
                isPresented: isPresented, x: x, interactive: interactive,
                content: isPresented ? AnyView(content()) : AnyView(EmptyView())))
    }
}

private struct PopupAnchor: NSViewRepresentable {
    let isPresented: Bool
    let x: CGFloat
    let interactive: Bool
    let content: AnyView

    func makeCoordinator() -> Coordinator { Coordinator() }
    func makeNSView(context: Context) -> NSView { NSView() }

    func updateNSView(_ view: NSView, context: Context) {
        context.coordinator.update(
            anchor: view, isPresented: isPresented, x: x, interactive: interactive, content: content)
    }

    static func dismantleNSView(_ view: NSView, coordinator: Coordinator) {
        coordinator.hide()
    }

    typealias Coordinator = FloatingPopupController
}

/// One borderless, non-activating child window that shows SwiftUI content under a screen rect.
@MainActor final class FloatingPopupController {
    private var panel: NSPanel?
    private let host = NSHostingView(rootView: AnyView(EmptyView()))

    func update(anchor: NSView, isPresented: Bool, x: CGFloat, interactive: Bool, content: AnyView) {
        guard isPresented, let window = anchor.window else { return hide() }
        let field = window.convertToScreen(anchor.convert(anchor.bounds, to: nil))
        show(content, in: window, below: field, x: x, interactive: interactive)
    }

    /// `rect` is in screen coordinates; the popup sits just under it, `x` points to the right.
    func show(_ content: AnyView, in window: NSWindow, below rect: NSRect, x: CGFloat = 0, interactive: Bool = false) {
        host.rootView = AnyView(content.fixedSize())
        let size = host.fittingSize
        let origin = NSPoint(x: rect.minX + x, y: rect.minY - size.height - 2)

        let panel = self.panel ?? makePanel()
        panel.ignoresMouseEvents = !interactive
        panel.setFrame(NSRect(origin: origin, size: size), display: true)
        if panel.parent == nil { window.addChildWindow(panel, ordered: .above) }
        panel.orderFront(nil)
    }

    func hide() {
        guard let panel else { return }
        panel.parent?.removeChildWindow(panel)
        panel.orderOut(nil)
    }

    private func makePanel() -> NSPanel {
        let panel = NSPanel(
            contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered, defer: true)
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = true
        panel.level = .floating
        panel.contentView = host
        self.panel = panel
        return panel
    }
}
