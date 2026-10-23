import Foundation

struct OpenAPIInputError: LocalizedError, Equatable {
    let message: String
    var errorDescription: String? { message }
}

/// Input validation and summary text for the OpenAPI import flows.
enum OpenAPIInput {
    /// Trimmed http(s) URL with a host, or the reason it was rejected.
    static func validateURL(_ raw: String) -> Result<String, OpenAPIInputError> {
        let text = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return .failure(.init(message: "Enter the URL of an OpenAPI spec.")) }
        guard let comps = URLComponents(string: text), let scheme = comps.scheme?.lowercased(),
            scheme == "http" || scheme == "https"
        else { return .failure(.init(message: "Only http:// and https:// URLs are supported.")) }
        guard let host = comps.host, !host.isEmpty else {
            return .failure(.init(message: "The URL has no host."))
        }
        return .success(text)
    }

    static func summaryText(_ summary: ImportSummary) -> String {
        "\(summary.added) added, \(summary.skipped) already present"
    }

    /// Why `folder` cannot receive a new service, or nil when it can.
    static func folderProblem(_ folder: URL) -> String? {
        let file = folder.appendingPathComponent("service.yaml").path
        if FileManager.default.fileExists(atPath: file) {
            return "\(folder.lastPathComponent) already contains a service.yaml. Choose an empty folder."
        }
        return nil
    }

    struct Created {
        let service: ServiceInfo
        let warnings: [String]
    }

    /// Creates a workspace in `folder` from `content`. An empty `name` lets the core use the spec title.
    static func createService(at folder: URL, name: String, content: String) throws -> Created {
        if let problem = folderProblem(folder) { throw OpenAPIInputError(message: problem) }
        let ws = try XRestWorkspace.createWorkspaceFromOpenapi(
            path: folder.path, name: name.trimmingCharacters(in: .whitespacesAndNewlines), content: content)
        return Created(service: ws.service(), warnings: ws.warnings())
    }
}

/// State of the "Export Code" sheet.
struct CodeExportState: Equatable {
    var language: CodeLanguage = .curl
    var includeSecrets = false
    private(set) var output = ""
    private(set) var error: String?

    var showsSecretWarning: Bool { includeSecrets }

    mutating func resetForOpen() {
        includeSecrets = false
        output = ""
        error = nil
    }

    mutating func generate(_ export: (CodeExportState, CodeLanguage, Bool) throws -> String) {
        do {
            output = try export(self, language, includeSecrets)
            error = nil
        } catch {
            output = ""
            self.error = describe(error)
        }
    }
}
