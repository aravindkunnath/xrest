import SwiftUI

/// Secrets screen: stored keys, what this service needs, add / reveal / delete.
/// Values are write-only except an explicit per-key Reveal, which lasts until the sheet closes.
struct SecretsSheet: View {
    @Bindable var model: WorkspaceModel
    @State private var stored: [String] = []
    @State private var needed: [SecretsState.Needed] = []
    @State private var newKey = ""
    @State private var newValue = ""
    @State private var revealed: [String: String] = [:]
    @State private var pendingDelete: String?
    @State private var error: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Secrets").font(.headline)
            Text("Stored in the macOS Keychain. Use them as {{secret.KEY}}. Values are never written to the workspace.")
                .font(.subheadline)
                .foregroundStyle(.secondary)

            if !needed.isEmpty { neededSection }
            storedSection
            addForm

            if let error {
                Text(error).font(.ui(12)).foregroundStyle(Theme.red)
            }
            HStack {
                Spacer()
                Button("Done") { model.showSecrets = false }
                    .keyboardShortcut(.defaultAction)
            }
        }
        .padding(20)
        .frame(width: 560)
        .onAppear {
            if let prefill = model.secretsPrefill { newKey = prefill }
            refresh()
        }
        .onDisappear {
            revealed = [:]
            model.secretsPrefill = nil
        }
        .alert(
            "Delete secret?",
            isPresented: Binding(get: { pendingDelete != nil }, set: { if !$0 { pendingDelete = nil } })
        ) {
            Button("Cancel", role: .cancel) {}
            Button("Delete", role: .destructive) {
                if let key = pendingDelete { delete(key) }
            }
        } message: {
            Text("\(pendingDelete ?? "") will be removed from the Keychain.")
        }
    }

    private var neededSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Needed by this service").font(.ui(12, weight: .semibold))
            ForEach(needed, id: \.key) { item in
                HStack {
                    Text(item.key).font(.mono(12))
                    Spacer()
                    if item.isMissing {
                        Text("missing").font(.ui(11)).foregroundStyle(Theme.amber)
                        Button("Add") { newKey = item.key }
                            .controlSize(.small)
                    } else {
                        Text("set").font(.ui(11)).foregroundStyle(Theme.green)
                    }
                }
                .padding(.horizontal, 8)
                .frame(height: 28)
                .chrome(
                    fill: item.isMissing ? Theme.amberFill : Theme.raised,
                    stroke: item.isMissing ? Theme.amberBorder : Theme.border, radius: 6)
            }
        }
    }

    private var storedSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Stored keys").font(.ui(12, weight: .semibold))
            if stored.isEmpty {
                Text("No secrets stored yet.").font(.ui(12)).foregroundStyle(.secondary)
            }
            ForEach(stored, id: \.self) { key in
                HStack(spacing: 8) {
                    Text(key).font(.mono(12))
                    if let value = revealed[key] {
                        Text(value).font(.mono(12)).foregroundStyle(Theme.amberSoft)
                            .textSelection(.enabled).lineLimit(1)
                    }
                    Spacer()
                    Button(revealed[key] == nil ? "Reveal" : "Hide") { toggleReveal(key) }
                        .controlSize(.small)
                    Button("Delete…", role: .destructive) { pendingDelete = key }
                        .controlSize(.small)
                }
                .padding(.horizontal, 8)
                .frame(height: 28)
            }
        }
    }

    private var addForm: some View {
        HStack(spacing: 8) {
            TextField("KEY", text: $newKey)
                .textFieldStyle(.roundedBorder)
                .font(.mono(12))
                .frame(width: 180)
            SecureField("Value", text: $newValue)
                .textFieldStyle(.roundedBorder)
            Button("Save") { save() }
                .disabled(trimmedKey.isEmpty || newValue.isEmpty)
        }
    }

    private var trimmedKey: String { newKey.trimmingCharacters(in: .whitespaces) }

    private func refresh() {
        stored = model.secrets.keys()
        needed = model.neededSecrets()
    }

    private func save() {
        do {
            try model.secrets.set(key: trimmedKey, value: newValue)
            revealed[trimmedKey] = nil
            newKey = ""
            newValue = ""
            error = nil
            refresh()
        } catch {
            self.error = describe(error)
        }
    }

    private func delete(_ key: String) {
        do {
            try model.secrets.delete(key: key)
            revealed[key] = nil
            error = nil
            refresh()
        } catch {
            self.error = describe(error)
        }
    }

    private func toggleReveal(_ key: String) {
        if revealed[key] != nil {
            revealed[key] = nil
        } else {
            revealed[key] = model.secrets.get(key: key) ?? ""
        }
    }
}
