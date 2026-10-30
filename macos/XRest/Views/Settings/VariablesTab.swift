import SwiftUI

/// Global variables: yours, available in every workspace, below everything a workspace defines.
struct VariablesTab: View {
    private let store = GlobalVariablesStore.shared
    @State private var rows: [KeyValueRow] = []
    @State private var loaded = false
    @State private var errorText: String?
    @State private var savedNotice = false

    private var normalized: [NameValue] {
        rows.map(\.nameValue)
            .map { var v = $0; v.name = v.name.trimmingCharacters(in: .whitespaces); return v }
            .filter { !$0.name.isEmpty }
    }

    private var isDirty: Bool { normalized != store.rows }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SettingsSectionHeader("Global variables")
            Text("Available in every workspace. A workspace, an environment or your own override of the same name wins. Use {{secret.KEY}} for anything sensitive.")
                .font(.ui(11)).foregroundStyle(Theme.muted)
                .fixedSize(horizontal: false, vertical: true)
            SettingsCard {
                if rows.isEmpty {
                    SettingsRow("No global variables yet") { EmptyView() }
                }
                ForEach($rows) { $row in
                    HStack(spacing: 8) {
                        Toggle("Enabled", isOn: $row.enabled).toggleStyle(.checkbox).labelsHidden()
                        TextField("NAME", text: $row.name)
                            .themedField(mono: true)
                            .frame(width: 180)
                        TextField("value", text: $row.value).themedField(mono: true)
                        Button {
                            rows.removeAll { $0.id == row.id }
                        } label: {
                            Image(systemName: "xmark").font(.system(size: 9, weight: .medium))
                                .foregroundStyle(Theme.muted).frame(width: 24, height: 24)
                        }
                        .buttonStyle(HoverButtonStyle(radius: 4))
                        .help("Remove")
                        .accessibilityLabel("Remove \(row.name.isEmpty ? "variable" : row.name)")
                    }
                    .padding(.horizontal, 14)
                    .padding(.vertical, 6)
                    .hairline(.bottom, color: Theme.rowDivider)
                }
            }
            HStack {
                Button("+ Add variable") { rows.append(KeyValueRow()) }
                if savedNotice && !isDirty {
                    Text("Saved").font(.ui(11)).foregroundStyle(Theme.muted)
                }
                Spacer()
                Button("Revert") {
                    rows = store.rows.map(KeyValueRow.init)
                    errorText = nil
                }
                .disabled(!isDirty)
                Button("Save") { save() }
                    .keyboardShortcut("s", modifiers: .command)
                    .disabled(!isDirty)
            }
            if let message = errorText ?? store.error {
                Text(message).font(.ui(12)).foregroundStyle(Theme.red)
            }
            HStack(spacing: 8) {
                Text("ONLY YOU")
                    .font(.ui(10, weight: .semibold))
                    .padding(.horizontal, 6).padding(.vertical, 1)
                    .overlay(RoundedRectangle(cornerRadius: 4).stroke(Theme.border))
                Text("Saved in Application Support/XRest/variables.yaml, never in a workspace.")
                    .font(.ui(11))
            }
            .foregroundStyle(Theme.muted)
            .padding(.top, 8)
        }
        .onAppear {
            if !loaded {
                rows = store.rows.map(KeyValueRow.init)
                loaded = true
            }
        }
    }

    private func save() {
        do {
            try store.save(normalized)
            rows = store.rows.map(KeyValueRow.init)
            errorText = nil
            savedNotice = true
        } catch {
            errorText = describe(error)
        }
    }
}
