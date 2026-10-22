import SwiftUI

/// Editable list of name/value rows (params and headers).
struct KeyValueEditor: View {
    @Binding var rows: [KeyValueRow]
    let namePlaceholder: String

    var body: some View {
        List {
            ForEach($rows) { $row in
                HStack(spacing: 8) {
                    Toggle("Enabled", isOn: $row.enabled)
                        .labelsHidden()
                    TextField(namePlaceholder, text: $row.name)
                        .textFieldStyle(.roundedBorder)
                    TextField("Value", text: $row.value)
                        .textFieldStyle(.roundedBorder)
                    Button {
                        rows.removeAll { $0.id == row.id }
                    } label: {
                        Image(systemName: "minus.circle")
                    }
                    .buttonStyle(.borderless)
                    .help("Remove")
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
