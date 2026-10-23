import AppKit
import SwiftUI

/// Design tokens from the "A · Workbench" mockup. Every token is a dark/light pair resolved from the window appearance (Settings: System/Light/Dark).
enum Theme {
    static let bg = Color(nsColor: .dynamic(dark: 0x0F1115, light: 0xF7F8FA))
    static let panel = Color(nsColor: .dynamic(dark: 0x13161C, light: 0xFFFFFF))
    static let sidebar = Color(nsColor: .dynamic(dark: 0x161920, light: 0xEFF1F5))
    static let raised = Color(nsColor: .dynamic(dark: 0x1A1E26, light: 0xF3F4F7))
    static let selected = Color(nsColor: .dynamic(dark: 0x232833, light: 0xDFE4EC))
    static let hover = Color(nsColor: .dynamic(dark: 0x1E222A, light: 0xE8EBF1))
    static let response = Color(nsColor: .dynamic(dark: 0x0C0E12, light: 0xFBFBFD))

    static let border = Color(nsColor: .dynamic(dark: 0x262A33, light: 0xD8DCE3))
    static let borderStrong = Color(nsColor: .dynamic(dark: 0x2E333D, light: 0xC4CAD4))
    static let rowDivider = Color(nsColor: .dynamic(dark: 0x1E222A, light: 0xE8EBF1))

    static let text = Color(nsColor: .dynamic(dark: 0xE6E8EC, light: 0x1B1E24))
    static let textSoft = Color(nsColor: .dynamic(dark: 0xC9CDD4, light: 0x3A404B))
    static let muted = Color(nsColor: .dynamic(dark: 0x9AA1AD, light: 0x626A78))

    static let amber = Color(nsColor: .dynamic(dark: 0xF2A93B, light: 0xB26A00))
    static let amberSoft = Color(nsColor: .dynamic(dark: 0xFFC56B, light: 0x935700))
    static let amberFill = Color(nsColor: .dynamic(dark: 0x2A2217, light: 0xFFF1D6))
    static let amberBorder = Color(nsColor: .dynamic(dark: 0x5A4520, light: 0xE6C27A))
    static let green = Color(nsColor: .dynamic(dark: 0x5EC28A, light: 0x1D8750))
    static let lightGreen = Color(nsColor: .dynamic(dark: 0xA6DB8A, light: 0x3B8A27))
    static let greenFill = Color(nsColor: .dynamic(dark: 0x18201A, light: 0xE4F4EA))
    static let greenBorder = Color(nsColor: .dynamic(dark: 0x3B4A2E, light: 0xA6D4B8))
    static let blue = Color(nsColor: .dynamic(dark: 0x6CB6FF, light: 0x1C6AC0))
    static let red = Color(nsColor: .dynamic(dark: 0xF07178, light: 0xC9313D))
    static let redFill = Color(nsColor: .dynamic(dark: 0x241517, light: 0xFCE9EB))
    static let redBorder = Color(nsColor: .dynamic(dark: 0x5C2A30, light: 0xEDB0B6))

    /// Disabled-toggle track and similar neutral controls.
    static let controlOff = Color(nsColor: .dynamic(dark: 0x3A3F4A, light: 0xB9BFCA))
    static let controlOffStrong = Color(nsColor: .dynamic(dark: 0x343A45, light: 0xAAB1BD))

    static func methodColor(_ method: String) -> Color {
        switch method.uppercased() {
        case "GET": return green
        case "POST": return amber
        case "PUT", "PATCH": return blue
        case "DELETE": return red
        default: return muted
        }
    }

    static func statusColor(_ status: UInt16) -> Color {
        switch StatusTone.of(status) {
        case .success: return green
        case .redirect: return amber
        case .error: return red
        }
    }
}

extension Color {
    init(hex: UInt32) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255)
    }
}

extension NSColor {
    convenience init(hex: UInt32) {
        self.init(
            srgbRed: CGFloat((hex >> 16) & 0xFF) / 255,
            green: CGFloat((hex >> 8) & 0xFF) / 255,
            blue: CGFloat(hex & 0xFF) / 255, alpha: 1)
    }
}

extension Font {
    /// JetBrains Mono when installed, otherwise the system monospaced face.
    static func mono(_ size: CGFloat, weight: Font.Weight = .regular) -> Font {
        // Zoom (Settings) scales code and body text; chrome stays at its designed size.
        // Read from the observable store so views using this font re-render on a zoom change.
        let size = size * CGFloat(ZoomStore.shared.zoom)
        if NSFont(name: "JetBrains Mono", size: size) != nil {
            return .custom("JetBrains Mono", size: size).weight(weight)
        }
        return .system(size: size, weight: weight, design: .monospaced)
    }

    /// Inter when installed, otherwise the system face.
    static func ui(_ size: CGFloat = 13, weight: Font.Weight = .regular) -> Font {
        if NSFont(name: "Inter", size: size) != nil {
            return .custom("Inter", size: size).weight(weight)
        }
        return .system(size: size, weight: weight)
    }
}

extension View {
    /// Rounded, bordered container used for pills and inputs.
    func chrome(
        fill: Color = Theme.raised, stroke: Color = Theme.border, radius: CGFloat = 6
    ) -> some View {
        background(RoundedRectangle(cornerRadius: radius).fill(fill))
            .overlay(RoundedRectangle(cornerRadius: radius).stroke(stroke, lineWidth: 1))
    }

    func hairline(_ edge: Edge, color: Color = Theme.border) -> some View {
        overlay(alignment: Alignment(edge)) {
            Rectangle().fill(color).frame(
                width: edge == .leading || edge == .trailing ? 1 : nil,
                height: edge == .top || edge == .bottom ? 1 : nil)
        }
    }
}

extension Alignment {
    fileprivate init(_ edge: Edge) {
        switch edge {
        case .top: self = .top
        case .bottom: self = .bottom
        case .leading: self = .leading
        case .trailing: self = .trailing
        }
    }
}

/// Plain button with a hover highlight; used for icon buttons and list rows.
struct HoverButtonStyle: ButtonStyle {
    var fill: Color = Theme.hover
    var radius: CGFloat = 6
    @State private var hovering = false

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .background(
                RoundedRectangle(cornerRadius: radius)
                    .fill(hovering || configuration.isPressed ? fill : .clear)
            )
            .onHover { hovering = $0 }
    }
}

/// Input styling from the mockup; apply to a `TextField` with `.textFieldStyle(.plain)`.
struct ThemedField: ViewModifier {
    var mono = false

    func body(content: Content) -> some View {
        content
            .textFieldStyle(.plain)
            .font(mono ? .mono(12) : .ui(12))
            .foregroundStyle(Theme.text)
            .padding(.horizontal, 8)
            .frame(height: 26)
            .chrome(fill: Theme.sidebar, stroke: Theme.borderStrong, radius: 5)
    }
}

extension View {
    func themedField(mono: Bool = false) -> some View {
        modifier(ThemedField(mono: mono))
    }
}

/// Underlined tab strip (request sections, response modes).
struct UnderlineTabs<Tab: Hashable>: View {
    let tabs: [Tab]
    @Binding var selection: Tab
    let label: (Tab) -> AnyView

    var body: some View {
        HStack(spacing: 18) {
            ForEach(tabs, id: \.self) { tab in
                Button {
                    selection = tab
                } label: {
                    label(tab)
                        .font(.ui(13))
                        .lineLimit(1)
                        .fixedSize()
                        .foregroundStyle(tab == selection ? Theme.text : Theme.muted)
                        .padding(.horizontal, 2)
                        .frame(maxHeight: .infinity)
                        .overlay(alignment: .bottom) {
                            Rectangle()
                                .fill(tab == selection ? Theme.amber : .clear)
                                .frame(height: 2)
                        }
                }
                .buttonStyle(.plain)
                .accessibilityAddTraits(tab == selection ? .isSelected : [])
            }
        }
        .padding(.horizontal, 16)
        .frame(height: 36)
        .hairline(.bottom)
    }
}

/// Small bordered tag, e.g. "scratch" or "local · not in git".
struct Tag: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.ui(10))
            .foregroundStyle(Theme.muted)
            .padding(.horizontal, 5)
            .padding(.vertical, 1)
            .overlay(RoundedRectangle(cornerRadius: 4).stroke(Theme.borderStrong, lineWidth: 1))
    }
}

extension NSColor {
    /// A color that resolves per appearance (dark / light) at draw time.
    static func dynamic(dark: UInt32, light: UInt32) -> NSColor {
        NSColor(name: nil) { appearance in
            appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
                ? NSColor(hex: dark) : NSColor(hex: light)
        }
    }
}
