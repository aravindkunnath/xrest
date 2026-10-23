import AppKit
import SwiftUI

/// Design tokens from the "A · Workbench" mockup. The app is dark-only.
enum Theme {
    static let bg = Color(hex: 0x0F1115)
    static let panel = Color(hex: 0x13161C)
    static let sidebar = Color(hex: 0x161920)
    static let raised = Color(hex: 0x1A1E26)
    static let selected = Color(hex: 0x232833)
    static let hover = Color(hex: 0x1E222A)
    static let response = Color(hex: 0x0C0E12)

    static let border = Color(hex: 0x262A33)
    static let borderStrong = Color(hex: 0x2E333D)
    static let rowDivider = Color(hex: 0x1E222A)

    static let text = Color(hex: 0xE6E8EC)
    static let textSoft = Color(hex: 0xC9CDD4)
    static let muted = Color(hex: 0x9AA1AD)

    static let amber = Color(hex: 0xF2A93B)
    static let amberSoft = Color(hex: 0xFFC56B)
    static let amberFill = Color(hex: 0x2A2217)
    static let amberBorder = Color(hex: 0x5A4520)
    static let green = Color(hex: 0x5EC28A)
    static let lightGreen = Color(hex: 0xA6DB8A)
    static let greenFill = Color(hex: 0x18201A)
    static let greenBorder = Color(hex: 0x3B4A2E)
    static let blue = Color(hex: 0x6CB6FF)
    static let red = Color(hex: 0xF07178)
    static let redFill = Color(hex: 0x241517)
    static let redBorder = Color(hex: 0x5C2A30)

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
