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

    var showsCountdown: Bool { effectiveStyle == .countdown }
    var showsTypeField: Bool { effectiveStyle == .typeName }
    var typePrompt: String { "Type \(environment) to confirm" }

    var sendEnabled: Bool {
        ConfirmationPolicy.canSend(
            style: effectiveStyle, remaining: remaining, typed: typed, environment: environment)
    }

    var sendLabel: String {
        if case .unresolved = reason { return "Send Anyway" }
        if effectiveStyle == .countdown, remaining > 0 { return "Send to \(environment) (\(remaining))" }
        return "Send to \(environment)"
    }

    var countdownCaption: String {
        remaining > 0 ? "Unlocks in \(remaining)s" : "Ready"
    }

    var defaultButton: DefaultButton { returnCancels ? .cancel : .send }
}
