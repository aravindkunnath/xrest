import AppKit
import SwiftUI

struct AppearanceTab: View {
    @AppStorage(AppPreferences.Key.appearance) private var appearance = AppearanceMode.system.rawValue
    @AppStorage(AppPreferences.Key.zoom) private var zoom = 1.0
    @AppStorage(AppPreferences.Key.editorFontFamily) private var fontFamily = ""
    @AppStorage(AppPreferences.Key.editorFontSize) private var fontSize = 12
    @AppStorage(AppPreferences.Key.wrapLines) private var wrapLines = true
    @AppStorage(AppPreferences.Key.showLineNumbers) private var showLineNumbers = true
    @AppStorage(AppPreferences.Key.density) private var density = Density.regular.rawValue

    /// Installed families that have a fixed-pitch face.
    private static let monospacedFamilies: [String] = {
        let manager = NSFontManager.shared
        return manager.availableFontFamilies.filter { family in
            manager.availableMembers(ofFontFamily: family)?.contains { member in
                (member[3] as? Int).map { NSFontTraitMask(rawValue: UInt($0)).contains(.fixedPitchFontMask) } ?? false
            } ?? false
        }
    }()

    private var familyOptions: [String] {
        Self.monospacedFamilies.contains(fontFamily) || fontFamily.isEmpty
            ? Self.monospacedFamilies : [fontFamily] + Self.monospacedFamilies
    }

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
            SettingsSectionHeader("Editor").padding(.top, 14)
            SettingsCard {
                SettingsRow("Font") {
                    Picker("Font", selection: $fontFamily) {
                        Text("System Mono").tag("")
                        ForEach(familyOptions, id: \.self) { Text($0).tag($0) }
                    }
                    .labelsHidden()
                    .fixedSize()
                }
                SettingsRow("Font size") {
                    SettingsStepper(
                        "Font size", value: $fontSize, range: AppPreferences.editorFontSizeRange, unit: "pt")
                }
                SettingsRow("Wrap long lines") { SettingsSwitch("Wrap long lines", isOn: $wrapLines) }
                SettingsRow("Show line numbers") {
                    SettingsSwitch("Show line numbers", isOn: $showLineNumbers)
                }
            }
            SettingsSectionHeader("Layout").padding(.top, 14)
            SettingsCard {
                SettingsRow("Density") {
                    SettingsSegmented(
                        "Density", selection: $density,
                        options: Density.allCases.map { ($0.rawValue, $0.title) })
                }
            }
        }
    }
}
