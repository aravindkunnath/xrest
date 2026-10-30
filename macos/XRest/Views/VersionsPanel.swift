import SwiftUI

/// "History" tab of a saved endpoint: versions newest first, the field diff of the selected one
/// against its predecessor, and "Restore this version".
struct VersionsPanel: View {
    @Bindable var model: WorkspaceModel

    private var currentVersion: Int32 {
        if case .endpoint(let id)? = model.selection {
            return model.endpoints.first { $0.id == id }?.lastVersion ?? 0
        }
        return 0
    }

    private var selectionKey: String {
        if case .endpoint(let id)? = model.selection { return "\(id)@\(currentVersion)" }
        return ""
    }

    var body: some View {
        VStack(spacing: 0) {
            if model.versionRows.isEmpty {
                Text("No versions yet.")
                    .font(.ui(12)).foregroundStyle(Theme.muted)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(model.versionRows) { row in
                            VersionRowView(
                                row: row, isCurrent: row.version == currentVersion,
                                isSelected: row.version == model.selectedVersion
                            ) { model.selectVersion(row.version) }
                            if row.version == model.selectedVersion {
                                detail(row)
                            }
                        }
                    }
                }
            }
        }
        .frame(maxHeight: .infinity, alignment: .top)
        .task(id: selectionKey) { model.loadVersions() }
        .confirmationDialog(
            "Restore version \(model.pendingRestore.map(String.init) ?? "")?",
            isPresented: Binding(
                get: { model.pendingRestore != nil },
                set: { if !$0 { model.cancelRestore() } }),
            titleVisibility: .visible
        ) {
            Button("Restore") { model.confirmRestore() }
            Button("Cancel", role: .cancel) { model.cancelRestore() }
        } message: {
            Text(
                "This saves it as a new version. Unsaved edits are saved first, and no history is lost."
            )
        }
    }

    @ViewBuilder
    private func detail(_ row: VersionRow) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            if model.diffRows.isEmpty {
                Text(row.summary == "Initial version" ? "First saved version." : "No field changes.")
                    .font(.ui(12)).foregroundStyle(Theme.muted)
            } else {
                ForEach(model.diffRows) { DiffRowView(row: $0) }
            }
            if row.version != currentVersion {
                Button("Restore this version") { model.requestRestore(row.version) }
                    .buttonStyle(.plain)
                    .font(.ui(12))
                    .foregroundStyle(Theme.amber)
                    .padding(.horizontal, 10)
                    .frame(height: 26)
                    .chrome(fill: Theme.amberFill, stroke: Theme.amberBorder, radius: 4)
                    .padding(.top, 4)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Theme.panel)
        .hairline(.bottom, color: Theme.rowDivider)
    }
}

private struct VersionRowView: View {
    let row: VersionRow
    let isCurrent: Bool
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                Text("v\(row.version)")
                    .font(.mono(12, weight: .semibold))
                    .foregroundStyle(isCurrent ? Theme.amber : Theme.textSoft)
                    .frame(width: 34, alignment: .leading)
                Text(row.summary)
                    .font(.mono(12)).foregroundStyle(Theme.text)
                    .lineLimit(1).truncationMode(.tail)
                    .frame(maxWidth: .infinity, alignment: .leading)
                if isCurrent {
                    Text("current").font(.ui(11)).foregroundStyle(Theme.muted)
                }
                Text(Date(timeIntervalSince1970: TimeInterval(row.timestamp)), style: .relative)
                    .font(.ui(11)).foregroundStyle(Theme.muted)
            }
            .padding(.horizontal, 16)
            .frame(height: 30)
            .background(isSelected ? Theme.selected : .clear)
            .contentShape(Rectangle())
            .hairline(.bottom, color: Theme.rowDivider)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Version \(row.version), \(row.summary)")
        .accessibilityAddTraits(isSelected ? .isSelected : [])
    }
}

private struct DiffRowView: View {
    let row: DiffRow

    private var color: Color {
        switch row.kind {
        case .added: return Theme.green
        case .removed: return Theme.red
        case .changed: return Theme.amber
        }
    }

    private var symbol: String {
        switch row.kind {
        case .added: return "+"
        case .removed: return "−"
        case .changed: return "~"
        }
    }

    private var valueText: String {
        switch row.kind {
        case .added: return row.after ?? ""
        case .removed: return row.before ?? ""
        case .changed: return "\(row.before ?? "")  →  \(row.after ?? "")"
        }
    }

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Text(symbol).font(.mono(12, weight: .bold)).foregroundStyle(color).frame(width: 12)
            Text(row.field).font(.mono(12)).foregroundStyle(Theme.textSoft)
            Text(valueText)
                .font(.mono(12)).foregroundStyle(color)
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .accessibilityElement(children: .combine)
    }
}
