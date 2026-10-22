import SwiftUI

private enum EditorTab: String, CaseIterable, Identifiable {
    case params = "Params"
    case headers = "Headers"
    case body = "Body"
    case options = "Options"
    var id: String { rawValue }
}

struct RequestDetailView: View {
    @Bindable var model: WorkspaceModel
    @State private var tab: EditorTab = .params

    /// Non-optional view of the draft for bindings; the parent only shows this view when set.
    private var draft: Binding<RequestDraft> {
        Binding(
            get: { model.draft ?? RequestDraft.blank },
            set: { model.draft = $0 })
    }

    var body: some View {
        VStack(spacing: 0) {
            if model.selectedEnvironmentIsUnsafe {
                UnsafeBanner(environment: model.selectedEnvironment ?? "")
            }
            header
            Divider()
            VSplitView {
                editor
                    .frame(minHeight: 160)
                ResponseView(model: model)
                    .frame(minHeight: 160)
            }
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                TextField("Name", text: draft.name)
                    .textFieldStyle(.plain)
                    .font(.title3.weight(.semibold))
                if model.isDirty {
                    Text("Edited")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Toggle("Use service auth", isOn: draft.authenticated)
                    .toggleStyle(.checkbox)
                    .help("Inject the service-level authentication into this request")
                    .disabled(!model.service.isAuthenticated)
                if model.isScratchSelected {
                    Button("Promote to Endpoint") { model.promoteSelected() }
                        .help("Move this scratch request into the versioned endpoints (⇧⌘P)")
                } else {
                    Label("Endpoint", systemImage: "tag")
                        .labelStyle(.titleAndIcon)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            HStack(spacing: 8) {
                Picker("Method", selection: draft.method) {
                    ForEach(RequestDraft.methods, id: \.self) { Text($0).tag($0) }
                }
                .labelsHidden()
                .frame(width: 110)

                TextField("https://… or {{BASE_URL}}/path", text: draft.url)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.body, design: .monospaced))
                    .onSubmit { model.requestSend() }

                Button {
                    model.requestSend()
                } label: {
                    if model.isSending {
                        ProgressView().controlSize(.small)
                    } else {
                        Text("Send")
                    }
                }
                .buttonStyle(.borderedProminent)
                .tint(model.selectedEnvironmentIsUnsafe ? .red : .accentColor)
                .disabled(model.isSending)
                .help("Send (⌘↩)")
            }
        }
        .padding(12)
    }

    private var editor: some View {
        VStack(spacing: 0) {
            Picker("Section", selection: $tab) {
                ForEach(EditorTab.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .labelsHidden()
            .frame(maxWidth: 420)
            .padding(.vertical, 8)

            switch tab {
            case .params:
                KeyValueEditor(rows: draft.params, namePlaceholder: "Parameter")
            case .headers:
                KeyValueEditor(rows: draft.headers, namePlaceholder: "Header")
            case .body:
                BodyEditor(draft: draft, workspaceRoot: model.service.rootPath)
            case .options:
                OptionsEditor(options: draft.options)
            }
        }
    }
}

struct UnsafeBanner: View {
    let environment: String

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: "exclamationmark.triangle.fill")
            Text("\(environment) is an unsafe environment. Sends need confirmation.")
                .font(.callout.weight(.medium))
            Spacer()
        }
        .foregroundStyle(.white)
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color.red)
    }
}
