import SwiftUI

/// Editable list of name/value rows (params, headers and form fields), laid out like the mockup's
/// KEY / VALUE grid.
///
/// With `chooseFile` set, each row can switch between a text value and a file (multipart
/// uploads); `chooseFile` returns the picked path, or nil if the user cancelled.
struct KeyValueEditor: View {
    @Binding var rows: [KeyValueRow]
    let namePlaceholder: String
    var chooseFile: (() -> String?)? = nil
    /// Offers `{{…}}` completions in value fields when set.
    var completions: CompletionProvider? = nil

    var body: some View {
        let metrics = DensityMetrics(PreferencesStore.shared.density)
        return ScrollView {
            VStack(spacing: 0) {
                HStack(spacing: 8) {
                    Color.clear.frame(width: 28)
                    Text("KEY").frame(maxWidth: .infinity, alignment: .leading)
                    Text("VALUE").frame(maxWidth: .infinity, alignment: .leading)
                    Color.clear.frame(width: 24)
                }
                .font(.ui(11))
                .tracking(0.7)
                .foregroundStyle(Theme.muted)
                .padding(.vertical, 6)

                ForEach($rows) { $row in
                    HStack(spacing: 8) {
                        Button {
                            row.enabled.toggle()
                        } label: {
                            RoundedRectangle(cornerRadius: 3)
                                .fill(row.enabled ? Theme.accent : .clear)
                                .overlay(
                                    RoundedRectangle(cornerRadius: 3)
                                        .stroke(
                                            row.enabled ? Theme.accent : Theme.controlOff,
                                            lineWidth: 1)
                                )
                                .overlay {
                                    if row.enabled {
                                        Image(systemName: "checkmark")
                                            .font(.system(size: 8, weight: .heavy))
                                            .foregroundStyle(Theme.bg)
                                    }
                                }
                                .frame(width: 14, height: 14)
                                .frame(width: 28, height: metrics.rowHeight)
                                .contentShape(Rectangle())
                        }
                        .buttonStyle(.plain)
                        .accessibilityLabel(row.enabled ? "Disable row" : "Enable row")

                        VariableTextField(
                            placeholder: namePlaceholder, text: $row.name,
                            completions: completions,
                            textColor: row.enabled ? Theme.text : Theme.muted
                        )
                        .frame(maxWidth: .infinity)

                        HStack(spacing: 6) {
                            if chooseFile != nil {
                                Picker("Kind", selection: $row.kind) {
                                    Text("Text").tag(KeyValueRow.plainKind)
                                    Text("File").tag(KeyValueRow.fileKind)
                                }
                                .labelsHidden()
                                .fixedSize()
                            }
                            VariableTextField(
                                placeholder: row.isFile ? "path/inside/workspace" : "Value",
                                text: $row.value,
                                completions: row.isFile ? nil : completions)
                            if row.isFile, let chooseFile {
                                Button("Choose…") {
                                    if let path = chooseFile() { row.value = path }
                                }
                            }
                        }
                        .frame(maxWidth: .infinity)

                        Button {
                            rows.removeAll { $0.id == row.id }
                        } label: {
                            Image(systemName: "xmark")
                                .font(.system(size: 9, weight: .medium))
                                .foregroundStyle(Theme.muted)
                                .frame(width: 24, height: metrics.rowHeight)
                        }
                        .buttonStyle(HoverButtonStyle(radius: 4))
                        .help("Remove")
                        .accessibilityLabel("Remove \(row.name.isEmpty ? namePlaceholder : row.name)")
                    }
                    .padding(.vertical, metrics.rowPadding)
                    .hairline(.top, color: Theme.rowDivider)
                }

                Button {
                    rows.append(KeyValueRow())
                } label: {
                    Text("+ Add")
                        .font(.ui(12))
                        .foregroundStyle(Theme.muted)
                        .padding(.vertical, 6)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .buttonStyle(.plain)
                .hairline(.top, color: Theme.rowDivider)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
        }
    }
}
