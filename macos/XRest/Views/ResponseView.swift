import SwiftUI

private enum ResponseTab: String, CaseIterable, Identifiable {
    case pretty = "Pretty"
    case raw = "Raw"
    case headers = "Headers"
    var id: String { rawValue }
}

struct ResponseView: View {
    let model: WorkspaceModel
    @State private var tab: ResponseTab = .pretty

    var body: some View {
        VStack(spacing: 0) {
            if let response = model.response {
                summary(response)
                switch tab {
                case .pretty:
                    bodyView(response, text: response.prettyBody ?? response.body)
                case .raw:
                    bodyView(response, text: response.body)
                case .headers:
                    headersView(response)
                }
            } else if let error = model.sendError {
                placeholder(
                    "Request failed", detail: error, symbol: "exclamationmark.octagon",
                    tint: Theme.red
                ) {
                    if let first = model.missingSecretKeys.first {
                        Button("Open Secrets…") { model.openSecrets(prefill: first) }
                            .buttonStyle(.bordered)
                            .padding(.top, 4)
                    }
                }
            } else if model.isSending {
                ProgressView()
                    .controlSize(.small)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                placeholder(
                    "No response yet", detail: "Press Send (⌘↵) to run the request.",
                    symbol: "arrow.down.circle", tint: Theme.muted)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.response)
    }

    private func placeholder<Extra: View>(
        _ title: String, detail: String, symbol: String, tint: Color,
        @ViewBuilder extra: () -> Extra = { EmptyView() }
    ) -> some View {
        VStack(spacing: 8) {
            Image(systemName: symbol).font(.system(size: 26)).foregroundStyle(tint)
            Text(title).font(.ui(13, weight: .medium)).foregroundStyle(Theme.text)
            Text(detail)
                .font(.ui(12))
                .foregroundStyle(Theme.muted)
                .multilineTextAlignment(.center)
                .textSelection(.enabled)
            extra()
        }
        .padding(24)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func summary(_ r: HttpResponse) -> some View {
        HStack(spacing: 14) {
            Text("\(r.status) \(r.statusText)")
                .font(.mono(12, weight: .semibold))
                .foregroundStyle(Theme.bg)
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(RoundedRectangle(cornerRadius: 4).fill(Theme.statusColor(r.status)))
            Text("\(r.timeMs) ms").fixedSize()
            Text(ByteCountFormatter.string(fromByteCount: Int64(r.sizeBytes), countStyle: .file)).fixedSize()
            if let env = r.environment {
                Text(env)
            }
            if !r.unresolvedVariables.isEmpty {
                Label(
                    "Unresolved: " + r.unresolvedVariables.map { "{{\($0)}}" }.joined(separator: " "),
                    systemImage: "exclamationmark.triangle"
                )
                .font(.ui(12))
                .foregroundStyle(Theme.amber)
            }
            Spacer()
            ForEach(ResponseTab.allCases) { item in
                Button(item.rawValue) { tab = item }
                    .buttonStyle(.plain)
                    .font(.ui(13))
                    .foregroundStyle(tab == item ? Theme.text : Theme.muted)
            }
        }
        .font(.mono(12))
        .lineLimit(1)
        .foregroundStyle(Theme.muted)
        .padding(.horizontal, 16)
        .frame(minHeight: 36)
        .hairline(.bottom)
    }

    @ViewBuilder
    private func bodyView(_ r: HttpResponse, text: String) -> some View {
        if r.bodyIsBase64 {
            placeholder(
                "Binary response", detail: "\(r.contentType) · \(r.sizeBytes) bytes",
                symbol: "doc", tint: Theme.muted)
        } else {
            CodeEditor(text: .constant(text), isEditable: false)
        }
    }

    private func headersView(_ r: HttpResponse) -> some View {
        ScrollView {
            VStack(spacing: 0) {
                ForEach(r.headers.indices, id: \.self) { index in
                    HStack(alignment: .top, spacing: 12) {
                        Text(r.headers[index].name)
                            .foregroundStyle(Theme.blue)
                            .frame(width: 180, alignment: .leading)
                        Text(r.headers[index].value)
                            .foregroundStyle(Theme.textSoft)
                            .textSelection(.enabled)
                        Spacer(minLength: 0)
                    }
                    .font(.mono(12))
                    .padding(.vertical, 6)
                    .hairline(.top, color: Theme.rowDivider)
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 6)
        }
    }
}
