import AppKit
import SwiftUI

/// Presents the standard folder picker used to open (or initialise) a workspace.
@MainActor
enum WorkspacePicker {
    static func chooseDirectory() -> URL? {
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.canCreateDirectories = true
        panel.allowsMultipleSelection = false
        panel.prompt = "Open Workspace"
        panel.message =
            "Choose a service directory (containing service.yaml), or an empty folder to start a new workspace."
        return panel.runModal() == .OK ? panel.url : nil
    }
}

struct WorkspaceCommands: Commands {
    @FocusedValue(\.workspaceModel) private var model
    @FocusedValue(\.windowURLSetter) private var urlSetter
    @Environment(\.openWindow) private var openWindow

    var body: some Commands {
        // Replaces File > New Window: ⌘N creates a scratch request.
        CommandGroup(replacing: .newItem) {
            Button("New Scratch Request") { model?.newScratch() }
                .keyboardShortcut("n", modifiers: .command)
                .disabled(model == nil)
            Button("Import cURL…") { model?.showImportCurl = true }
                .keyboardShortcut("i", modifiers: [.command, .shift])
                .disabled(model == nil)
            Divider()
            Button("Open Workspace…") { openWorkspace() }
                .keyboardShortcut("o", modifiers: .command)
            Menu("Open Recent") {
                ForEach(RecentStore.shared.items, id: \.self) { url in
                    Button(url.lastPathComponent) { open(url) }
                        .help(url.path)
                }
                Divider()
                Button("Clear Menu") { RecentStore.shared.clear() }
                    .disabled(RecentStore.shared.items.isEmpty)
            }
            .disabled(RecentStore.shared.items.isEmpty)
        }

        CommandGroup(replacing: .saveItem) {
            Button("Save") { model?.saveActive() }
                .keyboardShortcut("s", modifiers: .command)
                .disabled(model == nil)
        }

        CommandMenu("Request") {
            Button("Send") { model?.requestSend() }
                .keyboardShortcut(.return, modifiers: .command)
                .disabled(model == nil)
            Button("Promote to Endpoint") { model?.promoteSelected() }
                .keyboardShortcut("p", modifiers: [.command, .shift])
                .disabled(model == nil)
            Button("Secrets…") { model?.openSecrets() }
                .keyboardShortcut("k", modifiers: [.command, .shift])
                .disabled(model == nil)
            Divider()
            Menu("Switch Environment") {
                if let model {
                    ForEach(Array(model.environments.enumerated()), id: \.element.name) { index, env in
                        Button {
                            model.selectEnvironment(env.name)
                        } label: {
                            Text(menuTitle(env, selected: env.name == model.selectedEnvironment))
                        }
                        .keyboardShortcut(
                            KeyEquivalent(Character(String(min(index + 1, 9)))),
                            modifiers: [.command, .control])
                    }
                }
            }
            .disabled(model == nil)
        }
    }

    private func menuTitle(_ env: EnvironmentConfig, selected: Bool) -> String {
        let mark = selected ? "✓ " : ""
        return env.isUnsafe ? "\(mark)\(env.name)  (unsafe)" : "\(mark)\(env.name)"
    }

    private func openWorkspace() {
        guard let url = WorkspacePicker.chooseDirectory() else { return }
        open(url)
    }

    private func open(_ url: URL) {
        if model == nil, let urlSetter {
            urlSetter.set(url)
        } else {
            openWindow(value: url)
        }
    }
}
