import Foundation
import Observation

/// Countdown behind the unsafe-environment confirmation. Time is injected: the owner calls
/// `tick()` once per second (the sheet does it from a task; tests call it directly).
/// Reaching zero cancels; `accept()` is valid only while running and succeeds at most once.
@MainActor
@Observable
final class ConfirmationCountdown {
    enum State: Equatable {
        case running(remaining: Int)
        case accepted
        case cancelled
    }

    static let defaultDuration = 10

    let duration: Int
    private(set) var state: State

    init(duration: Int = ConfirmationCountdown.defaultDuration) {
        self.duration = duration
        state = .running(remaining: duration)
    }

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
        state = r <= 1 ? .cancelled : .running(remaining: r - 1)
    }

    /// Returns true only for the one accept that happens while running.
    @discardableResult
    func accept() -> Bool {
        guard isRunning else { return false }
        state = .accepted
        return true
    }

    func cancel() {
        if isRunning { state = .cancelled }
    }

    func reset() {
        state = .running(remaining: duration)
    }
}

/// An action waiting behind the unsafe confirmation. Any send path (Send button, ⌘↩, retry after
/// `ConfirmationRequired`, Test Preflight) wraps its work in one of these; `action` runs only if
/// the user accepts while the countdown is still running.
@MainActor
@Observable
final class PendingConfirmation: Identifiable {
    let id = UUID()
    let environment: String
    let method: String
    let url: String
    /// The URL with variables resolved and secrets masked, when known.
    let resolvedURL: String?
    let countdown: ConfirmationCountdown
    @ObservationIgnored private var action: (() -> Void)?

    init(
        environment: String, method: String, url: String, resolvedURL: String? = nil,
        countdown: ConfirmationCountdown? = nil,
        action: @escaping () -> Void
    ) {
        self.environment = environment
        self.method = method
        self.url = url
        self.resolvedURL = resolvedURL
        self.countdown = countdown ?? ConfirmationCountdown()
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
