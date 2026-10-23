import Foundation

/// One editable name/value row. Wraps the core's `NameValue` with a stable identity for SwiftUI.
struct KeyValueRow: Identifiable, Hashable {
    static let plainKind = "plain"
    static let fileKind = "file"

    var id = UUID()
    var name = ""
    var value = ""
    var enabled = true
    /// Preserved so a round trip through the editor does not drop secret references.
    var secretKey: String?
    /// `plain`, or `file` for a multipart field whose value is a path.
    var kind = KeyValueRow.plainKind

    init() {}

    init(_ nv: NameValue) {
        name = nv.name
        value = nv.value
        enabled = nv.enabled
        secretKey = nv.secretKey
        kind = nv.kind
    }

    var isFile: Bool { kind == KeyValueRow.fileKind }

    var nameValue: NameValue {
        NameValue(name: name, value: value, enabled: enabled, secretKey: secretKey, kind: kind)
    }
}

/// The body types the editor offers; maps one-to-one onto the core's `RequestBody`.
enum BodyKind: String, CaseIterable, Identifiable {
    case none = "None"
    case text = "Text"
    case json = "JSON"
    case form = "Form URL-Encoded"
    case multipart = "Multipart Form"
    case binary = "File"

    var id: String { rawValue }
}

/// The request currently open in the editor: an endpoint or scratch request being edited.
///
/// Each body type keeps its own content, so switching the type back and forth loses nothing.
/// Equality is defined by what would be saved (`name` + `spec`), so inactive editor state and
/// row identities never make a draft look edited.
struct RequestDraft: Equatable {
    var name: String
    var method: String
    var url: String {
        // `{name}` tokens in the URL path drive the Path rows: new tokens get a row, removed ones lose theirs.
        didSet {
            guard url != oldValue else { return }
            pathParams = PathTokens.synced(pathParams, tokens: PathTokens.extract(from: url))
        }
    }
    var authenticated: Bool
    var params: [KeyValueRow]
    var headers: [KeyValueRow]
    /// One row per `{name}` token in the URL path (kept in sync by `url`'s observer).
    var pathParams: [NameValue]

    var bodyKind: BodyKind = .none
    /// Content of a Text or JSON body.
    var bodyText = ""
    /// Explicit Content-Type for a Text or File body; empty means automatic.
    var bodyContentType = ""
    var formFields: [KeyValueRow] = []
    var multipartFields: [KeyValueRow] = []
    /// Path of a File body, relative to the workspace root.
    var binaryPath = ""

    var options: RequestOptions
    /// Not editable here yet; carried along untouched.
    var preflight: Preflight?

    static let methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]

    static let defaultOptions = RequestOptions(
        timeoutMs: nil, followRedirects: nil, verifyTls: nil, proxy: nil)

    init(name: String, spec: RequestSpec) {
        self.name = name
        method = spec.method.isEmpty ? "GET" : spec.method.uppercased()
        url = spec.url
        authenticated = spec.authenticated
        params = spec.params.map(KeyValueRow.init)
        headers = spec.headers.map(KeyValueRow.init)
        pathParams = spec.pathParams
        options = spec.options
        preflight = spec.preflight

        switch spec.body {
        case .empty:
            bodyKind = .none
        case .raw(let content, let contentType):
            bodyKind = .text
            bodyText = content
            bodyContentType = contentType ?? ""
        case .json(let content):
            bodyKind = .json
            bodyText = content
        case .form(let fields):
            bodyKind = .form
            formFields = fields.map(KeyValueRow.init)
        case .multipart(let fields):
            bodyKind = .multipart
            multipartFields = fields.map(KeyValueRow.init)
        case .binary(let path, let contentType):
            bodyKind = .binary
            binaryPath = path
            bodyContentType = contentType ?? ""
        }
    }

    func pathParamValue(_ name: String) -> String {
        pathParams.first { $0.name == name }?.value ?? ""
    }

    /// Set the value of an existing path token's row; unknown names are ignored.
    mutating func setPathParam(_ name: String, to value: String) {
        guard let i = pathParams.firstIndex(where: { $0.name == name }) else { return }
        pathParams[i].value = value
    }

    static var blank: RequestDraft {
        RequestDraft(
            name: "New Request",
            spec: RequestSpec(
                method: "GET", url: "{{BASE_URL}}/", authenticated: false,
                params: [], pathParams: [], headers: [], body: .empty, options: defaultOptions,
                preflight: nil))
    }

    private var explicitContentType: String? {
        let trimmed = bodyContentType.trimmingCharacters(in: .whitespaces)
        return trimmed.isEmpty ? nil : trimmed
    }

    var requestBody: RequestBody {
        switch bodyKind {
        case .none:
            return .empty
        case .text:
            return .raw(content: bodyText, contentType: explicitContentType)
        case .json:
            return .json(content: bodyText)
        case .form:
            return .form(fields: formFields.map(\.nameValue))
        case .multipart:
            return .multipart(fields: multipartFields.map(\.nameValue))
        case .binary:
            return .binary(path: binaryPath, contentType: explicitContentType)
        }
    }

    var spec: RequestSpec {
        RequestSpec(
            method: method, url: url, authenticated: authenticated,
            params: params.map(\.nameValue), pathParams: pathParams, headers: headers.map(\.nameValue),
            body: requestBody, options: options, preflight: preflight)
    }

    static func == (lhs: RequestDraft, rhs: RequestDraft) -> Bool {
        lhs.name == rhs.name && lhs.spec == rhs.spec
    }
}

extension XRestError {
    /// Human-readable message for alerts and inline errors.
    var message: String {
        switch self {
        case .ConfirmationRequired(let environment):
            return "Environment \"\(environment)\" is unsafe. Sending requires confirmation."
        case .UnsupportedFormat(let found, let supported):
            return
                "This workspace was saved by a newer version of XRest (format \(found); this version supports up to \(supported)). Update XRest to open it."
        case .MissingSecrets(let keys):
            return "Missing secrets: \(keys.joined(separator: ", "))"
        case .NotFound(let m), .InvalidInput(let m), .Io(let m), .Http(let m),
            .Preflight(let m), .NotImplemented(let m):
            return m
        }
    }
}

func describe(_ error: Error) -> String {
    if let e = error as? XRestError { return e.message }
    return error.localizedDescription
}
