import SwiftUI

/// Every variable layer in one place: the values an environment resolves to and where each comes
/// from, the variables shared by every environment, this user's overrides and the built-ins.
/// Replaces the request pane.
struct VariablesPanelView: View {
    @Bindable var model: WorkspaceModel
    private let globals = GlobalVariablesStore.shared

    @State private var environment: String?
    @State private var selected: String?
    @State private var shared: [KeyValueRow] = []
    @State private var loaded = false
    @State private var overrideText = ""
    @State private var editingOverride = false
    @State private var errorText: String?
    @State private var confirmClose = false

    private var envName: String? { environment ?? model.selectedEnvironment }

    private var normalizedShared: [NameValue] {
        shared.map(\.nameValue)
            .map { var v = $0; v.name = v.name.trimmingCharacters(in: .whitespaces); return v }
            .filter { !$0.name.isEmpty }
    }

    private var isDirty: Bool { normalizedShared != model.workspaceVariables }

    var body: some View {
        // Reading the global rows subscribes this view to changes made in Settings.
        let _ = globals.rows.count
        let rows = model.variableRows(environment: envName)
        let current = rows.first { $0.name == selected } ?? rows.first
        VStack(spacing: 0) {
            header
            HStack(spacing: 0) {
                ScrollView {
                    VStack(alignment: .leading, spacing: 22) {
                        valuesSection(rows, selected: current?.name)
                        sharedSection
                        builtinSection
                    }
                    .padding(16)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                inspector(current)
                    .frame(width: 320)
                    .background(Theme.panel)
                    .hairline(.leading)
            }
            footer
        }
        .onAppear {
            if !loaded {
                shared = model.workspaceVariables.map(KeyValueRow.init)
                loaded = true
            }
            model.panelDirty = isDirty
            model.registerPanelSave { if isDirty { saveShared() } }
        }
        .onDisappear { model.registerPanelSave(nil) }
        .onChange(of: isDirty) { _, dirty in model.panelDirty = dirty }
        .onChange(of: selected) { _, _ in editingOverride = false }
        .onChange(of: environment) { _, _ in editingOverride = false }
        .confirmationDialog("Discard unsaved changes?", isPresented: $confirmClose) {
            Button("Discard", role: .destructive) { model.closePanel() }
            Button("Keep editing", role: .cancel) {}
        }
    }

    // MARK: Header and footer

    private var header: some View {
        HStack(spacing: 12) {
            Text("Variables").font(.ui(14, weight: .semibold))
            Spacer()
            Text("Values in").font(.ui(12)).foregroundStyle(Theme.muted)
            environmentPicker
            Button("Close") { isDirty ? (confirmClose = true) : model.closePanel() }
        }
        .padding(.horizontal, 16)
        .frame(height: 40)
        .hairline(.bottom)
    }

    @ViewBuilder
    private var environmentPicker: some View {
        let binding = Binding<String>(
            get: { envName ?? "" }, set: { environment = $0 })
        let picker = Picker("Environment", selection: binding) {
            ForEach(model.environments, id: \.name) { env in
                Text(env.name).tag(env.name)
            }
        }
        .labelsHidden()
        .fixedSize()
        if model.environments.count <= 4 {
            picker.pickerStyle(.segmented)
        } else {
            picker.pickerStyle(.menu)
        }
    }

    private var footer: some View {
        VStack(alignment: .leading, spacing: 6) {
            if let errorText { Text(errorText).font(.ui(12)).foregroundStyle(Theme.red) }
            Text("The most specific value wins: Local override › Environment › Workspace › Global. Local overrides and secrets never go into git.")
                .font(.ui(11)).foregroundStyle(Theme.muted)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .hairline(.top)
    }

    // MARK: Values

    private func valuesSection(_ rows: [VariableRow], selected current: String?) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            sectionTitle("Values in \(envName ?? "no environment")")
            HStack(spacing: 12) {
                Text("NAME").frame(width: 170, alignment: .leading)
                Text("VALUE").frame(maxWidth: .infinity, alignment: .leading)
                Text("FROM").frame(width: 120, alignment: .leading)
            }
            .font(.ui(11)).tracking(0.7).foregroundStyle(Theme.muted)
            .padding(.horizontal, 8).padding(.vertical, 6)

            if rows.isEmpty {
                Text("No variables yet. Add them per environment in Environments, or below for every environment.")
                    .font(.ui(12)).foregroundStyle(Theme.muted)
                    .padding(8)
            }
            ForEach(rows) { row in
                Button {
                    selected = row.name
                } label: {
                    HStack(spacing: 12) {
                        Text(row.name).font(.mono(12, weight: .semibold))
                            .frame(width: 170, alignment: .leading)
                            .lineLimit(1)
                        HStack(spacing: 8) {
                            Text(row.display).font(.mono(12)).lineLimit(1)
                                .foregroundStyle(row.source == nil ? Theme.muted : Theme.textSoft)
                            if let hidden = row.shadowed {
                                Text(hidden).font(.mono(11)).strikethrough()
                                    .foregroundStyle(Theme.muted).lineLimit(1)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        sourceBadge(row.source)
                            .frame(width: 120, alignment: .leading)
                    }
                    .padding(.horizontal, 8)
                    .frame(minHeight: 32)
                    .background(
                        RoundedRectangle(cornerRadius: 6)
                            .fill(row.name == current ? Theme.selected : .clear))
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityLabel("\(row.name), \(row.display), from \(row.source?.title ?? "nowhere")")
                .accessibilityAddTraits(row.name == current ? .isSelected : [])
            }
        }
    }

    private func sourceBadge(_ layer: VariableLayer?) -> some View {
        let title = layer.map { $0 == .environment ? (envName ?? $0.title) : $0.title } ?? "Not set"
        let color = layer?.badgeColor ?? Theme.muted
        return Text(title)
            .font(.ui(11, weight: .semibold))
            .foregroundStyle(color)
            .padding(.horizontal, 7).padding(.vertical, 2)
            .background(Capsule().fill(color.opacity(0.14)))
    }

    // MARK: Shared by every environment

    private var sharedSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            sectionTitle("Shared by every environment")
            Text("Saved in variables.yaml and committed with the workspace. An environment's own value wins.")
                .font(.ui(11)).foregroundStyle(Theme.muted)
            ForEach($shared) { $row in
                HStack(spacing: 8) {
                    Toggle("Enabled", isOn: $row.enabled).toggleStyle(.checkbox).labelsHidden()
                    TextField("NAME", text: $row.name)
                        .themedField(mono: true)
                        .frame(width: 200)
                    if let key = row.secretKey {
                        Label(key, systemImage: "key.fill")
                            .font(.mono(12)).foregroundStyle(Theme.amberSoft)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        TextField("value", text: $row.value).themedField(mono: true)
                    }
                    Button {
                        shared.removeAll { $0.id == row.id }
                    } label: {
                        Image(systemName: "xmark").font(.system(size: 9, weight: .medium))
                            .foregroundStyle(Theme.muted).frame(width: 24, height: 24)
                    }
                    .buttonStyle(HoverButtonStyle(radius: 4))
                    .help("Remove")
                    .accessibilityLabel("Remove \(row.name.isEmpty ? "variable" : row.name)")
                }
            }
            HStack {
                Button("+ Add shared variable") { shared.append(KeyValueRow()) }
                    .buttonStyle(.plain).font(.ui(12)).foregroundStyle(Theme.muted)
                Spacer()
                Button("Revert") {
                    shared = model.workspaceVariables.map(KeyValueRow.init)
                    errorText = nil
                }
                .disabled(!isDirty)
                Button("Save") { saveShared() }
                    .disabled(!isDirty)
            }
            .padding(.top, 4)
        }
    }

    private func saveShared() {
        do {
            try model.saveWorkspaceVariables(normalizedShared)
            shared = model.workspaceVariables.map(KeyValueRow.init)
            errorText = nil
        } catch {
            errorText = describe(error)
        }
    }

    // MARK: Built-ins

    private var builtinSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            sectionTitle("Built-in")
            ForEach(builtinVariables(), id: \.name) { b in
                HStack(alignment: .firstTextBaseline, spacing: 12) {
                    Text("{{\(b.name)}}").font(.mono(12, weight: .semibold))
                        .frame(width: 170, alignment: .leading)
                        .textSelection(.enabled)
                    Text(b.description).font(.ui(12)).foregroundStyle(Theme.muted)
                }
            }
            Text("Each gets a fresh value per send; the same name used twice in one request has one value.")
                .font(.ui(11)).foregroundStyle(Theme.muted)
        }
    }

    // MARK: Inspector

    @ViewBuilder
    private func inspector(_ row: VariableRow?) -> some View {
        if let row, let env = envName {
            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    VStack(alignment: .leading, spacing: 3) {
                        Text(row.name).font(.mono(14, weight: .bold)).textSelection(.enabled)
                        Text("Every layer that defines it, most specific first")
                            .font(.ui(11)).foregroundStyle(Theme.muted)
                    }
                    ForEach(VariableInspector.entries(for: row)) { entry in
                        layerCard(entry, env: env)
                    }
                    overrideControls(row, env: env)
                }
                .padding(16)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        } else {
            Text("Select a variable to see where its value comes from.")
                .font(.ui(12)).foregroundStyle(Theme.muted)
                .padding(16)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
    }

    private func layerCard(_ entry: VariableInspector.Entry, env: String) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack {
                Text(entry.layer == .environment ? env : entry.layer.title)
                    .font(.ui(12, weight: .semibold))
                Spacer()
                if entry.inUse {
                    Text("In use").font(.ui(10, weight: .semibold))
                        .foregroundStyle(Theme.bg)
                        .padding(.horizontal, 6).padding(.vertical, 1)
                        .background(Capsule().fill(Theme.accent))
                } else if entry.overridden {
                    Text(entry.value?.enabled == false ? "Disabled" : "Overridden")
                        .font(.ui(10)).foregroundStyle(Theme.muted)
                }
            }
            Text(entry.layer.scope).font(.ui(10)).foregroundStyle(Theme.muted)
            if let value = entry.value {
                Text(VariableRow.show(value))
                    .font(.mono(12))
                    .strikethrough(entry.overridden)
                    .foregroundStyle(entry.inUse ? Theme.text : Theme.muted)
                    .textSelection(.enabled)
            } else {
                Text("Not set").font(.ui(12)).foregroundStyle(Theme.muted)
            }
            if entry.layer == .global {
                SettingsLink { Text("Edit global variables…").font(.ui(11)) }
                    .buttonStyle(.link)
                    .simultaneousGesture(TapGesture().onEnded {
                        UserDefaults.standard.set(SettingsSection.variables.rawValue, forKey: "settings.lastTab")
                    })
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 8).fill(Theme.bg))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(entry.inUse ? Theme.accent : Theme.border, lineWidth: entry.inUse ? 1.5 : 1))
    }

    @ViewBuilder
    private func overrideControls(_ row: VariableRow, env: String) -> some View {
        let existing = row.localOverride
        VStack(alignment: .leading, spacing: 6) {
            if editingOverride {
                TextField("Your value in \(env)", text: $overrideText)
                    .themedField(mono: true)
                    .onSubmit { commitOverride(row, env: env) }
                HStack {
                    Button("Save Override") { commitOverride(row, env: env) }
                    Button("Cancel") { editingOverride = false }
                }
            } else {
                HStack {
                    Button(existing == nil ? "Override for Me…" : "Edit My Override…") {
                        overrideText = existing?.value ?? (row.isSecret || row.source == nil ? "" : row.display)
                        editingOverride = true
                    }
                    if existing != nil {
                        Button("Remove My Override") {
                            run { try model.setLocalOverride(environment: env, name: row.name, value: nil) }
                        }
                    }
                }
            }
            Text("Only you see an override. It stays on this Mac and is never committed.")
                .font(.ui(11)).foregroundStyle(Theme.muted)
        }
    }

    private func commitOverride(_ row: VariableRow, env: String) {
        run { try model.setLocalOverride(environment: env, name: row.name, value: overrideText) }
        editingOverride = false
    }

    private func run(_ action: () throws -> Void) {
        do {
            try action()
            errorText = nil
        } catch {
            errorText = describe(error)
        }
    }

    private func sectionTitle(_ text: String) -> some View {
        Text(text.uppercased())
            .font(.ui(11, weight: .semibold)).tracking(0.7)
            .foregroundStyle(Theme.muted)
    }
}
