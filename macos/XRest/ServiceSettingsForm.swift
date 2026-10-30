import Foundation

enum AuthWarning {
    /// A credential typed in as literal text (not a `{{variable}}` / `{{secret.KEY}}` reference).
    static func isPlainCredential(_ value: String) -> Bool {
        let v = value.trimmingCharacters(in: .whitespaces)
        return !v.isEmpty && !v.contains("{{")
    }
}

/// Editable copy of the service's preflight. `cacheDuration` is split into "derived" vs a number.
struct PreflightForm {
    static let methods = ["GET", "POST", "PUT", "PATCH", "DELETE"]
    static let units = ["seconds", "minutes", "hours", "days", "epoch", "epoch_ms", "iso8601"]

    var enabled = true
    var method = "POST"
    var url = ""
    var bodyType = "raw"
    var body = ""
    var bodyParams: [KeyValueRow] = []
    var headers: [KeyValueRow] = []
    var tokenKey = "access_token"
    var tokenHeader = ""
    var cacheToken = true
    var durationDerived = true
    var durationNumber = ""
    var durationKey = "expires_in"
    var durationUnit = "seconds"

    init() {}

    init(_ p: Preflight) {
        enabled = p.enabled
        method = p.method
        url = p.url
        bodyType = p.bodyType.isEmpty ? "raw" : p.bodyType
        body = p.body
        bodyParams = p.bodyParams.map(KeyValueRow.init)
        headers = p.headers.map(KeyValueRow.init)
        tokenKey = p.tokenKey
        tokenHeader = p.tokenHeader ?? ""
        cacheToken = p.cacheToken
        let d = p.cacheDuration.trimmingCharacters(in: .whitespaces)
        if d.isEmpty || d == "derived" {
            durationDerived = true
        } else {
            durationDerived = false
            durationNumber = d
        }
        durationKey = p.cacheDurationKey
        durationUnit = p.cacheDurationUnit.isEmpty ? "seconds" : p.cacheDurationUnit
    }

    var preflight: Preflight {
        let header = tokenHeader.trimmingCharacters(in: .whitespaces)
        return Preflight(
            enabled: enabled, method: method, url: url, body: body, bodyType: bodyType,
            bodyParams: bodyParams.map(\.nameValue), headers: headers.map(\.nameValue),
            cacheToken: cacheToken,
            cacheDuration: durationDerived ? "derived" : durationNumber.trimmingCharacters(in: .whitespaces),
            cacheDurationKey: durationKey, cacheDurationUnit: durationUnit, tokenKey: tokenKey,
            tokenHeader: header.isEmpty ? nil : header)
    }
}

/// Service settings as edited in the UI; maps to and from `ServiceInfo` / `ServiceSettings`.
struct ServiceSettingsForm: Equatable {
    static let authTypes = ["none", "basic", "bearer", "apikey"]

    var name: String
    var isAuthenticated: Bool
    var authType: String
    var basicUser: String
    var basicPass: String
    var bearerToken: String
    var apiKeyName: String
    var apiKeyValue: String
    var apiKeyLocation: String
    /// nil: the service has no preflight (the editor offers to add one).
    var preflight: PreflightForm?

    init(_ info: ServiceInfo) {
        name = info.name
        isAuthenticated = info.isAuthenticated
        authType = info.authKind.isEmpty ? "none" : info.authKind
        basicUser = info.basicUser
        basicPass = info.basicPass
        bearerToken = info.bearerToken
        apiKeyName = info.apiKeyName
        apiKeyValue = info.apiKeyValue
        apiKeyLocation = info.apiKeyLocation.isEmpty ? "header" : info.apiKeyLocation
        preflight = info.preflight.map(PreflightForm.init)
    }

    func toSettings() -> ServiceSettings {
        ServiceSettings(
            name: name.trimmingCharacters(in: .whitespaces), isAuthenticated: isAuthenticated,
            authType: authType, basicUser: basicUser, basicPass: basicPass,
            bearerToken: bearerToken, apiKeyName: apiKeyName, apiKeyValue: apiKeyValue,
            apiKeyLocation: apiKeyLocation, preflight: preflight?.preflight)
    }

    /// Labels of the active auth type's secret-bearing fields that hold literal text.
    var plainCredentialFields: [String] {
        switch authType {
        case "basic": return AuthWarning.isPlainCredential(basicPass) ? ["Password"] : []
        case "bearer": return AuthWarning.isPlainCredential(bearerToken) ? ["Bearer token"] : []
        case "apikey": return AuthWarning.isPlainCredential(apiKeyValue) ? ["API key value"] : []
        default: return []
        }
    }

    var validationErrors: [String] {
        var errors: [String] = []
        if name.trimmingCharacters(in: .whitespaces).isEmpty { errors.append("The service needs a name.") }
        if let p = preflight, !p.durationDerived, Double(p.durationNumber.trimmingCharacters(in: .whitespaces)) == nil {
            errors.append("Cache duration must be a number or \"derived\".")
        }
        return errors
    }

    static func == (lhs: Self, rhs: Self) -> Bool { lhs.toSettings() == rhs.toSettings() }
}
