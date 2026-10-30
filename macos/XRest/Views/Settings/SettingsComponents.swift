import SwiftUI

/// Rounded card whose `SettingsRow`s are separated by hairlines.
struct SettingsCard<Content: View>: View {
    @ViewBuilder var content: Content

    var body: some View {
        VStack(spacing: 0) {
            content
        }
        // Each row draws a bottom hairline; pulling the content up by 1pt pushes the last one under the clip.
        .padding(.bottom, -1)
        .frame(maxWidth: .infinity)
        .background(Theme.panel)
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .overlay(RoundedRectangle(cornerRadius: 10).stroke(Theme.border, lineWidth: 1))
    }
}

struct SettingsRow<Trailing: View>: View {
    let title: String
    var hint: String?
    @ViewBuilder var trailing: Trailing

    init(_ title: String, hint: String? = nil, @ViewBuilder trailing: () -> Trailing) {
        self.title = title
        self.hint = hint
        self.trailing = trailing()
    }

    var body: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.ui(13))
                    .foregroundStyle(Theme.text)
                if let hint {
                    Text(hint)
                        .font(.ui(11))
                        .foregroundStyle(Theme.muted)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            Spacer(minLength: 12)
            trailing
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .frame(minHeight: 44)
        .hairline(.bottom, color: Theme.rowDivider)
    }
}

struct SettingsSectionHeader: View {
    let title: String

    init(_ title: String) { self.title = title }

    var body: some View {
        Text(title.uppercased())
            .font(.ui(11, weight: .medium))
            .tracking(0.6)
            .foregroundStyle(Theme.muted)
            .padding(.horizontal, 4)
            .accessibilityAddTraits(.isHeader)
    }
}

struct SettingsSwitch: View {
    let label: String
    @Binding var isOn: Bool

    init(_ label: String, isOn: Binding<Bool>) {
        self.label = label
        self._isOn = isOn
    }

    var body: some View {
        Toggle(label, isOn: $isOn)
            .labelsHidden()
            .toggleStyle(SwitchToggleStyle())
            .accessibilityLabel(label)
    }
}

private struct SwitchToggleStyle: ToggleStyle {
    func makeBody(configuration: Configuration) -> some View {
        Button {
            withAnimation(.easeOut(duration: 0.12)) { configuration.isOn.toggle() }
        } label: {
            ZStack(alignment: configuration.isOn ? .trailing : .leading) {
                Capsule().fill(configuration.isOn ? Theme.accent : Theme.controlOff)
                Circle()
                    .fill(Color.white)
                    .frame(width: 18, height: 18)
                    .shadow(color: .black.opacity(0.25), radius: 1, y: 1)
                    .padding(2)
            }
            .frame(width: 38, height: 22)
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(.isToggle)
        .accessibilityValue(configuration.isOn ? "On" : "Off")
    }
}

struct SettingsStepper: View {
    let label: String
    @Binding var value: Int
    let range: ClosedRange<Int>
    let unit: String

    init(_ label: String, value: Binding<Int>, range: ClosedRange<Int>, unit: String) {
        self.label = label
        self._value = value
        self.range = range
        self.unit = unit
    }

    var body: some View {
        HStack(spacing: 0) {
            stepButton("minus", "Decrease \(label)", enabled: value > range.lowerBound) {
                value = max(range.lowerBound, value - 1)
            }
            Text("\(value) \(unit)")
                .font(.mono(12))
                .foregroundStyle(Theme.text)
                .monospacedDigit()
                .frame(minWidth: 56)
                .padding(.horizontal, 6)
                .accessibilityHidden(true)
            stepButton("plus", "Increase \(label)", enabled: value < range.upperBound) {
                value = min(range.upperBound, value + 1)
            }
        }
        .chrome(fill: Theme.sidebar, stroke: Theme.borderStrong, radius: 6)
        .accessibilityElement(children: .contain)
        .accessibilityLabel(label)
        .accessibilityValue("\(value) \(unit)")
    }

    private func stepButton(
        _ symbol: String, _ axLabel: String, enabled: Bool, action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: symbol)
                .font(.system(size: 10, weight: .semibold))
                .foregroundStyle(enabled ? Theme.textSoft : Theme.controlOff)
                .frame(width: 26, height: 24)
                .contentShape(Rectangle())
                .accessibilityHidden(true)
        }
        .buttonStyle(HoverButtonStyle(radius: 5))
        .disabled(!enabled)
        .accessibilityLabel(axLabel)
    }
}

struct SettingsSegmented<Value: Hashable>: View {
    let label: String
    @Binding var selection: Value
    let options: [(Value, String)]

    init(_ label: String, selection: Binding<Value>, options: [(Value, String)]) {
        self.label = label
        self._selection = selection
        self.options = options
    }

    var body: some View {
        HStack(spacing: 2) {
            ForEach(options.indices, id: \.self) { i in
                let (value, title) = options[i]
                let selected = value == selection
                Button {
                    selection = value
                } label: {
                    Text(title)
                        .font(.ui(12, weight: selected ? .medium : .regular))
                        .foregroundStyle(selected ? Theme.text : Theme.muted)
                        .padding(.horizontal, 12)
                        .frame(height: 22)
                        .background(
                            RoundedRectangle(cornerRadius: 5)
                                .fill(selected ? Theme.selected : .clear))
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityLabel(title)
                .accessibilityAddTraits(selected ? .isSelected : [])
            }
        }
        .padding(2)
        .chrome(fill: Theme.sidebar, stroke: Theme.borderStrong, radius: 7)
        .accessibilityElement(children: .contain)
        .accessibilityLabel(label)
    }
}

/// A row of color dots bound to `AccentChoice` raw values.
struct AccentPicker: View {
    @Binding var selection: String

    var body: some View {
        HStack(spacing: 8) {
            ForEach(AccentChoice.allCases) { choice in
                let selected = selection == choice.rawValue
                Button {
                    selection = choice.rawValue
                } label: {
                    Circle()
                        .fill(Color(nsColor: choice.nsColor))
                        .frame(width: 16, height: 16)
                        .overlay {
                            if choice == .system {
                                Circle().trim(from: 0, to: 0.5).stroke(Theme.bg, lineWidth: 2).rotationEffect(.degrees(90))
                            }
                        }
                        .padding(3)
                        .overlay(Circle().stroke(selected ? Theme.text : .clear, lineWidth: 1.5))
                        .contentShape(Circle())
                }
                .buttonStyle(.plain)
                .help(choice.title)
                .accessibilityLabel("\(choice.title) accent")
                .accessibilityAddTraits(selected ? .isSelected : [])
            }
        }
    }
}

/// Three 84x56 previews bound to `AppearanceMode` raw values.
struct ThemeSwatchPicker: View {
    @Binding var selection: String

    var body: some View {
        HStack(spacing: 12) {
            ForEach(AppearanceMode.allCases) { mode in
                let selected = selection == mode.rawValue
                Button {
                    selection = mode.rawValue
                } label: {
                    VStack(spacing: 6) {
                        preview(mode)
                            .frame(width: 84, height: 56)
                            .clipShape(RoundedRectangle(cornerRadius: 8))
                            .overlay(
                                RoundedRectangle(cornerRadius: 8)
                                    .stroke(
                                        selected ? Theme.accent : Theme.border,
                                        lineWidth: selected ? 2 : 1)
                            )
                            .accessibilityHidden(true)
                        Text(mode.title)
                            .font(.ui(12))
                            .foregroundStyle(selected ? Theme.text : Theme.muted)
                    }
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityLabel("\(mode.title) appearance")
                .accessibilityAddTraits(selected ? .isSelected : [])
            }
        }
    }

    @ViewBuilder
    private func preview(_ mode: AppearanceMode) -> some View {
        switch mode {
        case .light: miniWindow(dark: false)
        case .dark: miniWindow(dark: true)
        case .system:
            HStack(spacing: 0) {
                miniWindow(dark: false).frame(width: 42, alignment: .leading).clipped()
                miniWindow(dark: true).frame(width: 42, alignment: .trailing).clipped()
            }
        }
    }

    /// Fixed palettes: a preview shows the target theme regardless of the current appearance.
    private func miniWindow(dark: Bool) -> some View {
        let bg = dark ? Color(hex: 0x0F1115) : Color(hex: 0xF7F8FA)
        let side = dark ? Color(hex: 0x161920) : Color(hex: 0xEFF1F5)
        let line = dark ? Color(hex: 0x2E333D) : Color(hex: 0xC4CAD4)
        return ZStack(alignment: .topLeading) {
            bg
            HStack(spacing: 0) {
                side.frame(width: 22)
                VStack(alignment: .leading, spacing: 5) {
                    Capsule().fill(Theme.accent).frame(width: 28, height: 4)
                    Capsule().fill(line).frame(width: 36, height: 4)
                    Capsule().fill(line).frame(width: 24, height: 4)
                }
                .padding(8)
            }
        }
        .frame(width: 84, height: 56, alignment: .topLeading)
    }
}

#Preview("Settings components") {
    struct Demo: View {
        @State private var on = true
        @State private var size = 13
        @State private var layout = "split"
        @State private var theme = "system"

        var body: some View {
            VStack(alignment: .leading, spacing: 8) {
                SettingsSectionHeader("Appearance")
                SettingsCard {
                    SettingsRow("Theme") { ThemeSwatchPicker(selection: $theme) }
                    SettingsRow("Font size", hint: "Editor and response body") {
                        SettingsStepper("Font size", value: $size, range: 9...24, unit: "pt")
                    }
                    SettingsRow("Layout") {
                        SettingsSegmented(
                            "Layout", selection: $layout,
                            options: [("split", "Split"), ("stack", "Stacked")])
                    }
                    SettingsRow("Word wrap") { SettingsSwitch("Word wrap", isOn: $on) }
                }
            }
            .padding(20)
            .frame(width: 520)
            .background(Theme.bg)
        }
    }
    return Demo()
}
