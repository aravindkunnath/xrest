import Foundation

enum ConfirmationPolicy {
    /// Countdown length for an unsafe send: the user's countdown when that is their style, and
    /// never less than the workspace's minimum for the environment (`teamMinimum`), whatever
    /// the style.
    static func countdownDuration(style: ConfirmStyle, seconds: Int, teamMinimum: Int = 0) -> Int {
        max(style == .countdown ? seconds : 0, max(0, teamMinimum))
    }

    /// Send unlocks only once any countdown has run out, plus the style's own condition.
    static func canSend(style: ConfirmStyle, remaining: Int, typed: String, environment: String) -> Bool {
        guard remaining <= 0 else { return false }
        switch style {
        case .countdown, .confirm: return true
        case .typeName: return typed.trimmingCharacters(in: .whitespaces) == environment
        }
    }
}
