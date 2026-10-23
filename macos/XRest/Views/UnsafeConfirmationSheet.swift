import SwiftUI

/// Confirmation shown before sending to an unsafe environment. Auto-cancels after the countdown.
struct UnsafeConfirmationSheet: View {
    let pending: PendingConfirmation
    let dismiss: () -> Void

    var body: some View {
        let countdown = pending.countdown
        VStack(alignment: .leading, spacing: 14) {
            Text("Send to \(pending.environment)?").font(.headline)
            Text("\(pending.environment) is marked unsafe. This request will be sent for real.")
                .font(.subheadline)
                .foregroundStyle(.secondary)
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

            VStack(alignment: .leading, spacing: 4) {
                ProgressView(value: countdown.progress).tint(Theme.red)
                    .animation(.linear(duration: 1), value: countdown.progress)
                Text("Cancels automatically in \(countdown.remaining)s")
                    .font(.ui(11)).foregroundStyle(Theme.muted)
            }

            HStack {
                Spacer()
                Button("Cancel") { pending.cancel(); dismiss() }
                    .keyboardShortcut(.defaultAction)
                Button("I Accept the Risk", role: .destructive) {
                    pending.accept(); dismiss()
                }
                .foregroundStyle(Theme.red)
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
            if countdown.state == .cancelled { dismiss() }
        }
        .onExitCommand { pending.cancel(); dismiss() }
    }
}
