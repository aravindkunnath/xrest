import XCTest

@testable import XRest

private let petSpec = """
    openapi: 3.0.0
    info: {title: Pet Store, version: "1"}
    servers: [{url: "https://api.example.com"}]
    paths:
      /pets:
        get: {operationId: listPets, summary: List pets, responses: {"200": {description: ok}}}
        post: {operationId: addPet, summary: Add pet, responses: {"200": {description: ok}}}
    """

final class OpenAPIInputTests: XCTestCase {
    func testURLValidation() {
        XCTAssertEqual(OpenAPIInput.validateURL("  https://a.io/spec.json "), .success("https://a.io/spec.json"))
        XCTAssertEqual(OpenAPIInput.validateURL("http://localhost:8080/x"), .success("http://localhost:8080/x"))
        for bad in ["", "   ", "ftp://a.io/x", "file:///etc/passwd", "a.io/spec", "https://", "not a url"] {
            guard case .failure = OpenAPIInput.validateURL(bad) else {
                return XCTFail("should reject \(bad)")
            }
        }
    }

    func testSummaryText() {
        XCTAssertEqual(OpenAPIInput.summaryText(ImportSummary(added: 3, skipped: 2)), "3 added, 2 already present")
        XCTAssertEqual(OpenAPIInput.summaryText(ImportSummary(added: 0, skipped: 0)), "0 added, 0 already present")
    }

    func testNewServiceRefusesExistingService() throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-oa-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }
        XCTAssertNil(OpenAPIInput.folderProblem(dir))
        FileManager.default.createFile(atPath: dir.appendingPathComponent("service.yaml").path, contents: Data())
        let problem = OpenAPIInput.folderProblem(dir)
        XCTAssertNotNil(problem)
        XCTAssertTrue(problem?.contains("service.yaml") ?? false)
    }
}

final class CodeExportStateTests: XCTestCase {
    func testRegeneratesAndShowsErrorsInline() {
        var calls: [(CodeLanguage, Bool)] = []
        var state = CodeExportState()
        state.generate { _, lang, secrets in
            calls.append((lang, secrets))
            return "code-\(lang)"
        }
        XCTAssertEqual(state.output, "code-curl")
        XCTAssertNil(state.error)
        state.language = .kotlin
        state.generate { _, lang, secrets in calls.append((lang, secrets)); return "k" }
        XCTAssertEqual(calls.last?.0, .kotlin)
        state.includeSecrets = true
        state.generate { _, _, _ in throw XRestError.InvalidInput(message: "BASE_URL unresolved") }
        XCTAssertEqual(state.output, "")
        XCTAssertNotNil(state.error)
        XCTAssertTrue(state.showsSecretWarning)
    }

    func testResetOnOpenTurnsSecretsOff() {
        var state = CodeExportState()
        state.includeSecrets = true
        state.language = .java
        state.resetForOpen()
        XCTAssertFalse(state.includeSecrets)
        XCTAssertFalse(state.showsSecretWarning)
    }
}

@MainActor
final class OpenAPIModelTests: XCTestCase {
    private var dir: URL!
    override func setUp() {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("xrest-oam-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    }
    override func tearDown() { try? FileManager.default.removeItem(at: dir) }

    private func makeModel() throws -> WorkspaceModel {
        let ws = dir.appendingPathComponent("svc")
        try FileManager.default.createDirectory(at: ws, withIntermediateDirectories: true)
        _ = try XRestWorkspace.createWorkspace(path: ws.path, name: "svc")
        let store = KeychainSecretStore(
            service: "dev.xrest.tests.\(UUID().uuidString)", indexURL: dir.appendingPathComponent("idx.json"))
        return try WorkspaceModel(url: ws, secrets: store)
    }

    func testCreateServiceFromSpec() throws {
        let target = dir.appendingPathComponent("pets")
        try FileManager.default.createDirectory(at: target, withIntermediateDirectories: true)
        let result = try OpenAPIInput.createService(at: target, name: "", content: petSpec)
        XCTAssertEqual(result.service.name, "Pet Store")
        XCTAssertTrue(FileManager.default.fileExists(atPath: target.appendingPathComponent("service.yaml").path))
    }

    func testReimportMergesAndRefreshesLists() throws {
        let model = try makeModel()
        XCTAssertTrue(model.endpoints.isEmpty)
        let first = try model.reimportOpenapi(petSpec)
        XCTAssertEqual(first, ImportSummary(added: 2, skipped: 0))
        XCTAssertEqual(model.endpoints.count, 2)
        let second = try model.reimportOpenapi(petSpec)
        XCTAssertEqual(OpenAPIInput.summaryText(second), "0 added, 2 already present")
    }

    func testExportUsesUnsavedDraft() throws {
        let model = try makeModel()
        model.newScratch()
        model.draft?.url = "https://example.com/unsaved"
        let code = try model.exportCode(language: .curl, includeSecrets: false)
        XCTAssertTrue(code.contains("https://example.com/unsaved"))
    }
}
