enum UnresolvedGuard {
    static func shouldAsk(unresolved: [String], enabled: Bool) -> Bool {
        enabled && !unresolved.isEmpty
    }
}
