import Foundation
import Observation

/// Countdown behind the unsafe-environment confirmation. Time is injected: the owner calls
/// `tick()` once per second (the sheet does it from a task; tests call it directly).
/// Reaching zero unlocks (`.ready`) and leaves the dialog open; `accept()` is valid while running
/// or ready and succeeds at most once. Only `cancel()` ends a pending confirmation.
@MainActor
@Observable
final class ConfirmationCountdown {
    enum State: Equatable {
        case running(remaining: Int)
        case ready
        case accepted
        case cancelled
    }

    nonisolated static let defaultDuration = 10

    let duration: Int
    private(set) var state: State

    init(duration: Int = ConfirmationCountdown.defaultDuration) {
        self.duration = duration
        state = duration > 0 ? .running(remaining: duration) : .ready
    }

    var isReady: Bool { state == .ready }

    var isRunning: Bool {
        if case .running = state { return true }
        return false
    }

    var remaining: Int {
        if case .running(let r) = state { return r }
        return 0
    }

    /// Fraction of time left, 1 -> 0.
    var progress: Double {
        duration > 0 ? Double(remaining) / Double(duration) : 0
    }

    func tick() {
        guard case .running(let r) = state else { return }
        state = r <= 1 ? .ready : .running(remaining: r - 1)
    }

    /// Returns true only for the one accept that happens while running or ready.
    @discardableResult
    func accept() -> Bool {
        guard isRunning || isReady else { return false }
        state = .accepted
        return true
    }

    func cancel() {
        if isRunning || isReady { state = .cancelled }
    }

    func reset() {
        state = duration > 0 ? .running(remaining: duration) : .ready
    }
}

/// An action waiting behind the unsafe confirmation. Any send path (Send button, ⌘↩, retry after
/// `ConfirmationRequired`, Test Preflight) wraps its work in one of these; `action` runs only if
/// the user accepts once the countdown has not been cancelled.
@MainActor
@Observable
final class PendingConfirmation: Identifiable {
    enum Reason: Equatable {
        case unsafeEnvironment
        case unresolved([String])
    }

    let id = UUID()
    let reason: Reason
    let environment: String
    let method: String
    let url: String
    /// The URL with variables resolved and secrets masked, when known.
    let resolvedURL: String?
    let countdown: ConfirmationCountdown
    let style: ConfirmStyle
    let returnCancels: Bool
    /// The workspace's minimum countdown for the environment (0 = none).
    let teamMinimum: Int
    @ObservationIgnored private var action: (() -> Void)?

    init(
        reason: Reason = .unsafeEnvironment,
        environment: String, method: String, url: String, resolvedURL: String? = nil,
        countdown: ConfirmationCountdown? = nil,
        style: ConfirmStyle = .countdown, returnCancels: Bool = true, teamMinimum: Int = 0,
        action: @escaping () -> Void
    ) {
        self.reason = reason
        self.environment = environment
        self.method = method
        self.url = url
        self.resolvedURL = resolvedURL
        self.countdown = countdown ?? ConfirmationCountdown()
        self.style = style
        self.returnCancels = returnCancels
        self.teamMinimum = teamMinimum
        self.action = action
    }

    func accept() {
        guard countdown.accept(), let run = action else { return }
        action = nil
        run()
    }

    func cancel() {
        countdown.cancel()
        action = nil
    }
}
