import SwiftUI

struct SidebarView: View {
    @Bindable var model: WorkspaceModel

    var body: some View {
        List(
            selection: Binding(
                get: { model.selection },
                set: { model.select($0) })
        ) {
            Section {
                if model.endpoints.isEmpty {
                    Text("No endpoints yet. Promote a scratch request to create one.")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
                ForEach(model.endpoints, id: \.id) { endpoint in
                    RequestRow(method: endpoint.spec.method, title: endpoint.name)
                        .tag(SidebarSelection.endpoint(endpoint.id))
                        .contextMenu {
                            Button("Delete…", role: .destructive) {
                                model.pendingDelete = .endpoint(endpoint.id)
                            }
                        }
                }
            } header: {
                Text("Endpoints")
            }

            Section {
                if model.scratch.isEmpty {
                    Text("Scratch requests are local, unversioned, and gitignored.")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
                ForEach(model.scratch, id: \.id) { request in
                    RequestRow(method: request.spec.method, title: request.name)
                        .tag(SidebarSelection.scratch(request.id))
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
            } header: {
                HStack {
                    Text("Scratchpad")
                    Spacer()
                    Button {
                        model.newScratch()
                    } label: {
                        Image(systemName: "plus")
                    }
                    .buttonStyle(.borderless)
                    .help("New Scratch Request (⌘N)")
                }
            }
        }
        .listStyle(.sidebar)
    }
}

struct RequestRow: View {
    let method: String
    let title: String

    var body: some View {
        HStack(spacing: 8) {
            MethodBadge(method: method)
            Text(title.isEmpty ? "Untitled" : title)
                .lineLimit(1)
        }
    }
}

struct MethodBadge: View {
    let method: String

    var body: some View {
        Text(method.uppercased())
            .font(.system(size: 9, weight: .bold, design: .monospaced))
            .foregroundStyle(color)
            .frame(width: 44, alignment: .leading)
    }

    private var color: Color {
        switch method.uppercased() {
        case "GET": return .green
        case "POST": return .orange
        case "PUT", "PATCH": return .blue
        case "DELETE": return .red
        default: return .secondary
        }
    }
}
