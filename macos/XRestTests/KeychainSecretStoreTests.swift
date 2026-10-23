import XCTest

@testable import XRest

final class KeychainSecretStoreTests: XCTestCase {
    private var service = ""
    private var indexURL: URL!
    private var stores: [KeychainSecretStore] = []

    override func setUp() {
        service = "dev.xrest.tests.\(UUID().uuidString)"
        indexURL = FileManager.default.temporaryDirectory
            .appendingPathComponent("xrest-secret-index-\(UUID().uuidString)/secret-keys.json")
    }

    override func tearDown() {
        let store = makeStore()
        for key in store.keys() { try? store.delete(key: key) }
        try? FileManager.default.removeItem(at: indexURL.deletingLastPathComponent())
    }

    private func makeStore() -> KeychainSecretStore {
        KeychainSecretStore(service: service, indexURL: indexURL)
    }

    func testSetGetDeleteRoundTrip() throws {
        let store = makeStore()
        try store.set(key: "API_TOKEN", value: "s3cret")
        XCTAssertEqual(store.get(key: "API_TOKEN"), "s3cret")
        try store.set(key: "API_TOKEN", value: "rotated")
        XCTAssertEqual(store.get(key: "API_TOKEN"), "rotated")
        try store.delete(key: "API_TOKEN")
        XCTAssertNil(store.get(key: "API_TOKEN"))
        XCTAssertEqual(store.keys(), [])
    }

    func testIndexHoldsNamesOnly() throws {
        let store = makeStore()
        try store.set(key: "B_KEY", value: "very-secret-value")
        try store.set(key: "A_KEY", value: "another-value")
        let text = try String(contentsOf: indexURL, encoding: .utf8)
        XCTAssertTrue(text.contains("A_KEY"))
        XCTAssertFalse(text.contains("very-secret-value"))
        XCTAssertFalse(text.contains("another-value"))
        XCTAssertEqual(store.keys(), ["A_KEY", "B_KEY"])
    }

    func testProviderReturnsNilForUnknownKey() {
        let provider: SecretProvider = makeStore()
        XCTAssertNil(provider.get(key: "NOPE"))
    }

    func testPersistedAcrossStoreInstances() throws {
        try makeStore().set(key: "PERSIST", value: "v1")
        let second = makeStore()
        XCTAssertEqual(second.keys(), ["PERSIST"])
        XCTAssertEqual(second.get(key: "PERSIST"), "v1")
    }
}

final class SecretsStateTests: XCTestCase {
    func testNeededRowsFlagMissingKeys() {
        let rows = SecretsState.needed(inUse: ["B", "A", "C"], stored: ["A"])
        XCTAssertEqual(rows.map(\.key), ["A", "B", "C"])
        XCTAssertEqual(rows.map(\.isMissing), [false, true, true])
    }

    func testMissingKeysFromSendError() {
        XCTAssertEqual(SecretsState.missingKeys(in: XRestError.MissingSecrets(keys: ["X", "Y"])), ["X", "Y"])
        XCTAssertEqual(SecretsState.missingKeys(in: XRestError.Io(message: "no")), [])
    }
}
