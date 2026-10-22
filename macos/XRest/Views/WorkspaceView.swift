import SwiftUI

struct WorkspaceView: View {
    @Bindable var model: WorkspaceModel

    var body: some View {
        NavigationSplitView {
            SidebarView(model: model)
                .navigationSplitViewColumnWidth(min: 200, ideal: 260, max: 420)
        } detail: {
            if model.draft != nil {
                RequestDetailView(model: model)
            } else {
                ContentUnavailableView(
                    "No Request Selected", systemImage: "paperplane",
                    description: Text("Pick a request in the sidebar, or press ⌘N for a new scratch request."))
            }
        }
        .navigationTitle(model.service.name)
        .navigationSubtitle(model.selectedEnvironment ?? "No environment")
        .toolbar {
            ToolbarItem(placement: .automatic) {
                EnvironmentPicker(model: model)
            }
        }
        .focusedSceneValue(\.workspaceModel, model)
        .alert(
            "Send to unsafe environment?", isPresented: $model.showUnsafeConfirmation
        ) {
            Button("Cancel", role: .cancel) {}
            Button("Send to \(model.selectedEnvironment ?? "environment")", role: .destructive) {
                model.confirmSend()
            }
        } message: {
            Text(
                "\(model.selectedEnvironment ?? "This environment") is marked unsafe. This request will be sent for real."
            )
        }
        .alert("Error", isPresented: errorBinding) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(model.errorMessage ?? "")
        }
        .alert(
            "Delete request?", isPresented: deleteBinding
        ) {
            Button("Cancel", role: .cancel) {}
            Button("Delete", role: .destructive) {
                if let target = model.pendingDelete { model.delete(target) }
            }
        } message: {
            Text("This removes the file from the workspace.")
        }
        .sheet(isPresented: $model.showImportCurl) {
            ImportCurlSheet(model: model)
        }
    }

    private var errorBinding: Binding<Bool> {
        Binding(
            get: { model.errorMessage != nil },
            set: { if !$0 { model.errorMessage = nil } })
    }

    private var deleteBinding: Binding<Bool> {
        Binding(
            get: { model.pendingDelete != nil },
            set: { if !$0 { model.pendingDelete = nil } })
    }
}

/// Toolbar environment picker. Unsafe environments carry a warning marker and the picker turns red
/// while one is active.
struct EnvironmentPicker: View {
    @Bindable var model: WorkspaceModel

    var body: some View {
        HStack(spacing: 6) {
            Picker(
                "Environment",
                selection: Binding(
                    get: { model.selectedEnvironment ?? "" },
                    set: { model.selectEnvironment($0) })
            ) {
                ForEach(model.environments, id: \.name) { env in
                    Text(env.isUnsafe ? "⚠︎ \(env.name)" : env.name).tag(env.name)
                }
            }
            .pickerStyle(.menu)
            .frame(minWidth: 130)

            if model.selectedEnvironmentIsUnsafe {
                Text("UNSAFE")
                    .font(.caption2.weight(.bold))
                    .foregroundStyle(.white)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(Capsule().fill(Color.red))
            }
        }
        .help("Environment used to resolve {{VARIABLES}} and to decide whether sends need confirmation")
    }
}

struct ImportCurlSheet: View {
    @Bindable var model: WorkspaceModel
    @State private var command = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Import cURL").font(.headline)
            Text("Paste a cURL command. It becomes a new scratch request.")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            CodeEditor(text: $command)
                .frame(minHeight: 160)
                .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.secondary.opacity(0.4)))
            HStack {
                Spacer()
                Button("Cancel", role: .cancel) { model.showImportCurl = false }
                    .keyboardShortcut(.cancelAction)
                Button("Import") { model.importCurl(command) }
                    .keyboardShortcut(.defaultAction)
                    .disabled(command.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
        .padding(20)
        .frame(width: 560)
    }
}
