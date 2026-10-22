import SwiftUI

/// Per-request transport options. "Default" leaves the option unset so the app default applies.
struct OptionsEditor: View {
    @Binding var options: RequestOptions

    private enum Toggle3: String, CaseIterable, Identifiable {
        case inherit = "Default"
        case on = "On"
        case off = "Off"
        var id: String { rawValue }

        init(_ value: Bool?) {
            switch value {
            case .none: self = .inherit
            case .some(true): self = .on
            case .some(false): self = .off
            }
        }

        var value: Bool? {
            switch self {
            case .inherit: return nil
            case .on: return true
            case .off: return false
            }
        }
    }

    private enum ProxyMode: String, CaseIterable, Identifiable {
        case system = "System"
        case direct = "Direct"
        case custom = "Custom"
        var id: String { rawValue }
    }

    var body: some View {
        Form {
            TextField("Timeout (seconds)", text: timeoutText, prompt: Text("Default (30)"))
                .frame(maxWidth: 280)

            Picker("Follow redirects", selection: tri(\.followRedirects)) {
                ForEach(Toggle3.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .frame(maxWidth: 280)

            Picker("Verify TLS certificates", selection: tri(\.verifyTls)) {
                ForEach(Toggle3.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .frame(maxWidth: 280)
            if options.verifyTls == false {
                Text("Invalid certificates will be accepted for this request.")
                    .font(.caption)
                    .foregroundStyle(.orange)
            }

            Picker("Proxy", selection: proxyMode) {
                ForEach(ProxyMode.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .frame(maxWidth: 280)
            if proxyMode.wrappedValue == .custom {
                TextField("Proxy URL", text: proxyURL, prompt: Text("http://proxy:8080"))
                    .font(.system(.body, design: .monospaced))
                    .frame(maxWidth: 360)
            }
        }
        .formStyle(.grouped)
    }

    private func tri(_ key: WritableKeyPath<RequestOptions, Bool?>) -> Binding<Toggle3> {
        Binding(
            get: { Toggle3(options[keyPath: key]) },
            set: { options[keyPath: key] = $0.value })
    }

    /// Seconds as text; blank or invalid means the default.
    private var timeoutText: Binding<String> {
        Binding(
            get: {
                guard let ms = options.timeoutMs else { return "" }
                return ms % 1000 == 0 ? String(ms / 1000) : String(Double(ms) / 1000)
            },
            set: { text in
                let trimmed = text.trimmingCharacters(in: .whitespaces)
                if let secs = Double(trimmed), secs > 0 {
                    options.timeoutMs = UInt64((secs * 1000).rounded())
                } else {
                    options.timeoutMs = nil
                }
            })
    }

    private var proxyMode: Binding<ProxyMode> {
        Binding(
            get: {
                switch options.proxy {
                case .none: return .system
                case .some(let p) where p.isEmpty: return .direct
                case .some: return .custom
                }
            },
            set: { mode in
                switch mode {
                case .system: options.proxy = nil
                case .direct: options.proxy = ""
                case .custom:
                    if options.proxy?.isEmpty ?? true { options.proxy = "http://" }
                }
            })
    }

    private var proxyURL: Binding<String> {
        Binding(
            get: { options.proxy ?? "" },
            set: { options.proxy = $0.isEmpty ? "http://" : $0 })
    }
}
