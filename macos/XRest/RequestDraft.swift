import Foundation

/// One editable name/value row. Wraps the core's `NameValue` with a stable identity for SwiftUI.
struct KeyValueRow: Identifiable, Hashable {
    var id = UUID()
    var name = ""
    var value = ""
    var enabled = true
    /// Preserved so a round trip through the editor does not drop secret references.
    var secretKey: String?
    var kind = "plain"

    init() {}

    init(_ nv: NameValue) {
        name = nv.name
        value = nv.value
        enabled = nv.enabled
        secretKey = nv.secretKey
        kind = nv.kind
    }

    var nameValue: NameValue {
        NameValue(name: name, value: value, enabled: enabled, secretKey: secretKey, kind: kind)
    }
}

/// The request currently open in the editor: an endpoint or scratch request being edited.
/// Everything the editor does not show (e.g. preflight) is carried along untouched.
struct RequestDraft: Equatable {
    var name: String
    var method: String
    var url: String
    var authenticated: Bool
    var params: [KeyValueRow]
    var headers: [KeyValueRow]
    var body: String
    var preflight: Preflight?

    static let methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]

    init(name: String, spec: RequestSpec) {
        self.name = name
        method = spec.method.isEmpty ? "GET" : spec.method.uppercased()
        url = spec.url
        authenticated = spec.authenticated
        params = spec.params.map(KeyValueRow.init)
        headers = spec.headers.map(KeyValueRow.init)
        body = spec.body
        preflight = spec.preflight
    }

    static var blank: RequestDraft {
        RequestDraft(
            name: "New Request",
            spec: RequestSpec(
                method: "GET", url: "{{BASE_URL}}/", authenticated: false,
                params: [], headers: [], body: "", preflight: nil))
    }

    var spec: RequestSpec {
        RequestSpec(
            method: method, url: url, authenticated: authenticated,
            params: params.map(\.nameValue), headers: headers.map(\.nameValue),
            body: body, preflight: preflight)
    }
}

extension XRestError {
    /// Human-readable message for alerts and inline errors.
    var message: String {
        switch self {
        case .ConfirmationRequired(let environment):
            return "Environment \"\(environment)\" is unsafe. Sending requires confirmation."
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
