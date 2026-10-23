import AppKit
import SwiftUI

struct WorkspaceView: View {
    @Bindable var model: WorkspaceModel

    var body: some View {
        VStack(spacing: 0) {
            TitleBar(model: model)
            HStack(spacing: 0) {
                ActivityRail(
                    openSecrets: { model.openSecrets() },
                    openEnvironments: { model.openPanel(.environments) },
                    openSettings: { model.openPanel(.settings) })
                SidebarView(model: model)
                    .frame(width: 280)
                MainColumn(model: model)
            }
            StatusBar(model: model)
        }
        .frame(minWidth: 1000, minHeight: 600)
        .background(Theme.bg)
        .foregroundStyle(Theme.text)
        .font(.ui(13))
        .preferredColorScheme(.dark)
        .tint(Theme.amber)
        .ignoresSafeArea()
        .navigationTitle(model.service.name)
        .focusedSceneValue(\.workspaceModel, model)
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
        .sheet(item: $model.pendingConfirmation) { pending in
            UnsafeConfirmationSheet(pending: pending, dismiss: { model.pendingConfirmation = nil })
        }
        .sheet(isPresented: $model.showSecrets) {
            SecretsSheet(model: model)
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

/// Title bar drawn by the app (the window uses a hidden title bar): workspace switcher,
/// environment switcher and the search box.
struct TitleBar: View {
    @Bindable var model: WorkspaceModel
    @Environment(\.openWindow) private var openWindow
    @FocusState private var searchFocused: Bool

    var body: some View {
        HStack(spacing: 10) {
            // Room for the traffic-light buttons.
            Color.clear.frame(width: 70, height: 1)
            workspaceMenu
            environmentMenu
            Spacer(minLength: 40)
            search
        }
        .padding(.horizontal, 12)
        .frame(height: 40)
        .background(Theme.panel)
        .hairline(.bottom)
    }

    private var workspaceMenu: some View {
        Menu {
            Button("Open Workspace…") {
                if let url = WorkspacePicker.chooseDirectory() { openWindow(value: url) }
            }
            Button("Reveal in Finder") {
                NSWorkspace.shared.activateFileViewerSelecting([model.rootURL])
            }
        } label: {
            HStack(spacing: 8) {
                Text(String(model.service.name.prefix(1)).uppercased())
                    .font(.ui(11, weight: .bold))
                    .foregroundStyle(Theme.bg)
                    .frame(width: 18, height: 18)
                    .background(RoundedRectangle(cornerRadius: 4).fill(Theme.amber))
                Text(model.service.name)
                    .font(.ui(13, weight: .semibold))
                    .foregroundStyle(Theme.text)
                chevron(Theme.muted)
            }
            .padding(.horizontal, 10)
            .frame(height: 28)
            .chrome()
        }
        .menuStyle(.button)
        .buttonStyle(.plain)
        .menuIndicator(.hidden)
        .fixedSize()
        .accessibilityLabel("Switch workspace")
    }

    private var environmentMenu: some View {
        let unsafe = model.selectedEnvironmentIsUnsafe
        let tint = unsafe ? Theme.red : Theme.lightGreen
        return Menu {
            ForEach(model.environments, id: \.name) { env in
                Button(env.isUnsafe ? "⚠︎ \(env.name)" : env.name) {
                    model.selectEnvironment(env.name)
                }
            }
        } label: {
            HStack(spacing: 6) {
                Circle()
                    .fill(unsafe ? Theme.red : Theme.green)
                    .frame(width: 7, height: 7)
                Text(model.selectedEnvironment ?? "No environment")
                    .font(.mono(12))
                if unsafe {
                    Text("UNSAFE").font(.ui(9, weight: .bold))
                }
                chevron(tint)
            }
            .foregroundStyle(tint)
            .padding(.horizontal, 10)
            .frame(height: 28)
            .chrome(
                fill: unsafe ? Theme.redFill : Theme.greenFill,
                stroke: unsafe ? Theme.redBorder : Theme.greenBorder)
        }
        .menuStyle(.button)
        .buttonStyle(.plain)
        .menuIndicator(.hidden)
        .fixedSize()
        .help("Environment used to resolve {{VARIABLES}} and to decide whether sends need confirmation")
        .accessibilityLabel("Environment: \(model.selectedEnvironment ?? "none")")
    }

    private var search: some View {
        HStack(spacing: 10) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 12))
                .foregroundStyle(Theme.muted)
            TextField("Search endpoints and scratch", text: $model.searchText)
                .textFieldStyle(.plain)
                .font(.ui(13))
                .foregroundStyle(Theme.text)
                .focused($searchFocused)
            Text("⌘K")
                .font(.mono(11))
                .foregroundStyle(Theme.muted)
        }
        .padding(.horizontal, 10)
        .frame(width: 300, height: 28)
        .chrome(fill: Theme.bg, stroke: Theme.border)
        .background(
            Button("") { searchFocused = true }
                .keyboardShortcut("k", modifiers: .command)
                .opacity(0)
        )
    }

    private func chevron(_ color: Color) -> some View {
        Image(systemName: "chevron.down")
            .font(.system(size: 8, weight: .bold))
            .foregroundStyle(color)
    }
}

/// The open request's tab, and the empty state when nothing is open.
struct MainColumn: View {
    @Bindable var model: WorkspaceModel

    var body: some View {
        VStack(spacing: 0) {
            if model.panel == nil { tabStrip }
            if model.panel == .environments {
                EnvironmentsEditorView(model: model)
            } else if model.panel == .settings {
                ServiceSettingsView(model: model)
            } else if model.draft != nil {
                RequestDetailView(model: model)
            } else {
                VStack(spacing: 8) {
                    Image(systemName: "paperplane")
                        .font(.system(size: 28))
                        .foregroundStyle(Theme.muted)
                    Text("No request selected")
                        .font(.ui(14, weight: .medium))
                    Text("Pick a request in the sidebar, or press ⌘N for a new scratch request.")
                        .font(.ui(12))
                        .foregroundStyle(Theme.muted)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.bg)
    }

    private var tabStrip: some View {
        HStack(spacing: 0) {
            if let draft = model.draft {
                HStack(spacing: 8) {
                    Text(draft.method)
                        .font(.mono(10, weight: .semibold))
                        .foregroundStyle(Theme.methodColor(draft.method))
                    Text(draft.name.isEmpty ? "Untitled" : draft.name)
                        .font(model.isScratchSelected ? .ui(13).italic() : .ui(13))
                        .lineLimit(1)
                    if model.isScratchSelected {
                        Tag(text: "scratch")
                    }
                    if model.isDirty {
                        Circle().fill(Theme.amber).frame(width: 6, height: 6)
                            .accessibilityLabel("Unsaved")
                    }
                }
                .padding(.horizontal, 14)
                .frame(maxHeight: .infinity)
                .background(Theme.bg)
                .overlay(alignment: .top) { Rectangle().fill(Theme.amber).frame(height: 2) }
                .hairline(.trailing)
            }
            Button {
                model.newScratch()
            } label: {
                Image(systemName: "plus")
                    .font(.system(size: 12))
                    .foregroundStyle(Theme.muted)
                    .frame(width: 38)
                    .frame(maxHeight: .infinity)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .help("New scratch request (⌘N)")
            .accessibilityLabel("New tab")
            Spacer(minLength: 0)
        }
        .frame(height: 38)
        .background(Theme.panel)
        .hairline(.bottom)
    }
}

struct StatusBar: View {
    let model: WorkspaceModel

    var body: some View {
        HStack(spacing: 16) {
            Label(model.service.name, systemImage: "shippingbox")
                .labelStyle(.titleAndIcon)
                .foregroundStyle(Theme.textSoft)
            if !model.warnings.isEmpty {
                Text("● \(model.warnings.count) warning\(model.warnings.count == 1 ? "" : "s")")
                    .foregroundStyle(Theme.amber)
                    .help(model.warnings.joined(separator: "\n"))
            }
            Spacer()
            Text(abbreviatedPath)
                .lineLimit(1)
                .truncationMode(.head)
        }
        .font(.ui(12))
        .foregroundStyle(Theme.muted)
        .padding(.horizontal, 12)
        .frame(height: 28)
        .background(Theme.panel)
        .hairline(.top)
    }

    private var abbreviatedPath: String {
        (model.rootURL.path as NSString).abbreviatingWithTildeInPath
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
