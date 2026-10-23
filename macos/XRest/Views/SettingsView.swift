import SwiftUI

/// The Settings window (⌘,). Values live in UserDefaults (never in a workspace); open windows
/// pick up changes through `UserDefaults.didChangeNotification`.
struct SettingsView: View {
    @AppStorage(AppPreferences.Key.appearance) private var appearance = AppearanceMode.system.rawValue
    @AppStorage(AppPreferences.Key.zoom) private var zoom = 1.0
    @AppStorage(AppPreferences.Key.timeoutSeconds) private var timeout = AppPreferences.defaultTimeoutSeconds
    @AppStorage(AppPreferences.Key.followRedirects) private var followRedirects = true
    @AppStorage(AppPreferences.Key.verifyTLS) private var verifyTLS = true
    @AppStorage(AppPreferences.Key.proxyMode) private var proxyMode = ProxyMode.system.rawValue
    @AppStorage(AppPreferences.Key.proxyURL) private var proxyURL = ""
    @AppStorage(AppPreferences.Key.maxVersions) private var maxVersions = AppPreferences.maxVersionsRange.upperBound

    var body: some View {
        Form {
            Section("Appearance") {
                Picker("Theme", selection: $appearance) {
                    ForEach(AppearanceMode.allCases) { Text($0.title).tag($0.rawValue) }
                }
                .pickerStyle(.segmented)
                Picker("Zoom", selection: $zoom) {
                    ForEach(AppPreferences.zoomSteps, id: \.self) { Text("\(Int(($0 * 100).rounded()))%").tag($0) }
                }
                Text("Scales code, request bodies and responses.")
                    .font(.caption).foregroundStyle(.secondary)
            }

            Section("Request defaults") {
                TextField("Timeout (seconds)", value: $timeout, format: .number)
                    .frame(maxWidth: 200)
                Toggle("Follow redirects", isOn: $followRedirects)
                Toggle("Verify TLS certificates", isOn: $verifyTLS)
                if !verifyTLS {
                    Text("Invalid certificates will be accepted unless a request overrides this.")
                        .font(.caption).foregroundStyle(.orange)
                }
                Picker("Proxy", selection: $proxyMode) {
                    ForEach(ProxyMode.allCases) { Text($0.title).tag($0.rawValue) }
                }
                .pickerStyle(.segmented)
                if proxyMode == ProxyMode.custom.rawValue {
                    TextField("Proxy URL", text: $proxyURL, prompt: Text("http://proxy:8080"))
                }
                Text("Requests can override these in their Options tab.")
                    .font(.caption).foregroundStyle(.secondary)
            }

            Section("Versions") {
                Stepper(
                    "Keep \(maxVersions) versions per endpoint", value: $maxVersions,
                    in: AppPreferences.maxVersionsRange)
            }
        }
        .formStyle(.grouped)
        .frame(width: 460)
        .fixedSize(horizontal: false, vertical: true)
        .preferredColorScheme(AppearanceMode(rawValue: appearance)?.colorScheme)
    }
}
