import SwiftUI

/// Service name, authentication and preflight. Replaces the request pane.
struct ServiceSettingsView: View {
    @Bindable var model: WorkspaceModel
    @State private var form: ServiceSettingsForm
    @State private var saveError: String?
    @State private var confirmClose = false

    init(model: WorkspaceModel) {
        self.model = model
        _form = State(initialValue: ServiceSettingsForm(model.service))
    }

    private var isDirty: Bool { form != ServiceSettingsForm(model.service) }
    private var errors: [String] { form.validationErrors }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Service settings").font(.ui(14, weight: .semibold))
                Spacer()
                Button("Close") { isDirty ? (confirmClose = true) : model.closePanel() }
            }
            .padding(.horizontal, 16)
            .frame(height: 40)
            .hairline(.bottom)

            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    generalSection
                    authSection
                    preflightSection
                }
                .padding(16)
                .frame(maxWidth: 720, alignment: .leading)
                .frame(maxWidth: .infinity, alignment: .leading)
            }

            VStack(alignment: .leading, spacing: 6) {
                ForEach(errors, id: \.self) { Text($0).font(.ui(12)).foregroundStyle(Theme.red) }
                if let saveError { Text(saveError).font(.ui(12)).foregroundStyle(Theme.red) }
                HStack {
                    Spacer()
                    Button("Revert") {
                        form = ServiceSettingsForm(model.service)
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

    // MARK: Sections

    private var generalSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionTitle("General")
            TextField("Name", text: $form.name).textFieldStyle(.roundedBorder)
        }
    }

    private var authSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionTitle("Authentication")
            Toggle("Authenticate requests", isOn: $form.isAuthenticated)
            Text("Applies to requests that have Authenticate turned on. Values may use {{variables}} and {{secret.KEY}}.")
                .font(.ui(11)).foregroundStyle(Theme.muted)
            Picker("Type", selection: $form.authType) {
                Text("None").tag("none")
                Text("Basic").tag("basic")
                Text("Bearer").tag("bearer")
                Text("API key").tag("apikey")
            }
            .pickerStyle(.segmented)
            .frame(maxWidth: 360)

            switch form.authType {
            case "basic":
                TextField("Username", text: $form.basicUser).textFieldStyle(.roundedBorder)
                TextField("Password", text: $form.basicPass).textFieldStyle(.roundedBorder)
            case "bearer":
                TextField("Token", text: $form.bearerToken).textFieldStyle(.roundedBorder)
            case "apikey":
                TextField("Name", text: $form.apiKeyName).textFieldStyle(.roundedBorder)
                TextField("Value", text: $form.apiKeyValue).textFieldStyle(.roundedBorder)
                Picker("Send in", selection: $form.apiKeyLocation) {
                    Text("Header").tag("header")
                    Text("Query").tag("query")
                }
                .pickerStyle(.segmented)
                .frame(maxWidth: 240)
            default:
                EmptyView()
            }

            if !form.plainCredentialFields.isEmpty {
                HStack(alignment: .top, spacing: 8) {
                    Image(systemName: "exclamationmark.triangle.fill").foregroundStyle(Theme.amber)
                    Text("\(form.plainCredentialFields.joined(separator: ", ")) is stored as plain text in service.yaml. Use {{secret.KEY}} to keep it in the Keychain.")
                        .font(.ui(12))
                    Spacer()
                    Button("Secrets…") { model.openSecrets() }.controlSize(.small)
                }
                .padding(8)
                .chrome(fill: Theme.amberFill, stroke: Theme.amberBorder, radius: 6)
            }
        }
    }

    @ViewBuilder
    private var preflightSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionTitle("Preflight")
            if form.preflight == nil {
                Text("No preflight. A preflight fetches a token before each request and writes it to a header.")
                    .font(.ui(12)).foregroundStyle(Theme.muted)
                Button("Add preflight") { form.preflight = PreflightForm() }
            } else {
                preflightFields(Binding(get: { form.preflight ?? PreflightForm() }, set: { form.preflight = $0 }))
                HStack {
                    Button("Test preflight") { model.testPreflight(form.toSettings().preflight!) }
                        .disabled(isTesting || (form.preflight?.url.isEmpty ?? true))
                    Button("Remove preflight", role: .destructive) { form.preflight = nil }
                    if isTesting { ProgressView().controlSize(.small) }
                }
                testResult
            }
        }
    }

    private var isTesting: Bool {
        if case .running = model.preflightTest { return true }
        return false
    }

    private func preflightFields(_ p: Binding<PreflightForm>) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Toggle("Enabled", isOn: p.enabled)
            HStack {
                Picker("Method", selection: p.method) {
                    ForEach(PreflightForm.methods, id: \.self) { Text($0).tag($0) }
                }
                .frame(width: 160)
                TextField("URL", text: p.url).textFieldStyle(.roundedBorder).font(.mono(12))
            }
            Picker("Body", selection: p.bodyType) {
                Text("Raw").tag("raw")
                Text("URL-encoded").tag("urlencoded")
            }
            .pickerStyle(.segmented).frame(maxWidth: 300)
            if p.wrappedValue.bodyType == "urlencoded" {
                KeyValueEditor(rows: p.bodyParams, namePlaceholder: "Field")
                    .frame(height: 140).chrome()
            } else {
                TextEditor(text: p.body)
                    .font(.mono(12)).frame(height: 100).chrome()
            }
            Text("Headers").font(.ui(12, weight: .semibold))
            KeyValueEditor(rows: p.headers, namePlaceholder: "Header")
                .frame(height: 120).chrome()
            HStack {
                TextField("Token key (JSON path)", text: p.tokenKey).textFieldStyle(.roundedBorder)
                TextField("Token header (default Authorization)", text: p.tokenHeader)
                    .textFieldStyle(.roundedBorder)
            }
            Toggle("Cache token", isOn: p.cacheToken)
            if p.wrappedValue.cacheToken {
                HStack {
                    Picker("Duration", selection: p.durationDerived) {
                        Text("Derived").tag(true)
                        Text("Fixed").tag(false)
                    }
                    .pickerStyle(.segmented).frame(width: 200)
                    if !p.wrappedValue.durationDerived {
                        TextField("Amount", text: p.durationNumber)
                            .textFieldStyle(.roundedBorder).frame(width: 100)
                    } else {
                        TextField("Response key (e.g. expires_in)", text: p.durationKey)
                            .textFieldStyle(.roundedBorder)
                    }
                    Picker("Unit", selection: p.durationUnit) {
                        ForEach(PreflightForm.units, id: \.self) { Text($0).tag($0) }
                    }
                    .frame(width: 180)
                }
            }
        }
    }

    @ViewBuilder
    private var testResult: some View {
        switch model.preflightTest {
        case .idle, .running:
            EmptyView()
        case .failed(let message):
            Text(message).font(.ui(12)).foregroundStyle(Theme.red).textSelection(.enabled)
        case .done(let r):
            VStack(alignment: .leading, spacing: 6) {
                Text("\(r.method) \(r.url)").font(.mono(12, weight: .semibold)).textSelection(.enabled)
                ForEach(Array(r.requestHeaders.enumerated()), id: \.offset) { _, h in
                    Text("\(h.name): \(h.value)").font(.mono(11)).foregroundStyle(Theme.muted)
                }
                if !r.requestBody.isEmpty { monoBlock(r.requestBody) }
                Divider()
                if let status = r.status {
                    Text("Response \(status) in \(r.timeMs) ms").font(.ui(12, weight: .semibold))
                }
                ForEach(Array(r.responseHeaders.enumerated()), id: \.offset) { _, h in
                    Text("\(h.name): \(h.value)").font(.mono(11)).foregroundStyle(Theme.muted)
                }
                if !r.responseBody.isEmpty { monoBlock(r.responseBody) }
                if let token = r.token {
                    Text("Extracted token").font(.ui(12, weight: .semibold)).foregroundStyle(Theme.green)
                    monoBlock(token)
                }
                if let error = r.error {
                    Text(error).font(.ui(12)).foregroundStyle(Theme.red).textSelection(.enabled)
                }
            }
            .padding(10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .chrome(fill: Theme.response)
        }
    }

    private func monoBlock(_ text: String) -> some View {
        Text(text).font(.mono(11)).textSelection(.enabled)
            .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func sectionTitle(_ text: String) -> some View {
        Text(text.uppercased()).font(.ui(11)).tracking(0.7).foregroundStyle(Theme.muted)
    }

    private func save() {
        do {
            try model.saveServiceSettings(form.toSettings())
            form = ServiceSettingsForm(model.service)
            saveError = nil
        } catch {
            saveError = describe(error)
        }
    }
}
