import SwiftUI

struct VersionsTab: View {
    @AppStorage(AppPreferences.Key.maxVersions) private var maxVersions = AppPreferences.maxVersionsRange.upperBound

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SettingsSectionHeader("History")
            SettingsCard {
                SettingsRow("Versions per endpoint", hint: "Older versions are discarded beyond this limit.") {
                    SettingsStepper(
                        "Versions per endpoint", value: $maxVersions,
                        range: AppPreferences.maxVersionsRange, unit: "versions")
                }
            }
        }
    }
}
