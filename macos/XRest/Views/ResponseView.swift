import AppKit
import SwiftUI

enum ResponseTab: String, CaseIterable, Identifiable {
    case pretty = "Pretty"
    case raw = "Raw"
    case headers = "Headers"
    case cookies = "Cookies"
    var id: String { rawValue }

    /// Tabs to offer for a response; Cookies only when it set any.
    static func visibleCases(for response: HttpResponse) -> [ResponseTab] {
        allCases.filter { $0 != .cookies || !response.cookies.isEmpty }
    }
}

struct ResponseView: View {
    let model: WorkspaceModel

    var body: some View {
        VStack(spacing: 0) {
            if let response = model.response {
                ResponseContent(response: response)
            } else if model.wasCancelled {
                responsePlaceholder(
                    "Cancelled", detail: "The request was cancelled before a response arrived.",
                    symbol: "xmark.circle", tint: Theme.muted)
            } else if let error = model.sendError {
                responsePlaceholder(
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
                responsePlaceholder(
                    "No response yet", detail: "Press Send (⌘↵) to run the request.",
                    symbol: "arrow.down.circle", tint: Theme.muted)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.response)
    }
}

func responsePlaceholder<Extra: View>(
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

/// The P0 response viewer for one response (summary bar, Pretty / Raw / Headers). Also used
/// by the History panel.
struct ResponseContent: View {
    let response: HttpResponse
    @State private var tab: ResponseTab = .pretty
    @State private var showAll = false

    var body: some View {
        VStack(spacing: 0) {
            summary(response)
            if !response.unresolvedVariables.isEmpty { unresolvedStrip(response) }
            switch tab {
            case .pretty:
                prettyView(response)
            case .raw:
                if response.bodyIsBase64 {
                    rawBinaryView(response)
                } else {
                    textBody(response.body)
                }
            case .headers:
                headersView(response)
            case .cookies:
                cookiesView(response)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.response)
        .onChange(of: response) {
            showAll = false
            if !ResponseTab.visibleCases(for: response).contains(tab) { tab = .pretty }
        }
    }

    private func placeholder(_ title: String, detail: String, symbol: String, tint: Color) -> some View {
        responsePlaceholder(title, detail: detail, symbol: symbol, tint: tint)
    }

    private func unresolvedStrip(_ r: HttpResponse) -> some View {
        Label(
            "Unresolved variables: " + r.unresolvedVariables.map { "{{\($0)}}" }.joined(separator: " "),
            systemImage: "exclamationmark.triangle"
        )
        .font(.ui(12))
        .foregroundStyle(Theme.amber)
        .lineLimit(2)
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 16)
        .padding(.vertical, 6)
        .background(Theme.amber.opacity(0.1))
        .hairline(.bottom)
        .textSelection(.enabled)
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
            Spacer()
            Button { copyBody(r) } label: { Image(systemName: "doc.on.doc") }
                .buttonStyle(.plain).help("Copy body")
            Button { saveBody(r) } label: { Image(systemName: "square.and.arrow.down") }
                .buttonStyle(.plain).help("Save body…")
            ForEach(ResponseTab.visibleCases(for: r)) { item in
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
    private func prettyView(_ r: HttpResponse) -> some View {
        switch ResponseKind.classify(contentType: r.contentType, bodyIsBase64: r.bodyIsBase64) {
        case .json:
            textBody(r.prettyBody ?? r.body)
        case .html:
            HTMLPreview(html: r.body)
        case .image:
            if let data = ResponseBodyBytes.decode(body: r.body, isBase64: true),
                let image = NSImage(data: data)
            {
                ScrollView([.horizontal, .vertical]) {
                    Image(nsImage: image).padding(16)
                }
            } else {
                placeholder(
                    "Image could not be decoded", detail: "\(r.contentType) · \(r.sizeBytes) bytes",
                    symbol: "photo", tint: Theme.muted)
            }
        case .text:
            textBody(r.body)
        case .binary:
            rawBinaryView(r)
        }
    }

    private func rawBinaryView(_ r: HttpResponse) -> some View {
        let data = ResponseBodyBytes.decode(body: r.body, isBase64: true) ?? Data()
        return CodeEditor(text: .constant(HexDump.format(data)), isEditable: false)
    }

    /// Text body, truncated above 2 MiB until "Show all" is pressed.
    private func textBody(_ text: String) -> some View {
        let result = showAll ? (text: text, truncated: false) : BodyTruncation.apply(text)
        return VStack(spacing: 0) {
            if result.truncated {
                HStack {
                    Text("Showing the first 2 MiB of \(ByteCountFormatter.string(fromByteCount: Int64(text.utf8.count), countStyle: .file)).")
                    Button("Show all") { showAll = true }
                }
                .font(.ui(12))
                .foregroundStyle(Theme.amber)
                .padding(.vertical, 6)
                .frame(maxWidth: .infinity)
                .hairline(.bottom)
            }
            CodeEditor(text: .constant(result.text), isEditable: false)
        }
    }

    private func copyBody(_ r: HttpResponse) {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(r.prettyBody ?? r.body, forType: .string)
    }

    private func saveBody(_ r: HttpResponse) {
        guard let data = ResponseBodyBytes.decode(body: r.body, isBase64: r.bodyIsBase64) else { return }
        let panel = NSSavePanel()
        panel.nameFieldStringValue = "response"
        panel.canCreateDirectories = true
        guard panel.runModal() == .OK, let url = panel.url else { return }
        try? data.write(to: url)
    }

    private func cookiesView(_ r: HttpResponse) -> some View {
        func expiry(_ c: ResponseCookie) -> String {
            [c.expires, c.maxAge.map { "Max-Age=\($0)" }].compactMap { $0 }.joined(separator: " · ")
        }
        func flags(_ c: ResponseCookie) -> String {
            [c.secure ? "Secure" : nil, c.httpOnly ? "HttpOnly" : nil, c.sameSite.map { "SameSite=\($0)" }]
                .compactMap { $0 }.joined(separator: " ")
        }
        return ScrollView {
            VStack(spacing: 0) {
                ForEach(r.cookies.indices, id: \.self) { index in
                    let c = r.cookies[index]
                    HStack(alignment: .top, spacing: 12) {
                        Text(c.name).foregroundStyle(Theme.blue)
                            .frame(width: 140, alignment: .leading)
                        Text(c.value).foregroundStyle(Theme.textSoft)
                            .frame(maxWidth: .infinity, alignment: .leading)
                        Text(c.domain ?? "").frame(width: 120, alignment: .leading)
                        Text(c.path ?? "").frame(width: 80, alignment: .leading)
                        Text(expiry(c)).frame(width: 180, alignment: .leading)
                        Text(flags(c)).frame(width: 160, alignment: .leading)
                    }
                    .font(.mono(12))
                    .foregroundStyle(Theme.muted)
                    .textSelection(.enabled)
                    .padding(.vertical, 6)
                    .hairline(.top, color: Theme.rowDivider)
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 6)
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
