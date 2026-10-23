import Foundation

/// Suppresses the `onSubmit` that follows a Return which just accepted a completion.
struct SubmitGate {
    private var accepted = false

    mutating func markAccepted() { accepted = true }

    /// True when the submit should run; false (once) right after a completion was accepted.
    mutating func shouldSubmit() -> Bool {
        defer { accepted = false }
        return !accepted
    }

    mutating func clear() { accepted = false }
}
