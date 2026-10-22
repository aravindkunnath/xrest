import SwiftUI

private enum ResponseTab: String, CaseIterable, Identifiable {
    case body = "Body"
    case headers = "Headers"
    var id: String { rawValue }
}

struct ResponseView: View {
    let model: WorkspaceModel
    @State private var tab: ResponseTab = .body

    var body: some View {
        VStack(spacing: 0) {
            if let response = model.response {
                summary(response)
                Divider()
                Picker("Section", selection: $tab) {
                    ForEach(ResponseTab.allCases) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .frame(maxWidth: 240)
                .padding(.vertical, 6)

                switch tab {
                case .body:
                    bodyView(response)
                case .headers:
                    headersView(response)
                }
            } else if let error = model.sendError {
                ContentUnavailableView(
                    "Request Failed", systemImage: "exclamationmark.octagon",
                    description: Text(error))
            } else if model.isSending {
                ProgressView("Sending…")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ContentUnavailableView(
                    "No Response Yet", systemImage: "arrow.down.circle",
                    description: Text("Press Send (⌘↩) to run the request."))
            }
        }
    }

    private func summary(_ r: HttpResponse) -> some View {
        HStack(spacing: 14) {
            Text("\(r.status) \(r.statusText)")
                .font(.headline)
                .foregroundStyle(statusColor(r.status))
            Text("\(r.timeMs) ms")
            Text(ByteCountFormatter.string(fromByteCount: Int64(r.sizeBytes), countStyle: .file))
            if let env = r.environment {
                Text(env).foregroundStyle(.secondary)
            }
            Spacer()
            if !r.unresolvedVariables.isEmpty {
                Label(
                    "Unresolved: " + r.unresolvedVariables.map { "{{\($0)}}" }.joined(separator: " "),
                    systemImage: "exclamationmark.triangle"
                )
                .foregroundStyle(.orange)
                .font(.callout)
            }
        }
        .font(.callout.monospacedDigit())
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }

    @ViewBuilder
    private func bodyView(_ r: HttpResponse) -> some View {
        if r.bodyIsBase64 {
            ContentUnavailableView(
                "Binary Response", systemImage: "doc",
                description: Text("\(r.contentType) · \(r.sizeBytes) bytes"))
        } else {
            CodeEditor(text: .constant(r.prettyBody ?? r.body), isEditable: false)
        }
    }

    private func headersView(_ r: HttpResponse) -> some View {
        List(r.headers.indices, id: \.self) { index in
            HStack(alignment: .top) {
                Text(r.headers[index].name)
                    .fontWeight(.medium)
                    .frame(width: 200, alignment: .leading)
                Text(r.headers[index].value)
                    .textSelection(.enabled)
                Spacer()
            }
            .font(.system(.callout, design: .monospaced))
        }
        .listStyle(.plain)
    }

    private func statusColor(_ status: UInt16) -> Color {
        switch status {
        case 200..<300: return .green
        case 300..<400: return .blue
        case 400..<500: return .orange
        default: return .red
        }
    }
}
