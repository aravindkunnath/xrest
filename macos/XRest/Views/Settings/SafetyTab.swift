import SwiftUI

struct SafetyTab: View {
    @AppStorage(AppPreferences.Key.confirmStyle) private var style = ConfirmStyle.countdown.rawValue
    @AppStorage(AppPreferences.Key.confirmCountdownSeconds) private var seconds = 3
    @AppStorage(AppPreferences.Key.returnCancels) private var returnCancels = true
    @AppStorage(AppPreferences.Key.warnUnresolved) private var warnUnresolved = true
    @State private var preview: PendingConfirmation?

    private var confirmStyle: ConfirmStyle { ConfirmStyle(rawValue: style) ?? .countdown }

    private static let hints: [ConfirmStyle: String] = [
        .countdown: "Send unlocks after a short delay.",
        .confirm: "Send is available immediately.",
        .typeName: "Slowest, for environments where mistakes are costly.",
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SettingsSectionHeader("Unsafe environments")
            SettingsCard {
                ForEach(ConfirmStyle.allCases) { option in
                    styleRow(option)
                }
                SettingsRow("Countdown length") {
                    SettingsStepper(
                        "Countdown length", value: $seconds,
                        range: AppPreferences.countdownRange, unit: "s")
                }
                .disabled(confirmStyle != .countdown)
                .opacity(confirmStyle == .countdown ? 1 : 0.5)
                SettingsRow(
                    "Return key cancels",
                    hint: UnsafeConfirmationPresentation.returnCancelsHint(returnCancels: returnCancels)
                ) {
                    SettingsSwitch("Return key cancels", isOn: $returnCancels)
                }
                SettingsRow("Preview the dialog with these settings") {
                    Button("Preview Dialog") { preview = samplePending() }
                }
            }

            SettingsSectionHeader("Any environment").padding(.top, 12)
            SettingsCard {
                SettingsRow(
                    "Warn about unresolved variables",
                    hint: "Ask before sending a request that still contains a missing variable."
                ) {
                    SettingsSwitch("Warn about unresolved variables", isOn: $warnUnresolved)
                }
            }
            Text("Which environments count as unsafe is set per workspace in Environments.")
                .font(.ui(11))
                .foregroundStyle(Theme.muted)
                .padding(.horizontal, 4)
        }
        .sheet(item: $preview) { pending in
            UnsafeConfirmationSheet(pending: pending) { if preview === pending { preview = nil } }
        }
    }

    private func styleRow(_ option: ConfirmStyle) -> some View {
        let selected = option == confirmStyle
        return Button {
            style = option.rawValue
        } label: {
            SettingsRow(option.title, hint: Self.hints[option]) {
                Image(systemName: selected ? "largecircle.fill.circle" : "circle")
                    .foregroundStyle(selected ? Theme.amber : Theme.controlOff)
                    .accessibilityHidden(true)
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(option.title)
        .accessibilityAddTraits(selected ? .isSelected : [])
    }

    /// The action is a no-op, so the preview can never send a real request.
    private func samplePending() -> PendingConfirmation {
        PendingConfirmation(
            environment: "PROD", method: "POST", url: "{{BASE_URL}}/v1/refunds",
            resolvedURL: "https://api.example.com/v1/refunds",
            countdown: ConfirmationCountdown(
                duration: ConfirmationPolicy.countdownDuration(style: confirmStyle, seconds: seconds)),
            style: confirmStyle, returnCancels: returnCancels, action: {})
    }
}
