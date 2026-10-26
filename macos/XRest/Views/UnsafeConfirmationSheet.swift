import SwiftUI

/// Confirmation shown before an unsafe or unresolved send. What unlocks Send depends on the
/// user's confirm style (countdown, confirm, or typing the environment name).
struct UnsafeConfirmationSheet: View {
    let pending: PendingConfirmation
    let dismiss: () -> Void

    @State private var typed = ""

    private var presentation: UnsafeConfirmationPresentation {
        UnsafeConfirmationPresentation(
            reason: pending.reason, style: pending.style, environment: pending.environment,
            returnCancels: pending.returnCancels, remaining: pending.countdown.remaining,
            typed: typed)
    }

    var body: some View {
        let countdown = pending.countdown
        let p = presentation
        VStack(alignment: .leading, spacing: 14) {
            Text(p.title).font(.headline)
            bodyText
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(pending.method).font(.mono(12, weight: .bold)).foregroundStyle(Theme.red)
                VStack(alignment: .leading, spacing: 3) {
                    Text(pending.resolvedURL ?? pending.url)
                        .font(.mono(12)).lineLimit(3).textSelection(.enabled)
                    if let resolved = pending.resolvedURL, resolved != pending.url {
                        Text(pending.url)
                            .font(.mono(11)).foregroundStyle(Theme.muted)
                            .lineLimit(2).textSelection(.enabled)
                    }
                }
            }
            .padding(10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Theme.redFill, in: RoundedRectangle(cornerRadius: 6))
            .overlay(RoundedRectangle(cornerRadius: 6).stroke(Theme.redBorder))

            if p.showsTypeField {
                VStack(alignment: .leading, spacing: 4) {
                    Text(p.typePrompt).font(.ui(11)).foregroundStyle(Theme.muted)
                    TextField(p.typePrompt, text: $typed, prompt: Text(pending.environment))
                        .labelsHidden()
                        .textFieldStyle(.roundedBorder)
                        .font(.mono(12))
                }
            }

            if p.showsCountdown {
                VStack(alignment: .leading, spacing: 4) {
                    ProgressView(value: countdown.progress).tint(Theme.red)
                        .animation(.linear(duration: 1), value: countdown.progress)
                    Text(p.countdownCaption)
                        .font(.ui(11)).foregroundStyle(Theme.muted)
                }
            }

            HStack {
                Spacer()
                cancelButton(isDefault: p.defaultButton == .cancel)
                sendButton(p)
            }
        }
        .padding(20)
        .frame(width: 460)
        .task {
            while countdown.isRunning {
                try? await Task.sleep(for: .seconds(1))
                if Task.isCancelled { return }
                countdown.tick()
            }
        }
        .onExitCommand { pending.cancel(); dismiss() }
    }

    @ViewBuilder
    private var bodyText: some View {
        switch pending.reason {
        case .unsafeEnvironment:
            Text("\(pending.environment) is marked unsafe. This request will be sent for real.")
                .font(.subheadline)
                .foregroundStyle(.secondary)
        case .unresolved(let names):
            VStack(alignment: .leading, spacing: 4) {
                Text("These variables have no value and will be sent as written:")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                Text(names.joined(separator: ", "))
                    .font(.mono(12)).foregroundStyle(Theme.red)
                    .textSelection(.enabled)
            }
        }
    }

    @ViewBuilder
    private func cancelButton(isDefault: Bool) -> some View {
        let button = Button("Cancel") { pending.cancel(); dismiss() }
        if isDefault {
            button.keyboardShortcut(.defaultAction)
        } else {
            button
        }
    }

    @ViewBuilder
    private func sendButton(_ p: UnsafeConfirmationPresentation) -> some View {
        let button = Button(p.sendLabel, role: .destructive) {
            // Return can reach a locked button's shortcut path; never act unless unlocked.
            guard presentation.sendEnabled else { return }
            pending.accept(); dismiss()
        }
        .foregroundStyle(Theme.red)
        .disabled(!p.sendEnabled)
        if p.defaultButton == .send {
            button.keyboardShortcut(.defaultAction)
        } else {
            button
        }
    }
}
