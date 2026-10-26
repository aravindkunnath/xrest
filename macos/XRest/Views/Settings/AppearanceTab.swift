import SwiftUI

struct AppearanceTab: View {
    @AppStorage(AppPreferences.Key.appearance) private var appearance = AppearanceMode.system.rawValue
    @AppStorage(AppPreferences.Key.zoom) private var zoom = 1.0

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SettingsSectionHeader("Theme")
            SettingsCard {
                SettingsRow("Theme") { ThemeSwatchPicker(selection: $appearance) }
            }
            SettingsSectionHeader("Zoom").padding(.top, 14)
            SettingsCard {
                SettingsRow("Zoom", hint: "Scales code, request bodies and responses.") {
                    Picker("Zoom", selection: $zoom) {
                        ForEach(AppPreferences.zoomSteps, id: \.self) {
                            Text("\(Int(($0 * 100).rounded()))%").tag($0)
                        }
                    }
                    .labelsHidden()
                    .fixedSize()
                }
            }
        }
    }
}
