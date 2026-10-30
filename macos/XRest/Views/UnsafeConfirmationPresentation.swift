import Foundation

/// Pure presentation decisions for `UnsafeConfirmationSheet`, kept separate so they can be unit tested.
struct UnsafeConfirmationPresentation: Equatable {
    enum DefaultButton: Equatable { case cancel, send }

    let reason: PendingConfirmation.Reason
    let style: ConfirmStyle
    let environment: String
    let returnCancels: Bool
    let remaining: Int
    let typed: String
    /// The workspace's minimum countdown for this environment (0 = none).
    var teamMinimum: Int = 0

    /// Unresolved-variable prompts are a zero-delay confirm regardless of the preferred style.
    private var effectiveStyle: ConfirmStyle {
        if case .unresolved = reason { return .confirm }
        return style
    }

    var title: String {
        switch reason {
        case .unsafeEnvironment: return "Send to \(environment)?"
        case .unresolved: return "Send with unresolved variables?"
        }
    }

    /// Hint under the "Return key cancels" setting; describes what Return does in the dialog.
    static func returnCancelsHint(returnCancels: Bool) -> String {
        returnCancels
            ? "Pressing Return in the dialog never sends."
            : "Pressing Return sends once Send is unlocked."
    }

    /// A team minimum runs a countdown even when the user's style has none.
    var showsCountdown: Bool {
        if case .unresolved = reason { return false }
        return effectiveStyle == .countdown || teamMinimum > 0
    }

    /// Explains a countdown the user did not choose, or one longer than theirs.
    var teamMinimumNote: String? {
        if case .unresolved = reason { return nil }
        guard teamMinimum > 0 else { return nil }
        return "This workspace waits at least \(teamMinimum)s before sending to \(environment)."
    }
    var showsTypeField: Bool { effectiveStyle == .typeName }
    var typePrompt: String { "Type \(environment) to confirm" }

    var sendEnabled: Bool {
        // A countdown only holds Send back while one is shown (never for unresolved prompts).
        ConfirmationPolicy.canSend(
            style: effectiveStyle, remaining: showsCountdown ? remaining : 0, typed: typed,
            environment: environment)
    }

    var sendLabel: String {
        if case .unresolved = reason { return "Send Anyway" }
        if showsCountdown, remaining > 0 { return "Send to \(environment) (\(remaining))" }
        return "Send to \(environment)"
    }

    var countdownCaption: String {
        remaining > 0 ? "Unlocks in \(remaining)s" : "Ready"
    }

    var defaultButton: DefaultButton { returnCancels ? .cancel : .send }
}
