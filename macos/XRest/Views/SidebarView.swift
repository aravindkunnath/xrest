import SwiftUI

/// Narrow icon rail on the far left. Only Explorer is wired up so far.
struct ActivityRail: View {
    let openSecrets: () -> Void
    let openEnvironments: () -> Void
    let openSettings: () -> Void
    let openGit: () -> Void
    let openHistory: () -> Void
    let gitBadge: Bool

    private struct Item: Identifiable {
        let id: String
        let symbol: String
        var active = false
        var enabled = false
    }

    private let items = [
        Item(id: "Explorer", symbol: "line.3.horizontal", active: true),
        Item(id: "History", symbol: "clock", enabled: true),
        Item(id: "Environments and variables", symbol: "curlybraces", enabled: true),
        Item(id: "Git", symbol: "arrow.triangle.branch", enabled: true),
        Item(id: "Secrets", symbol: "key", enabled: true),
    ]

    var body: some View {
        VStack(spacing: 6) {
            ForEach(items) { item in
                railButton(item.id, symbol: item.symbol, active: item.active, enabled: item.enabled)
            }
            Spacer()
            railButton("Settings", symbol: "gearshape", active: false, enabled: true)
        }
        .padding(.vertical, 10)
        .frame(width: 48)
        .frame(maxHeight: .infinity)
        .background(Theme.panel)
        .hairline(.trailing)
    }

    private func railButton(_ label: String, symbol: String, active: Bool, enabled: Bool = false)
        -> some View
    {
        Button {
            switch label {
            case "Secrets": openSecrets()
            case "Environments and variables": openEnvironments()
            case "Settings": openSettings()
            case "Git": openGit()
            case "History": openHistory()
            default: break
            }
        } label: {
            Image(systemName: symbol)
                .font(.system(size: 15))
                .foregroundStyle(active ? Theme.amber : Theme.muted)
                .frame(width: 34, height: 34)
                .background(RoundedRectangle(cornerRadius: 8).fill(active ? Theme.selected : .clear))
                .overlay(alignment: .topTrailing) {
                    if label == "Git" && gitBadge {
                        Circle().fill(Theme.amber).frame(width: 8, height: 8).padding(5)
                    }
                }
        }
        .buttonStyle(HoverButtonStyle(radius: 8))
        .accessibilityLabel(label == "Git" && gitBadge ? "Git, uncommitted or unpushed changes" : label)
        .help(active || enabled ? label : "\(label) (coming soon)")
    }
}

struct SidebarView: View {
    @Bindable var model: WorkspaceModel

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                VStack(spacing: 0) {
                    endpointsSection
                    scratchSection
                }
            }
            .scrollContentBackground(.hidden)
        }
        .frame(maxHeight: .infinity)
        .background(Theme.sidebar)
        .hairline(.trailing)
    }

    private var query: String { model.searchText.trimmingCharacters(in: .whitespaces) }

    private func matches(_ name: String, _ method: String) -> Bool {
        query.isEmpty
            || name.localizedCaseInsensitiveContains(query)
            || method.localizedCaseInsensitiveContains(query)
    }

    // MARK: Endpoints

    private var endpointsSection: some View {
        let shown = model.endpoints.filter { matches($0.name, $0.spec.method) }
        return VStack(spacing: 0) {
            sectionHeader("ENDPOINTS") {
                iconButton("plus", help: "New request (starts as a scratch request)") {
                    model.newScratch()
                }
                iconButton("square.and.arrow.down", help: "Import cURL (⇧⌘I)") {
                    model.showImportCurl = true
                }
            }
            VStack(spacing: 0) {
                if model.endpoints.isEmpty {
                    hint("No endpoints yet. Promote a scratch request to create one.")
                } else if shown.isEmpty {
                    hint("No endpoints match “\(query)”.")
                }
                ForEach(shown, id: \.id) { endpoint in
                    EndpointRow(
                        method: endpoint.spec.method, title: endpoint.name,
                        version: "v\(endpoint.lastVersion)",
                        selected: model.selection == .endpoint(endpoint.id)
                    ) {
                        model.select(.endpoint(endpoint.id))
                    }
                    .contextMenu {
                        Button("Delete…", role: .destructive) {
                            model.pendingDelete = .endpoint(endpoint.id)
                        }
                    }
                }
            }
            .padding(.horizontal, 6)
        }
    }

    // MARK: Scratchpad

    private var scratchSection: some View {
        let shown = model.scratch.filter { matches($0.name, $0.spec.method) }
        return VStack(spacing: 0) {
            HStack {
                sectionTitle("SCRATCHPAD")
                Spacer()
                Tag(text: "local · not in git")
            }
            .padding(.horizontal, 12)
            .padding(.top, 18)
            .padding(.bottom, 6)
            .hairline(.top)
            .padding(.top, 8)

            VStack(spacing: 0) {
                if model.scratch.isEmpty {
                    hint("Scratch requests are local, unversioned, and gitignored.")
                } else if shown.isEmpty {
                    hint("No scratch requests match “\(query)”.")
                }
                ForEach(shown, id: \.id) { request in
                    ScratchRow(
                        method: request.spec.method, title: request.name,
                        selected: model.selection == .scratch(request.id),
                        select: { model.select(.scratch(request.id)) },
                        promote: {
                            model.select(.scratch(request.id))
                            model.promoteSelected()
                        }
                    )
                    .contextMenu {
                        Button("Promote to Endpoint") {
                            model.select(.scratch(request.id))
                            model.promoteSelected()
                        }
                        Divider()
                        Button("Delete…", role: .destructive) {
                            model.pendingDelete = .scratch(request.id)
                        }
                    }
                }
                Button {
                    model.newScratch()
                } label: {
                    Text("+ New scratch request")
                        .font(.ui(13))
                        .foregroundStyle(Theme.muted)
                        .padding(.horizontal, 10)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .frame(height: 30)
                        .overlay(
                            RoundedRectangle(cornerRadius: 6)
                                .strokeBorder(
                                    Theme.controlOffStrong,
                                    style: StrokeStyle(lineWidth: 1, dash: [4, 3]))
                        )
                }
                .buttonStyle(.plain)
                .padding(.horizontal, 2)
                .padding(.vertical, 6)
            }
            .padding(.horizontal, 6)
        }
    }

    // MARK: Pieces

    private func sectionTitle(_ text: String) -> some View {
        Text(text)
            .font(.ui(11, weight: .semibold))
            .tracking(0.9)
            .foregroundStyle(Theme.muted)
    }

    private func sectionHeader<Actions: View>(
        _ title: String, @ViewBuilder actions: () -> Actions
    ) -> some View {
        HStack {
            sectionTitle(title)
            Spacer()
            HStack(spacing: 2, content: actions)
        }
        .padding(.horizontal, 12)
        .padding(.top, 12)
        .padding(.bottom, 6)
    }

    private func iconButton(_ symbol: String, help: String, action: @escaping () -> Void)
        -> some View
    {
        Button(action: action) {
            Image(systemName: symbol)
                .font(.system(size: 11, weight: .medium))
                .foregroundStyle(Theme.muted)
                .frame(width: 24, height: 24)
        }
        .buttonStyle(HoverButtonStyle(radius: 4))
        .help(help)
        .accessibilityLabel(help)
    }

    private func hint(_ text: String) -> some View {
        Text(text)
            .font(.ui(12))
            .foregroundStyle(Theme.muted)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 8)
            .padding(.vertical, 6)
    }
}

struct MethodBadge: View {
    let method: String

    var body: some View {
        Text(method.uppercased())
            .font(.mono(10, weight: .semibold))
            .foregroundStyle(Theme.methodColor(method))
            .frame(width: 46, alignment: .leading)
    }
}

struct EndpointRow: View {
    let method: String
    let title: String
    let version: String
    let selected: Bool
    let action: () -> Void

    var body: some View {
        let metrics = DensityMetrics(PreferencesStore.shared.density)
        return Button(action: action) {
            HStack(spacing: 8) {
                MethodBadge(method: method)
                Text(title.isEmpty ? "Untitled" : title)
                    .font(.ui(13))
                    .foregroundStyle(Theme.text)
                    .lineLimit(1)
                    .truncationMode(.tail)
                    .frame(maxWidth: .infinity, alignment: .leading)
                Text(version)
                    .font(.mono(10))
                    .foregroundStyle(Theme.muted)
            }
            .padding(.horizontal, 8)
            .padding(.vertical, metrics.sidebarRowPadding)
            .background(RoundedRectangle(cornerRadius: 6).fill(selected ? Theme.selected : .clear))
            .contentShape(Rectangle())
        }
        .buttonStyle(HoverButtonStyle(radius: 6))
    }
}

struct ScratchRow: View {
    let method: String
    let title: String
    let selected: Bool
    let select: () -> Void
    let promote: () -> Void
    @State private var hovering = false

    var body: some View {
        let metrics = DensityMetrics(PreferencesStore.shared.density)
        return HStack(spacing: 8) {
            MethodBadge(method: method)
            Text(title.isEmpty ? "Untitled" : title)
                .font(.ui(13).italic())
                .foregroundStyle(Theme.textSoft)
                .lineLimit(1)
                .truncationMode(.tail)
                .frame(maxWidth: .infinity, alignment: .leading)
            if hovering {
                Button(action: promote) {
                    Text("Promote ↑")
                        .font(.ui(11))
                        .foregroundStyle(Theme.amber)
                        .padding(.horizontal, 7)
                        .frame(height: 22)
                        .chrome(fill: Theme.amberFill, stroke: Theme.amberBorder, radius: 4)
                }
                .buttonStyle(.plain)
                .accessibilityLabel("Promote to endpoint")
            }
        }
        .padding(.horizontal, 8)
        .padding(.vertical, metrics.sidebarRowPadding)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(selected ? Theme.selected : (hovering ? Theme.hover : .clear))
        )
        .contentShape(Rectangle())
        .onTapGesture(perform: select)
        .onHover { hovering = $0 }
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits(.isButton)
    }
}
