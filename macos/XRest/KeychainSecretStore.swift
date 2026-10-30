import Foundation
import Security

struct KeychainError: Error, LocalizedError {
    let status: OSStatus
    var errorDescription: String? {
        let text = SecCopyErrorMessageString(status, nil) as String? ?? "status \(status)"
        return "Keychain error: \(text)"
    }
}

/// Secret values in the macOS Keychain (generic passwords), exposed to the Rust core through
/// the generated `SecretProvider` protocol.
///
/// The Keychain cannot enumerate cheaply, so a names-only index is kept in a JSON file. Values
/// never go in the index, logs or UserDefaults. Uses the file-based login keychain (no
/// data-protection entitlement needed).
final class KeychainSecretStore: SecretProvider, @unchecked Sendable {
    static let appService = "dev.xrest.secrets"

    static var defaultIndexURL: URL {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("XRest/secret-keys.json")
    }

    private let service: String
    private let indexURL: URL
    private let lock = NSLock()

    init(service: String = KeychainSecretStore.appService, indexURL: URL = KeychainSecretStore.defaultIndexURL) {
        self.service = service
        self.indexURL = indexURL
    }

    private func query(_ key: String) -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: key,
        ]
    }

    // MARK: SecretProvider

    func get(key: String) -> String? {
        lock.lock()
        defer { lock.unlock() }
        var q = query(key)
        q[kSecReturnData as String] = true
        q[kSecMatchLimit as String] = kSecMatchLimitOne
        var out: CFTypeRef?
        guard SecItemCopyMatching(q as CFDictionary, &out) == errSecSuccess,
            let data = out as? Data
        else { return nil }
        return String(data: data, encoding: .utf8)
    }

    /// Existence from the names index only: no `SecItemCopyMatching`, so no Keychain read.
    func contains(key: String) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return readIndex().contains(key)
    }

    // MARK: Management

    func set(key: String, value: String) throws {
        lock.lock()
        defer { lock.unlock() }
        let data = Data(value.utf8)
        var status = SecItemUpdate(
            query(key) as CFDictionary, [kSecValueData as String: data] as CFDictionary)
        if status == errSecItemNotFound {
            var add = query(key)
            add[kSecValueData as String] = data
            status = SecItemAdd(add as CFDictionary, nil)
        }
        guard status == errSecSuccess else { throw KeychainError(status: status) }
        try writeIndex(readIndex().union([key]))
    }

    func delete(key: String) throws {
        lock.lock()
        defer { lock.unlock() }
        let status = SecItemDelete(query(key) as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw KeychainError(status: status)
        }
        try writeIndex(readIndex().subtracting([key]))
    }

    /// Stored key names, sorted.
    func keys() -> [String] {
        lock.lock()
        defer { lock.unlock() }
        return readIndex().sorted()
    }

    // MARK: Index (names only)

    private func readIndex() -> Set<String> {
        guard let data = try? Data(contentsOf: indexURL),
            let names = try? JSONDecoder().decode([String].self, from: data)
        else { return [] }
        return Set(names)
    }

    private func writeIndex(_ names: Set<String>) throws {
        try FileManager.default.createDirectory(
            at: indexURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        try JSONEncoder().encode(names.sorted()).write(to: indexURL, options: .atomic)
    }
}

/// Pure logic behind the Secrets screen.
enum SecretsState {
    struct Needed: Equatable {
        let key: String
        let isMissing: Bool
    }

    static func needed(inUse: [String], stored: [String]) -> [Needed] {
        let have = Set(stored)
        return Set(inUse).sorted().map { Needed(key: $0, isMissing: !have.contains($0)) }
    }

    static func missingKeys(in error: Error) -> [String] {
        if case XRestError.MissingSecrets(let keys) = error { return keys }
        return []
    }
}
