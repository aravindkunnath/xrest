import SwiftUI

/// Git panel: initialize, status, commit, pull / push / sync, auto-commit toggle.
struct GitPanelView: View {
    @Bindable var model: WorkspaceModel
    @State private var remote = ""
    @State private var message = ""

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Git").font(.ui(14, weight: .semibold))
                Spacer()
                if let busy = model.gitBusy {
                    ProgressView().controlSize(.small)
                    Text(busyText(busy)).font(.ui(12)).foregroundStyle(Theme.muted)
                }
                Button("Refresh") { Task { await model.refreshGitStatus() } }
                    .disabled(model.gitBusy != nil)
                Button("Close") { model.closePanel() }
            }
            .padding(.horizontal, 16)
            .frame(height: 40)
            .hairline(.bottom)

            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    if let status = model.gitStatus, status.isRepo {
                        repoContent(status)
                    } else if model.gitStatus != nil {
                        initContent
                    } else {
                        Text("Reading status…").font(.ui(12)).foregroundStyle(Theme.muted)
                    }
                    if let error = model.gitError {
                        Text(error)
                            .font(.mono(11)).foregroundStyle(Theme.red)
                            .textSelection(.enabled)
                            .padding(8)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .chrome(fill: Theme.redFill, stroke: Theme.redBorder, radius: 6)
                    }
                    if let notice = model.gitNotice {
                        Text(notice).font(.ui(12))
                            .padding(8)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .chrome(fill: Theme.amberFill, stroke: Theme.amberBorder, radius: 6)
                    }
                }
                .padding(16)
                .frame(maxWidth: 720, alignment: .leading)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .task { await model.refreshGitStatus() }
    }

    private func busyText(_ a: WorkspaceModel.GitAction) -> String {
        switch a {
        case .initializing: return "Initializing…"
        case .committing: return "Committing…"
        case .pulling: return "Pulling…"
        case .pushing: return "Pushing…"
        case .syncing: return "Syncing…"
        }
    }

    private var initContent: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("This workspace is not a Git repository.").font(.ui(13))
            TextField("Remote URL (optional)", text: $remote).textFieldStyle(.roundedBorder)
                .frame(maxWidth: 420)
            Button("Initialize") { Task { await model.gitInit(remote: remote) } }
                .disabled(model.gitBusy != nil)
        }
    }

    @ViewBuilder
    private func repoContent(_ status: GitStatus) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            row("Branch", status.branch)
            row("Remote", status.remoteUrl ?? "None")
            row("Sync", GitFormat.syncSummary(ahead: status.ahead, behind: status.behind))
            row("Last sync", GitFormat.lastSync(status.lastSync))
        }
        Toggle(
            "Auto-commit saves",
            isOn: Binding(get: { model.autoCommit }, set: { model.setAutoCommit($0) }))
        VStack(alignment: .leading, spacing: 6) {
            Text("CHANGES (\(status.uncommitted.count))").font(.ui(11, weight: .semibold))
                .foregroundStyle(Theme.muted)
            if status.uncommitted.isEmpty {
                Text("Nothing to commit.").font(.ui(12)).foregroundStyle(Theme.muted)
            }
            ForEach(status.uncommitted, id: \.path) { f in
                HStack(spacing: 8) {
                    Text(GitFormat.statusLabel(f.status)).font(.ui(11)).foregroundStyle(Theme.muted)
                        .frame(width: 64, alignment: .leading)
                    Text(f.path).font(.mono(12))
                }
            }
        }
        if !status.uncommitted.isEmpty {
            HStack {
                TextField(
                    GitFormat.defaultCommitMessage(status.uncommitted), text: $message
                )
                .textFieldStyle(.roundedBorder)
                Button("Commit") {
                    Task {
                        await model.gitCommit(message: message)
                        if model.gitError == nil { message = "" }
                    }
                }
                .disabled(model.gitBusy != nil)
            }
        }
        HStack {
            Button("Pull") { Task { await model.gitPull() } }
            Button("Push") { Task { await model.gitPush() } }
            Button("Sync") { Task { await model.gitSync() } }
        }
        .disabled(model.gitBusy != nil || status.remoteUrl == nil)
        if status.remoteUrl == nil {
            Text("Add a remote to pull and push.").font(.ui(11)).foregroundStyle(Theme.muted)
        }
    }

    private func row(_ label: String, _ value: String) -> some View {
        HStack(alignment: .top) {
            Text(label).font(.ui(12)).foregroundStyle(Theme.muted).frame(width: 80, alignment: .leading)
            Text(value).font(.ui(12)).textSelection(.enabled)
        }
    }
}
