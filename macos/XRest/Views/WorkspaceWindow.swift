import SwiftUI

/// Root of one window: welcome screen until a workspace URL is set, then the workspace UI.
struct WorkspaceWindow: View {
    @Binding var url: URL?
    @State private var model: WorkspaceModel?
    @Environment(\.openWindow) private var openWindow
    @State private var errorMessage: String?
    /// A folder that is not a workspace yet; the user is asked whether to initialise it.
    @State private var folderToInitialise: URL?
    @State private var showNewService = false

    var body: some View {
        Group {
            if let model {
                WorkspaceView(model: model)
            } else {
                WelcomeView(open: chooseWorkspace, newFromOpenAPI: { showNewService = true }, openRecent: { url = $0 })
            }
        }
        .focusedSceneValue(\.windowURLSetter, WindowURLSetter { url = $0 })
        .focusedSceneValue(\.newServiceAction, NewServiceAction { showNewService = true })
        .sheet(isPresented: $showNewService) {
            NewServiceSheet { created in
                RecentStore.shared.record(created)
                if model == nil { url = created } else { openWindow(value: created) }
            }
        }
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
            let opened = try WorkspaceModel(url: url)
            opened.attachHistory(HistoryStore.shared)
            model = opened
            RecentStore.shared.record(url)
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
            let created = try WorkspaceModel.create(at: target)
            created.attachHistory(HistoryStore.shared)
            model = created
            RecentStore.shared.record(target)
        } catch {
            errorMessage = describe(error)
            url = nil
        }
    }
}

struct WelcomeView: View {
    let open: () -> Void
    let newFromOpenAPI: () -> Void
    let openRecent: (URL) -> Void
    private var recents: RecentStore { RecentStore.shared }

    var body: some View {
        VStack(spacing: 14) {
            Image(systemName: "paperplane.fill")
                .font(.system(size: 30))
                .foregroundStyle(Theme.amber)
            Text("XRest").font(.ui(20, weight: .semibold))
            Text("One service per workspace. Open a service directory to get started.")
                .font(.ui(13))
                .foregroundStyle(Theme.muted)
            Button(action: open) {
                Text("Open Workspace…")
                    .font(.ui(13, weight: .semibold))
                    .foregroundStyle(Theme.bg)
                    .padding(.horizontal, 20)
                    .frame(height: 36)
                    .background(RoundedRectangle(cornerRadius: 6).fill(Theme.amber))
            }
            .buttonStyle(.plain)
            Button("New Service from OpenAPI…", action: newFromOpenAPI)
                .buttonStyle(.link)
                .font(.ui(12))

            if !recents.items.isEmpty {
                VStack(alignment: .leading, spacing: 2) {
                    Text("RECENT")
                        .font(.ui(11)).tracking(0.7).foregroundStyle(Theme.muted)
                        .padding(.bottom, 4)
                    ForEach(recents.items, id: \.self) { url in
                        Button { openRecent(url) } label: {
                            HStack(spacing: 8) {
                                Text(url.lastPathComponent)
                                    .font(.ui(13)).foregroundStyle(Theme.text)
                                Text(abbreviated(url))
                                    .font(.mono(11)).foregroundStyle(Theme.muted)
                                    .lineLimit(1).truncationMode(.head)
                                Spacer(minLength: 0)
                            }
                            .padding(.horizontal, 8)
                            .frame(height: 26)
                            .contentShape(Rectangle())
                        }
                        .buttonStyle(HoverButtonStyle(radius: 4))
                        .accessibilityLabel("Open \(url.lastPathComponent)")
                    }
                }
                .frame(width: 380)
                .padding(.top, 10)
            }
        }
        .onAppear { recents.refresh() }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .frame(minWidth: 480, minHeight: 320)
        .background(Theme.bg)
        .foregroundStyle(Theme.text)
        .preferredColorScheme(.dark)
        .ignoresSafeArea()
    }

    private func abbreviated(_ url: URL) -> String {
        (url.deletingLastPathComponent().path as NSString).abbreviatingWithTildeInPath
    }
}
