import SwiftUI

/// Editable list of name/value rows (params, headers and form fields).
///
/// With `chooseFile` set, each row can switch between a text value and a file (multipart
/// uploads); `chooseFile` returns the picked path, or nil if the user cancelled.
struct KeyValueEditor: View {
    @Binding var rows: [KeyValueRow]
    let namePlaceholder: String
    var chooseFile: (() -> String?)? = nil

    var body: some View {
        List {
            ForEach($rows) { $row in
                HStack(spacing: 8) {
                    Toggle("Enabled", isOn: $row.enabled)
                        .labelsHidden()
                    TextField(namePlaceholder, text: $row.name)
                        .textFieldStyle(.roundedBorder)
                    if chooseFile != nil {
                        Picker("Kind", selection: $row.kind) {
                            Text("Text").tag(KeyValueRow.plainKind)
                            Text("File").tag(KeyValueRow.fileKind)
                        }
                        .labelsHidden()
                        .fixedSize()
                    }
                    TextField(row.isFile ? "path/inside/workspace" : "Value", text: $row.value)
                        .textFieldStyle(.roundedBorder)
                    if row.isFile, let chooseFile {
                        Button("Choose…") {
                            if let path = chooseFile() { row.value = path }
                        }
                    }
                    Button {
                        rows.removeAll { $0.id == row.id }
                    } label: {
                        Image(systemName: "minus.circle")
                    }
                    .buttonStyle(.borderless)
                    .help("Remove")
                    .accessibilityLabel("Remove \(row.name.isEmpty ? namePlaceholder : row.name)")
                }
            }
            Button {
                rows.append(KeyValueRow())
            } label: {
                Label("Add", systemImage: "plus")
            }
            .buttonStyle(.borderless)
        }
        .listStyle(.plain)
    }
}
