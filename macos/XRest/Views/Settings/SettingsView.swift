import SwiftUI

/// The Settings window (⌘,). Values live in UserDefaults (never in a workspace); open windows
/// pick up changes through `UserDefaults.didChangeNotification`.
struct SettingsView: View {
    @AppStorage(AppPreferences.Key.appearance) private var appearance = AppearanceMode.system.rawValue
    @AppStorage("settings.lastTab") private var lastTab = SettingsSection.appearance.rawValue

    private var section: SettingsSection { SettingsSection(rawValue: lastTab) ?? .appearance }

    var body: some View {
        VStack(spacing: 0) {
            tabStrip
            Divider().overlay(Theme.border)
            ScrollView {
                content
                    .padding(28)
                    .frame(maxWidth: .infinity, alignment: .topLeading)
            }
            .background(Theme.bg)
        }
        .frame(width: 640)
        .frame(minHeight: 360, idealHeight: 460)
        .background(Theme.bg)
        .navigationTitle(section.title)
        .preferredColorScheme(AppearanceMode(rawValue: appearance)?.colorScheme)
    }

    private var tabStrip: some View {
        HStack(spacing: 4) {
            ForEach(SettingsSection.allCases) { item in
                let selected = item == section
                Button {
                    lastTab = item.rawValue
                } label: {
                    VStack(spacing: 4) {
                        Image(systemName: item.symbol)
                            .font(.system(size: 18))
                            .accessibilityHidden(true)
                        Text(item.title).font(.ui(11))
                    }
                    .foregroundStyle(selected ? Theme.text : Theme.muted)
                    .frame(width: 76, height: 52)
                    .background(
                        RoundedRectangle(cornerRadius: 8).fill(selected ? Theme.selected : .clear)
                    )
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityLabel(item.title)
                .accessibilityAddTraits(selected ? .isSelected : [])
            }
        }
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity)
        .background(Theme.sidebar)
    }

    @ViewBuilder
    private var content: some View {
        switch section {
        case .appearance: AppearanceTab()
        case .requests: RequestsTab()
        case .versions: VersionsTab()
        case .safety: Text("Safety settings are coming soon.").foregroundStyle(Theme.muted)
        }
    }
}
