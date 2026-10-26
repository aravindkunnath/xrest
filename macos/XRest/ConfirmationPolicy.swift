import Foundation

enum ConfirmationPolicy {
    static func countdownDuration(style: ConfirmStyle, seconds: Int) -> Int {
        style == .countdown ? seconds : 0
    }

    static func canSend(style: ConfirmStyle, remaining: Int, typed: String, environment: String) -> Bool {
        switch style {
        case .countdown: return remaining <= 0
        case .confirm: return true
        case .typeName: return typed.trimmingCharacters(in: .whitespaces) == environment
        }
    }
}
