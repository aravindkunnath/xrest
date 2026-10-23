import SwiftUI

/// Variables grid: rows are variable names, columns are environments. Replaces the request pane.
struct EnvironmentsEditorView: View {
    @Bindable var model: WorkspaceModel
    @State private var grid = EnvironmentGrid(environments: [])
    @State private var loaded = false
    @State private var saveError: String?
    @State private var confirmClose = false

    private let nameWidth: CGFloat = 200
    private let cellWidth: CGFloat = 240

    private var isDirty: Bool { grid.isDirty(comparedTo: model.environments) }
    private var errors: [String] { grid.validationErrors }

    var body: some View {
        VStack(spacing: 0) {
            header
            ScrollView([.horizontal, .vertical]) {
                VStack(alignment: .leading, spacing: 0) {
                    columnHeader
                    ForEach(grid.rows) { row in rowView(row) }
                    Button {
                        grid.addRow()
                    } label: {
                        Text("+ Add variable")
                            .font(.ui(12))
                            .foregroundStyle(Theme.muted)
                            .padding(.vertical, 8)
                    }
                    .buttonStyle(.plain)
                }
                .padding(16)
            }
            footer
        }
        .onAppear {
            if !loaded {
                grid = EnvironmentGrid(environments: model.environments)
                loaded = true
            }
            // Defensive: a panel that appears clean must not leave a stale dirty flag behind.
            model.panelDirty = isDirty
        }
        .onChange(of: isDirty) { _, dirty in model.panelDirty = dirty }
        .onAppear {
            model.registerPanelSave { if isDirty && errors.isEmpty { save() } }
        }
        .onDisappear { model.registerPanelSave(nil) }
        .confirmationDialog("Discard unsaved changes?", isPresented: $confirmClose) {
            Button("Discard", role: .destructive) { model.closePanel() }
            Button("Keep editing", role: .cancel) {}
        }
    }

    private var header: some View {
        HStack {
            Text("Environments and variables").font(.ui(14, weight: .semibold))
            Spacer()
            Button("Close") { isDirty ? (confirmClose = true) : model.closePanel() }
        }
        .padding(.horizontal, 16)
        .frame(height: 40)
        .hairline(.bottom)
    }

    private var columnHeader: some View {
        HStack(alignment: .bottom, spacing: 12) {
            Text("VARIABLE").font(.ui(11)).tracking(0.7).foregroundStyle(Theme.muted)
                .frame(width: nameWidth, alignment: .leading)
            ForEach(Array(grid.columns.enumerated()), id: \.offset) { index, col in
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 2) {
                        ColumnNameField(name: col.name) { grid.renameColumn(from: col.name, to: $0) }
                        columnButton("chevron.left", "Move left", disabled: index == 0) {
                            grid.moveColumn(from: IndexSet(integer: index), to: index - 1)
                        }
                        columnButton(
                            "chevron.right", "Move right", disabled: index == grid.columns.count - 1
                        ) {
                            grid.moveColumn(from: IndexSet(integer: index), to: index + 2)
                        }
                        columnButton("trash", "Remove environment", disabled: grid.columns.count <= 1) {
                            grid.removeColumn(name: col.name)
                        }
                    }
                    Toggle(
                        "Unsafe",
                        isOn: Binding(
                            get: { col.isUnsafe }, set: { grid.setUnsafe(env: col.name, $0) })
                    )
                    .toggleStyle(.checkbox)
                    .font(.ui(11))
                    .foregroundStyle(col.isUnsafe ? Theme.amber : Theme.muted)
                    .help("Sends in an unsafe environment need a countdown confirmation")
                }
                .frame(width: cellWidth, alignment: .leading)
            }
            Button {
                grid.addColumn(name: uniqueEnvironmentName())
            } label: {
                Text("+ Add environment").font(.ui(12)).foregroundStyle(Theme.muted)
            }
            .buttonStyle(.plain)
            Color.clear.frame(width: 24)
        }
        .padding(.bottom, 6)
    }

    private func columnButton(
        _ icon: String, _ label: String, disabled: Bool, action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: 9, weight: .medium))
                .foregroundStyle(Theme.muted)
                .frame(width: 20, height: 20)
        }
        .buttonStyle(HoverButtonStyle(radius: 4))
        .disabled(disabled)
        .help(label)
        .accessibilityLabel(label)
    }

    private func uniqueEnvironmentName() -> String {
        let taken = Set(grid.columns.map(\.name))
        if !taken.contains("ENV") { return "ENV" }
        var n = 2
        while taken.contains("ENV \(n)") { n += 1 }
        return "ENV \(n)"
    }

    private func rowView(_ row: EnvironmentGrid.Row) -> some View {
        HStack(spacing: 12) {
            TextField(
                "NAME",
                text: Binding(
                    get: { row.name }, set: { grid.setName(rowID: row.id, $0) })
            )
            .textFieldStyle(.plain)
            .font(.mono(12))
            .frame(width: nameWidth, alignment: .leading)

            ForEach(Array(grid.columns.enumerated()), id: \.offset) { _, col in
                cell(row, env: col.name)
                    .frame(width: cellWidth, alignment: .leading)
            }
            Button {
                grid.removeRow(id: row.id)
            } label: {
                Image(systemName: "xmark")
                    .font(.system(size: 9, weight: .medium))
                    .foregroundStyle(Theme.muted)
                    .frame(width: 24, height: 24)
            }
            .buttonStyle(HoverButtonStyle(radius: 4))
            .help("Remove variable from every environment")
            .accessibilityLabel("Remove \(row.name.isEmpty ? "variable" : row.name)")
        }
        .padding(.vertical, 4)
        .hairline(.top, color: Theme.rowDivider)
    }

    private func cell(_ row: EnvironmentGrid.Row, env: String) -> some View {
        let value = row.cells[env]
        let enabled = value?.enabled ?? true
        return HStack(spacing: 6) {
            Toggle(
                "Enabled",
                isOn: Binding(
                    get: { enabled }, set: { grid.setEnabled(rowID: row.id, env: env, $0) })
            )
            .toggleStyle(.checkbox)
            .labelsHidden()
            .disabled(value == nil)
            .help("Enabled")

            if let key = value?.secretKey {
                HStack(spacing: 4) {
                    Image(systemName: "key.fill").font(.system(size: 9))
                    Text(key).lineLimit(1)
                }
                .font(.mono(12))
                .foregroundStyle(Theme.amberSoft)
                .frame(maxWidth: .infinity, alignment: .leading)
            } else {
                TextField(
                    "not set",
                    text: Binding(
                        get: { value?.value ?? "" },
                        set: { grid.setValue(rowID: row.id, env: env, $0) })
                )
                .textFieldStyle(.plain)
                .font(.mono(12))
                .foregroundStyle(enabled ? Theme.textSoft : Theme.muted)
            }

            Menu {
                if value?.secretKey != nil {
                    Button("Unlink secret") { grid.unlinkSecret(rowID: row.id, env: env) }
                }
                let keys = model.secretKeyChoices()
                if keys.isEmpty {
                    Text("No secrets yet")
                } else {
                    Menu("Link secret") {
                        ForEach(keys, id: \.self) { key in
                            Button(key) { grid.linkSecret(rowID: row.id, env: env, key: key) }
                        }
                    }
                }
                Button("Manage secrets…") { model.openSecrets() }
            } label: {
                Image(systemName: "key")
            }
            .menuStyle(.borderlessButton)
            .menuIndicator(.hidden)
            .fixedSize()
            .help("Link secret")
            .accessibilityLabel("Link secret")
        }
    }

    private var footer: some View {
        VStack(alignment: .leading, spacing: 6) {
            ForEach(errors, id: \.self) { Text($0).font(.ui(12)).foregroundStyle(Theme.red) }
            if let saveError { Text(saveError).font(.ui(12)).foregroundStyle(Theme.red) }
            HStack {
                Text("Secrets are referenced by key; values stay in the Keychain.")
                    .font(.ui(11)).foregroundStyle(Theme.muted)
                Spacer()
                Button("Revert") {
                    grid = EnvironmentGrid(environments: model.environments)
                    saveError = nil
                }
                .disabled(!isDirty)
                Button("Save") { save() }
                    .disabled(!isDirty || !errors.isEmpty)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .hairline(.top)
    }

    private func save() {
        do {
            try model.saveEnvironments(grid.toConfigs())
            grid = EnvironmentGrid(environments: model.environments)
            saveError = nil
        } catch {
            saveError = describe(error)
        }
    }
}

/// Environment name field that commits on return or focus loss, so half-typed names never
/// collide with another environment mid-edit.
private struct ColumnNameField: View {
    let name: String
    let commit: (String) -> Void
    @State private var draft = ""
    @FocusState private var focused: Bool

    var body: some View {
        TextField("ENVIRONMENT", text: $draft)
            .textFieldStyle(.plain)
            .font(.ui(12, weight: .semibold))
            .focused($focused)
            .onAppear { draft = name }
            .onChange(of: name) { _, new in if !focused { draft = new } }
            .onSubmit { commit(draft) }
            .onChange(of: focused) { _, now in if !now { commit(draft) } }
    }
}
