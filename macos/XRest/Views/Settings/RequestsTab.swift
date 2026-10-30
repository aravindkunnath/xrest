import SwiftUI

struct RequestsTab: View {
    @AppStorage(AppPreferences.Key.timeoutSeconds) private var timeout = AppPreferences.defaultTimeoutSeconds
    @AppStorage(AppPreferences.Key.followRedirects) private var followRedirects = true
    @AppStorage(AppPreferences.Key.verifyTLS) private var verifyTLS = true
    @AppStorage(AppPreferences.Key.proxyMode) private var proxyMode = ProxyMode.system.rawValue
    @AppStorage(AppPreferences.Key.proxyURL) private var proxyURL = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SettingsSectionHeader("Request defaults")
            SettingsCard {
                SettingsRow("Timeout (seconds)") {
                    TextField("Timeout (seconds)", value: $timeout, format: .number)
                        .labelsHidden()
                        .multilineTextAlignment(.trailing)
                        .frame(width: 80)
                }
                SettingsRow("Follow redirects") { SettingsSwitch("Follow redirects", isOn: $followRedirects) }
                SettingsRow("Verify TLS certificates") {
                    SettingsSwitch("Verify TLS certificates", isOn: $verifyTLS)
                }
                if !verifyTLS {
                    Text("Invalid certificates will be accepted unless a request overrides this.")
                        .font(.ui(11))
                        .foregroundStyle(Theme.amber)
                        .fixedSize(horizontal: false, vertical: true)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, 14)
                        .padding(.vertical, 8)
                        .hairline(.bottom, color: Theme.rowDivider)
                }
                SettingsRow("Proxy") {
                    SettingsSegmented(
                        "Proxy", selection: $proxyMode,
                        options: ProxyMode.allCases.map { ($0.rawValue, $0.title) })
                }
                if proxyMode == ProxyMode.custom.rawValue {
                    SettingsRow("Proxy URL") {
                        TextField("Proxy URL", text: $proxyURL, prompt: Text("http://proxy:8080"))
                            .labelsHidden()
                            .frame(width: 220)
                    }
                }
            }
            Text("Requests can override these in their Options tab.")
                .font(.ui(11))
                .foregroundStyle(Theme.muted)
                .padding(.horizontal, 4)
        }
    }
}
