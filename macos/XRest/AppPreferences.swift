import Foundation
import SwiftUI

enum AppearanceMode: String, CaseIterable, Identifiable {
    case system, light, dark
    var id: String { rawValue }

    var title: String { rawValue.capitalized }

    /// `nil` follows the system appearance.
    var colorScheme: ColorScheme? {
        switch self {
        case .system: return nil
        case .light: return .light
        case .dark: return .dark
        }
    }
}

enum ProxyMode: String, CaseIterable, Identifiable {
    case system, direct, custom
    var id: String { rawValue }

    var title: String { rawValue.capitalized }
}

/// App-wide preferences (Settings window), stored in `UserDefaults` and never in a workspace.
/// The same keys back `@AppStorage` in the views.
struct AppPreferences {
    enum Key {
        static let appearance = "appearance"
        static let zoom = "zoom"
        static let timeoutSeconds = "http.timeoutSeconds"
        static let followRedirects = "http.followRedirects"
        static let verifyTLS = "http.verifyTLS"
        static let proxyMode = "http.proxyMode"
        static let proxyURL = "http.proxyURL"
        static let maxVersions = "maxVersions"
    }

    static let zoomRange: ClosedRange<Double> = 0.8...2.0
    static let zoomSteps: [Double] = [0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0]
    static let defaultTimeoutSeconds = 30.0
    static let maxVersionsRange = 5...50

    var defaults: UserDefaults = .standard

    /// Zoom as read by fonts and editors anywhere in the app.
    static var currentZoom: Double { AppPreferences().zoom }

    /// The next (or previous) zoom step relative to `current`.
    static func stepped(from current: Double, up: Bool) -> Double {
        if up { return zoomSteps.first { $0 > current + 0.001 } ?? zoomSteps.last! }
        return zoomSteps.last { $0 < current - 0.001 } ?? zoomSteps.first!
    }

    var appearance: AppearanceMode {
        get { (defaults.string(forKey: Key.appearance)).flatMap(AppearanceMode.init) ?? .system }
        nonmutating set { defaults.set(newValue.rawValue, forKey: Key.appearance) }
    }

    var zoom: Double {
        get {
            guard let v = defaults.object(forKey: Key.zoom) as? Double else { return 1.0 }
            return min(max(v, Self.zoomRange.lowerBound), Self.zoomRange.upperBound)
        }
        nonmutating set { defaults.set(newValue, forKey: Key.zoom) }
    }

    var timeoutSeconds: Double {
        get {
            guard let v = defaults.object(forKey: Key.timeoutSeconds) as? Double, v > 0 else {
                return Self.defaultTimeoutSeconds
            }
            return v
        }
        nonmutating set { defaults.set(newValue, forKey: Key.timeoutSeconds) }
    }

    var followRedirects: Bool {
        get { defaults.object(forKey: Key.followRedirects) as? Bool ?? true }
        nonmutating set { defaults.set(newValue, forKey: Key.followRedirects) }
    }

    var verifyTLS: Bool {
        get { defaults.object(forKey: Key.verifyTLS) as? Bool ?? true }
        nonmutating set { defaults.set(newValue, forKey: Key.verifyTLS) }
    }

    var proxyMode: ProxyMode {
        get { (defaults.string(forKey: Key.proxyMode)).flatMap(ProxyMode.init) ?? .system }
        nonmutating set { defaults.set(newValue.rawValue, forKey: Key.proxyMode) }
    }

    var proxyURL: String {
        get { defaults.string(forKey: Key.proxyURL) ?? "" }
        nonmutating set { defaults.set(newValue, forKey: Key.proxyURL) }
    }

    var maxVersions: Int {
        get {
            let v = defaults.object(forKey: Key.maxVersions) as? Int ?? Self.maxVersionsRange.upperBound
            return min(max(v, Self.maxVersionsRange.lowerBound), Self.maxVersionsRange.upperBound)
        }
        nonmutating set { defaults.set(newValue, forKey: Key.maxVersions) }
    }

    /// The transport defaults handed to the core (`set_http_options`).
    var httpOptions: HttpOptions {
        let proxy: String?
        switch proxyMode {
        case .system: proxy = nil
        case .direct: proxy = ""
        case .custom:
            let url = proxyURL.trimmingCharacters(in: .whitespaces)
            // An unfinished entry (blank or just the scheme) must not become a broken proxy.
            proxy = ["", "http://", "https://"].contains(url) ? nil : url
        }
        return HttpOptions(
            timeoutMs: UInt64((timeoutSeconds * 1000).rounded()), followRedirects: followRedirects,
            verifyTls: verifyTLS, proxy: proxy)
    }
}
