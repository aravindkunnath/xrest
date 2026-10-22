import SwiftUI

/// Root of one window: welcome screen until a workspace URL is set, then the workspace UI.
struct WorkspaceWindow: View {
    @Binding var url: URL?
    @State private var model: WorkspaceModel?
    @State private var errorMessage: String?
    /// A folder that is not a workspace yet; the user is asked whether to initialise it.
    @State private var folderToInitialise: URL?

    var body: some View {
        Group {
            if let model {
                WorkspaceView(model: model)
            } else {
                WelcomeView(open: chooseWorkspace)
            }
        }
        .focusedSceneValue(\.windowURLSetter, WindowURLSetter { url = $0 })
        .task(id: url) { load() }
        .alert("Could not open workspace", isPresented: errorBinding) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage ?? "")
        }
        .alert("Start a new workspace here?", isPresented: initialiseBinding) {
            Button("Create Workspace") { initialise() }
            Button("Cancel", role: .cancel) { url = nil }
        } message: {
            Text(
                "\(folderToInitialise?.lastPathComponent ?? "This folder") has no service.yaml. XRest can create a new workspace (service) in it."
            )
        }
    }

    private var errorBinding: Binding<Bool> {
        Binding(get: { errorMessage != nil }, set: { if !$0 { errorMessage = nil } })
    }

    private var initialiseBinding: Binding<Bool> {
        Binding(get: { folderToInitialise != nil }, set: { if !$0 { folderToInitialise = nil } })
    }

    private func chooseWorkspace() {
        if let picked = WorkspacePicker.chooseDirectory() { url = picked }
    }

    private func load() {
        guard let url else {
            model = nil
            return
        }
        do {
            model = try WorkspaceModel(url: url)
        } catch XRestError.InvalidInput {
            // Not a workspace: offer to create one if the folder looks usable.
            model = nil
            folderToInitialise = url
        } catch {
            model = nil
            errorMessage = describe(error)
            self.url = nil
        }
    }

    private func initialise() {
        guard let target = folderToInitialise else { return }
        folderToInitialise = nil
        do {
            model = try WorkspaceModel.create(at: target)
        } catch {
            errorMessage = describe(error)
            url = nil
        }
    }
}

struct WelcomeView: View {
    let open: () -> Void

    var body: some View {
        ContentUnavailableView {
            Label("XRest", systemImage: "paperplane.fill")
        } description: {
            Text("One service per workspace. Open a service directory to get started.")
        } actions: {
            Button("Open Workspace…", action: open)
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
        }
        .frame(minWidth: 480, minHeight: 320)
    }
}
