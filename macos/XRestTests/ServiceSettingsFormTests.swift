import XCTest

@testable import XRest

final class ServiceSettingsFormTests: XCTestCase {
    private func info(preflight: Preflight? = nil, authKind: String = "bearer") -> ServiceInfo {
        ServiceInfo(
            id: "i", name: "Svc", rootPath: "/tmp", isAuthenticated: true, authKind: authKind,
            formatVersion: 2, basicUser: "u", basicPass: "p", bearerToken: "{{secret.T}}",
            apiKeyName: "X-Key", apiKeyValue: "v", apiKeyLocation: "", preflight: preflight)
    }

    private func preflight(duration: String = "derived", header: String? = nil) -> Preflight {
        Preflight(
            enabled: true, method: "POST", url: "{{BASE}}/t", body: "{}", bodyType: "raw",
            bodyParams: [], headers: [], cacheToken: true, cacheDuration: duration,
            cacheDurationKey: "expires_in", cacheDurationUnit: "epoch_ms", tokenKey: "data.token",
            tokenHeader: header)
    }

    func testFormFromInfoMapsFieldsAndDefaultsLocationToHeader() {
        let f = ServiceSettingsForm(info())
        XCTAssertEqual(f.name, "Svc")
        XCTAssertTrue(f.isAuthenticated)
        XCTAssertEqual(f.authType, "bearer")
        XCTAssertEqual(f.apiKeyLocation, "header")
        XCTAssertNil(f.preflight)
    }

    func testToSettingsRoundTrip() {
        let pf = preflight(duration: "3600", header: "X-Auth")
        let s = ServiceSettingsForm(info(preflight: pf)).toSettings()
        XCTAssertEqual(s.name, "Svc")
        XCTAssertEqual(s.authType, "bearer")
        XCTAssertEqual(s.bearerToken, "{{secret.T}}")
        XCTAssertEqual(s.apiKeyLocation, "header")
        XCTAssertEqual(s.preflight, pf)
    }

    func testDerivedDurationAndEmptyTokenHeaderMapping() {
        var f = ServiceSettingsForm(info(preflight: preflight()))
        XCTAssertTrue(f.preflight!.durationDerived)
        f.preflight!.durationDerived = false
        f.preflight!.durationNumber = "120"
        f.preflight!.tokenHeader = "  "
        let pf = f.toSettings().preflight!
        XCTAssertEqual(pf.cacheDuration, "120")
        XCTAssertNil(pf.tokenHeader)
        f.preflight!.durationDerived = true
        XCTAssertEqual(f.toSettings().preflight!.cacheDuration, "derived")
    }

    func testNonNumericDurationIsInvalid() {
        var f = ServiceSettingsForm(info(preflight: preflight()))
        f.preflight!.durationDerived = false
        f.preflight!.durationNumber = "abc"
        XCTAssertFalse(f.validationErrors.isEmpty)
        f.preflight!.durationNumber = "10"
        XCTAssertTrue(f.validationErrors.isEmpty)
    }

    func testEmptyNameInvalidAndDirtiness() {
        let original = ServiceSettingsForm(info())
        var f = original
        XCTAssertEqual(f, original)
        f.name = " "
        XCTAssertFalse(f.validationErrors.isEmpty)
        XCTAssertNotEqual(f, original)
    }

    func testPlainCredentialWarning() {
        XCTAssertFalse(AuthWarning.isPlainCredential(""))
        XCTAssertFalse(AuthWarning.isPlainCredential("{{secret.T}}"))
        XCTAssertFalse(AuthWarning.isPlainCredential("{{ TOKEN }}"))
        XCTAssertTrue(AuthWarning.isPlainCredential("abc123"))
        var f = ServiceSettingsForm(info())
        f.authType = "bearer"
        f.bearerToken = "abc"
        XCTAssertEqual(f.plainCredentialFields, ["Bearer token"])
        f.authType = "none"
        XCTAssertTrue(f.plainCredentialFields.isEmpty)
        f.authType = "basic"
        f.basicPass = "pw"
        XCTAssertEqual(f.plainCredentialFields, ["Password"])
        f.authType = "apikey"
        f.apiKeyValue = "k"
        XCTAssertEqual(f.plainCredentialFields, ["API key value"])
    }
}
