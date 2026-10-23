import SwiftUI

private enum EditorTab: String, CaseIterable, Identifiable {
    case params = "Params"
    case headers = "Headers"
    case body = "Body"
    case auth = "Auth"
    case options = "Options"
    var id: String { rawValue }
}

/// The main column: open-request tab, URL bar, and the request / response split.
struct RequestDetailView: View {
    @Bindable var model: WorkspaceModel
    @State private var tab: EditorTab = .params

    /// Non-optional view of the draft for bindings; the parent only shows this view when set.
    private var draft: Binding<RequestDraft> {
        Binding(
            get: { model.draft ?? RequestDraft.blank },
            set: { model.draft = $0 })
    }

    var body: some View {
        VStack(spacing: 0) {
            if model.selectedEnvironmentIsUnsafe {
                UnsafeBanner(environment: model.selectedEnvironment ?? "")
            }
            urlBar
            HStack(spacing: 0) {
                requestPane
                    .frame(minWidth: 320, maxWidth: .infinity)
                ResponseView(model: model)
                    .frame(minWidth: 320, maxWidth: .infinity)
                    .hairline(.leading)
            }
        }
        .background(Theme.bg)
    }

    // MARK: URL bar

    private var urlBar: some View {
        HStack(spacing: 8) {
            HStack(spacing: 0) {
                Menu {
                    ForEach(RequestDraft.methods, id: \.self) { method in
                        Button(method) { draft.wrappedValue.method = method }
                    }
                } label: {
                    Text(draft.wrappedValue.method)
                        .font(.mono(13, weight: .semibold))
                        .foregroundStyle(Theme.methodColor(draft.wrappedValue.method))
                        .padding(.horizontal, 12)
                        .frame(height: 36)
                        .contentShape(Rectangle())
                }
                .menuStyle(.button)
                .buttonStyle(.plain)
                .menuIndicator(.hidden)
                .fixedSize()
                .hairline(.trailing, color: Theme.borderStrong)
                .accessibilityLabel("HTTP method")

                URLField(text: draft.url, onSubmit: model.requestSend)
                    .padding(.horizontal, 12)
            }
            .frame(height: 36)
            .chrome(fill: Theme.sidebar, stroke: Theme.borderStrong, radius: 6)

            Button {
                model.saveCurrent()
            } label: {
                Text("Save")
                    .font(.ui(13))
                    .foregroundStyle(model.isDirty ? Theme.textSoft : Theme.muted)
                    .padding(.horizontal, 16)
                    .frame(height: 36)
                    .chrome(fill: Theme.sidebar, stroke: Theme.borderStrong, radius: 6)
            }
            .buttonStyle(.plain)
            .disabled(!model.isDirty)
            .help("Save (⌘S)")

            Button {
                model.requestSend()
            } label: {
                HStack(spacing: 6) {
                    if model.isSending {
                        ProgressView().controlSize(.small).tint(Theme.bg)
                    } else {
                        Text("Send").font(.ui(13, weight: .semibold))
                        Text("⌘↵").font(.mono(11)).opacity(0.7).fixedSize()
                    }
                }
                .foregroundStyle(model.selectedEnvironmentIsUnsafe ? Color.white : Theme.bg)
                .padding(.horizontal, 20)
                .frame(height: 36)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(model.selectedEnvironmentIsUnsafe ? Theme.red : Theme.amber)
                )
            }
            .buttonStyle(.plain)
            .disabled(model.isSending)
            .help("Send (⌘↩)")
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .hairline(.bottom)
    }

    // MARK: Request pane

    private var requestPane: some View {
        VStack(spacing: 0) {
            nameRow
            UnderlineTabs(
                tabs: EditorTab.allCases, selection: $tab,
                label: { AnyView(tabLabel($0)) })
            switch tab {
            case .params:
                KeyValueEditor(rows: draft.params, namePlaceholder: "Parameter")
            case .headers:
                KeyValueEditor(rows: draft.headers, namePlaceholder: "Header")
            case .body:
                BodyEditor(draft: draft, workspaceRoot: model.service.rootPath)
            case .auth:
                authPane
            case .options:
                OptionsEditor(options: draft.options)
            }
        }
        .frame(maxHeight: .infinity, alignment: .top)
    }

    private var nameRow: some View {
        HStack(spacing: 8) {
            TextField("Name", text: draft.name)
                .textFieldStyle(.plain)
                .font(.ui(13, weight: .semibold))
                .foregroundStyle(Theme.text)
            if model.isScratchSelected {
                Button("Promote ↑") { model.promoteSelected() }
                    .buttonStyle(.plain)
                    .font(.ui(11))
                    .foregroundStyle(Theme.amber)
                    .padding(.horizontal, 7)
                    .frame(height: 22)
                    .chrome(fill: Theme.amberFill, stroke: Theme.amberBorder, radius: 4)
                    .help("Move this scratch request into the versioned endpoints (⇧⌘P)")
            }
        }
        .padding(.horizontal, 16)
        .frame(height: 34)
        .hairline(.bottom)
    }

    @ViewBuilder
    private func tabLabel(_ tab: EditorTab) -> some View {
        switch tab {
        case .params:
            HStack(spacing: 4) {
                Text("Params")
                count(draft.wrappedValue.params)
            }
        case .headers:
            HStack(spacing: 4) {
                Text("Headers")
                count(draft.wrappedValue.headers)
            }
        case .body:
            Text("Body")
        case .auth:
            HStack(spacing: 4) {
                Text("Auth ·")
                if draft.wrappedValue.authenticated, model.service.isAuthenticated {
                    Text("\(authKindName) (service)").foregroundStyle(Theme.lightGreen)
                } else {
                    Text("None")
                }
            }
        case .options:
            Text("Options")
        }
    }

    @ViewBuilder
    private func count(_ rows: [KeyValueRow]) -> some View {
        let n = rows.filter { $0.enabled && !$0.name.isEmpty }.count
        if n > 0 { Text("\(n)").foregroundStyle(Theme.muted) }
    }

    private var authKindName: String {
        switch model.service.authKind {
        case "bearer": return "Bearer"
        case "basic": return "Basic"
        case "apikey": return "API key"
        default: return model.service.authKind.capitalized
        }
    }

    private var authPane: some View {
        VStack(alignment: .leading, spacing: 10) {
            Toggle(isOn: draft.authenticated) {
                Text("Use service auth")
                    .font(.ui(13))
                    .foregroundStyle(Theme.text)
            }
            .toggleStyle(.checkbox)
            .tint(Theme.amber)
            .disabled(!model.service.isAuthenticated)
            Text(
                model.service.isAuthenticated
                    ? "Injects the service-level \(authKindName) authentication into this request."
                    : "This service has no authentication configured."
            )
            .font(.ui(12))
            .foregroundStyle(Theme.muted)
            Spacer()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

/// URL input that tints `{{VARIABLES}}` amber and query values blue, like the mockup.
///
/// The real text field draws transparent text on top of a styled copy of the same string, so
/// editing, selection and the caret stay native.
struct URLField: View {
    @Binding var text: String
    let onSubmit: () -> Void

    var body: some View {
        ZStack(alignment: .leading) {
            Text(styled)
                .font(.mono(13))
                .lineLimit(1)
                .truncationMode(.tail)
                .frame(maxWidth: .infinity, alignment: .leading)
                .allowsHitTesting(false)
                .accessibilityHidden(true)
            TextField("https://… or {{BASE_URL}}/path", text: $text)
                .textFieldStyle(.plain)
                .font(.mono(13))
                .foregroundStyle(.clear)
                .tint(Theme.amber)
                .onSubmit(onSubmit)
                .accessibilityLabel("Request URL")
        }
    }

    private var styled: AttributedString {
        var result = AttributedString(text.isEmpty ? " " : text)
        result.foregroundColor = Theme.text
        let ns = text as NSString
        let full = NSRange(location: 0, length: ns.length)

        func paint(_ pattern: String, _ apply: (inout AttributedSubstring) -> Void) {
            guard let regex = try? NSRegularExpression(pattern: pattern) else { return }
            for match in regex.matches(in: text, range: full) {
                guard let range = Range(match.range, in: text),
                    let lower = AttributedString.Index(range.lowerBound, within: result),
                    let upper = AttributedString.Index(range.upperBound, within: result)
                else { continue }
                apply(&result[lower..<upper])
            }
        }
        paint(#"(?<==)[^&#]+"#) { $0.foregroundColor = Theme.blue }
        paint(#"\{\{[^}]*\}\}"#) {
            $0.foregroundColor = Theme.amber
            $0.backgroundColor = Theme.amberFill
        }
        return result
    }
}

struct UnsafeBanner: View {
    let environment: String

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: "exclamationmark.triangle.fill")
            Text("\(environment) is an unsafe environment. Sends need confirmation.")
                .font(.ui(12, weight: .medium))
            Spacer()
        }
        .foregroundStyle(.white)
        .padding(.horizontal, 16)
        .padding(.vertical, 6)
        .background(Color(hex: 0xB3343D))
    }
}
