import AppKit
import SwiftUI

/// Edits the request body: a type picker plus the editor for that type.
struct BodyEditor: View {
    @Binding var draft: RequestDraft
    /// Absolute path of the workspace; files are stored relative to it.
    let workspaceRoot: String

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Picker("Body type", selection: $draft.bodyKind) {
                    ForEach(BodyKind.allCases) { Text($0.rawValue).tag($0) }
                }
                .labelsHidden()
                .fixedSize()
                .tint(Theme.amber)

                if draft.bodyKind == .text || draft.bodyKind == .binary {
                    TextField("Content-Type (automatic)", text: $draft.bodyContentType)
                        .themedField(mono: true)
                        .frame(maxWidth: 280)
                }
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)

            content
        }
    }

    @ViewBuilder
    private var content: some View {
        switch draft.bodyKind {
        case .none:
            Text("This request has no body.")
                .font(.ui(12))
                .foregroundStyle(Theme.muted)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        case .text, .json:
            CodeEditor(text: $draft.bodyText)
        case .form:
            KeyValueEditor(rows: $draft.formFields, namePlaceholder: "Field")
        case .multipart:
            KeyValueEditor(
                rows: $draft.multipartFields, namePlaceholder: "Field",
                chooseFile: { chooseFile() })
        case .binary:
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 8) {
                    TextField("path/inside/workspace", text: $draft.binaryPath)
                        .themedField(mono: true)
                    Button("Choose…") {
                        if let path = chooseFile() { draft.binaryPath = path }
                    }
                }
                Text("Files sent in a request must be inside the workspace folder.")
                    .font(.ui(11))
                    .foregroundStyle(Theme.muted)
                Spacer()
            }
            .padding(.horizontal, 16)
        }
    }

    /// Pick a file, returned relative to the workspace when it is inside it. A file outside is
    /// returned as an absolute path; the core refuses to send it and says why.
    private func chooseFile() -> String? {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        panel.directoryURL = URL(fileURLWithPath: workspaceRoot, isDirectory: true)
        panel.prompt = "Choose"
        panel.message = "Choose a file inside the workspace folder."
        guard panel.runModal() == .OK, let url = panel.url else { return nil }
        let root = URL(fileURLWithPath: workspaceRoot).standardizedFileURL.path
        let path = url.standardizedFileURL.path
        let prefix = root.hasSuffix("/") ? root : root + "/"
        return path.hasPrefix(prefix) ? String(path.dropFirst(prefix.count)) : path
    }
}
