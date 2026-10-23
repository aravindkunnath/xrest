import AppKit
import SwiftUI

enum SpecSourceKind: String, CaseIterable, Identifiable {
    case file = "File"
    case url = "URL"
    var id: String { rawValue }
}

/// Reads spec text from a chosen file or a fetched URL. Shared by the new-service and re-import sheets.
@MainActor
@Observable
final class SpecLoader {
    var kind = SpecSourceKind.file
    var fileURL: URL?
    var urlText = ""
    var isFetching = false

    var hasInput: Bool {
        switch kind {
        case .file: fileURL != nil
        case .url: !urlText.trimmingCharacters(in: .whitespaces).isEmpty
        }
    }

    func chooseFile() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        panel.message = "Choose an OpenAPI or Swagger spec (JSON or YAML)."
        if panel.runModal() == .OK { fileURL = panel.url }
    }

    func load() async throws -> String {
        switch kind {
        case .file:
            guard let fileURL else { throw OpenAPIInputError(message: "Choose a spec file.") }
            return try String(contentsOf: fileURL, encoding: .utf8)
        case .url:
            switch OpenAPIInput.validateURL(urlText) {
            case .failure(let e): throw e
            case .success(let valid):
                isFetching = true
                defer { isFetching = false }
                return try await fetchText(url: valid)
            }
        }
    }
}

private struct SpecSourcePicker: View {
    @Bindable var loader: SpecLoader

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Picker("Source", selection: $loader.kind) {
                ForEach(SpecSourceKind.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .labelsHidden()
            switch loader.kind {
            case .file:
                HStack {
                    Text(loader.fileURL?.path ?? "No file chosen")
                        .lineLimit(1).truncationMode(.head)
                        .foregroundStyle(loader.fileURL == nil ? .secondary : .primary)
                    Spacer()
                    Button("Choose…") { loader.chooseFile() }
                }
            case .url:
                TextField("https://example.com/openapi.json", text: $loader.urlText)
                    .textFieldStyle(.roundedBorder)
            }
        }
    }
}

struct NewServiceSheet: View {
    let onCreated: (URL) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var loader = SpecLoader()
    @State private var name = ""
    @State private var folder: URL?
    @State private var error: String?
    @State private var working = false
    @State private var created: URL?
    @State private var warnings: [String] = []

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("New Service from OpenAPI").font(.headline)
            if let created {
                Text("Created \(created.lastPathComponent).").font(.subheadline)
                Text("Import warnings").font(.subheadline.weight(.semibold))
                ScrollView {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(Array(warnings.enumerated()), id: \.offset) { _, w in Text(w).font(.callout) }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 160)
                HStack {
                    Spacer()
                    Button("Open Service") { onCreated(created); dismiss() }
                        .keyboardShortcut(.defaultAction)
                }
            } else {
                SpecSourcePicker(loader: loader)
                TextField("Name (defaults to the spec title)", text: $name)
                    .textFieldStyle(.roundedBorder)
                HStack {
                    Text(folder?.path ?? "No destination folder chosen")
                        .lineLimit(1).truncationMode(.head)
                        .foregroundStyle(folder == nil ? .secondary : .primary)
                    Spacer()
                    Button("Choose Folder…") { chooseFolder() }
                }
                if let error {
                    Text(error).font(.callout).foregroundStyle(.red).textSelection(.enabled)
                }
                HStack {
                    if working { ProgressView().controlSize(.small) }
                    Spacer()
                    Button("Cancel", role: .cancel) { dismiss() }.keyboardShortcut(.cancelAction)
                    Button("Create") { create() }
                        .keyboardShortcut(.defaultAction)
                        .disabled(!loader.hasInput || folder == nil || working)
                }
            }
        }
        .padding(20)
        .frame(width: 520)
    }

    private func chooseFolder() {
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.canCreateDirectories = true
        panel.allowsMultipleSelection = false
        panel.prompt = "Choose"
        panel.message = "Choose the folder for the new service."
        if panel.runModal() == .OK { folder = panel.url; error = nil }
    }

    private func create() {
        guard let folder else { return }
        if let problem = OpenAPIInput.folderProblem(folder) {
            error = problem
            return
        }
        error = nil
        working = true
        Task {
            defer { working = false }
            do {
                let content = try await loader.load()
                let result = try OpenAPIInput.createService(at: folder, name: name, content: content)
                if result.warnings.isEmpty {
                    onCreated(folder)
                    dismiss()
                } else {
                    warnings = result.warnings
                    created = folder
                }
            } catch {
                self.error = describe(error)
            }
        }
    }
}

struct ReimportOpenAPISheet: View {
    @Bindable var model: WorkspaceModel
    @State private var loader = SpecLoader()
    @State private var error: String?
    @State private var result: String?
    @State private var working = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Re-import OpenAPI").font(.headline)
            Text("New operations are added as endpoints; existing ones (same method and path) are left alone.")
                .font(.subheadline).foregroundStyle(.secondary)
            SpecSourcePicker(loader: loader)
            if let error { Text(error).font(.callout).foregroundStyle(.red).textSelection(.enabled) }
            if let result { Text(result).font(.callout.weight(.semibold)) }
            HStack {
                if working { ProgressView().controlSize(.small) }
                Spacer()
                Button(result == nil ? "Cancel" : "Done") { model.showReimport = false }
                    .keyboardShortcut(.cancelAction)
                Button("Import") { run() }
                    .keyboardShortcut(.defaultAction)
                    .disabled(!loader.hasInput || working)
            }
        }
        .padding(20)
        .frame(width: 520)
    }

    private func run() {
        error = nil
        result = nil
        working = true
        Task {
            defer { working = false }
            do {
                let content = try await loader.load()
                result = OpenAPIInput.summaryText(try model.reimportOpenapi(content))
            } catch {
                self.error = describe(error)
            }
        }
    }
}

struct ExportCodeSheet: View {
    @Bindable var model: WorkspaceModel
    @State private var state = CodeExportState()
    @State private var copied = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Export Code").font(.headline)
            Picker("Language", selection: $state.language) {
                Text("curl").tag(CodeLanguage.curl)
                Text("axios").tag(CodeLanguage.axios)
                Text("Java").tag(CodeLanguage.java)
                Text("Kotlin").tag(CodeLanguage.kotlin)
            }
            .pickerStyle(.segmented)
            Toggle("Include secret values", isOn: $state.includeSecrets)
            if state.showsSecretWarning {
                Label(
                    "Secret values will appear in this code. Do not share or commit it.",
                    systemImage: "exclamationmark.triangle.fill"
                )
                .font(.callout).foregroundStyle(.orange)
            }
            if let error = state.error {
                Text(error).font(.callout).foregroundStyle(.red).textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            CodeEditor(text: .constant(state.output), isEditable: false)
                .frame(minHeight: 220)
                .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.secondary.opacity(0.4)))
            HStack {
                if copied { Text("Copied").font(.callout).foregroundStyle(.secondary) }
                Spacer()
                Button("Close") { model.showExportCode = false }.keyboardShortcut(.cancelAction)
                Button("Copy") {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(state.output, forType: .string)
                    copied = true
                }
                .keyboardShortcut(.defaultAction)
                .disabled(state.output.isEmpty)
            }
        }
        .padding(20)
        .frame(width: 640)
        .onAppear {
            state.resetForOpen()
            regenerate()
        }
        .onChange(of: state.language) { regenerate() }
        .onChange(of: state.includeSecrets) { regenerate() }
    }

    private func regenerate() {
        copied = false
        state.generate { _, lang, secrets in
            try model.exportCode(language: lang, includeSecrets: secrets)
        }
    }
}
