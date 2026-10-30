import Foundation
import Observation
import SwiftUI

extension EnvironmentConfig {
    /// An environment with default rules: no color, confirm every request, no team minimum.
    init(name: String, isUnsafe: Bool, variables: [NameValue]) {
        self.init(
            name: name, isUnsafe: isUnsafe, variables: variables, color: nil, confirm: .every,
            minCountdownSeconds: 0)
    }
}

extension NameValue {
    static func plain(_ name: String, _ value: String) -> NameValue {
        NameValue(name: name, value: value, enabled: true, secretKey: nil, kind: "plain")
    }
}

/// Color names shared with the core (`color_palette()`), mapped to app colors that work in light
/// and dark mode.
enum PaletteColor {
    static let names: [String] = colorPalette()

    static func color(_ name: String?) -> Color? {
        switch name {
        case "red": Theme.red
        case "orange": Theme.orange
        case "amber": Theme.amber
        case "green": Theme.green
        case "teal": Theme.teal
        case "blue": Theme.blue
        case "purple": Theme.purple
        case "pink": Theme.pink
        case "graphite": Theme.graphite
        default: nil
        }
    }

    static func title(_ name: String) -> String { name.prefix(1).uppercased() + name.dropFirst() }
}

/// Workspace icon names shared with the core (`workspace_icons()`), mapped to SF Symbols.
enum WorkspaceIcon {
    static let names: [String] = workspaceIcons()

    static func symbol(_ name: String?) -> String? {
        switch name {
        case "card": "creditcard"
        case "server": "server.rack"
        case "person": "person"
        case "box": "shippingbox"
        case "globe": "globe"
        case "bolt": "bolt"
        default: nil
        }
    }
}

extension VariableLayer {
    var title: String {
        switch self {
        case .local: "Local override"
        case .environment: "Environment"
        case .workspace: "Workspace"
        case .global: "Global"
        case .builtIn: "Built-in"
        }
    }

    /// Where the value is stored and who sees it.
    var scope: String {
        switch self {
        case .local: "Only you · this Mac (.xrest/overrides.yaml)"
        case .environment: "Shared · environments.yaml"
        case .workspace: "Shared · every environment (variables.yaml)"
        case .global: "Only you · every workspace"
        case .builtIn: "Computed when the request is sent"
        }
    }

    var badgeColor: Color {
        switch self {
        case .local: Theme.purple
        case .environment: Theme.blue
        case .workspace: Theme.green
        case .global, .builtIn: Theme.muted
        }
    }
}

/// The user's global variables, shared by every open workspace window and stored in
/// Application Support. Attached to each workspace's core on open.
@MainActor
@Observable
final class GlobalVariablesStore {
    @ObservationIgnored let globals: XRestGlobals?
    private(set) var rows: [NameValue]
    private(set) var error: String?

    static var defaultURL: URL {
        let base =
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Application Support")
        return base.appendingPathComponent("XRest/variables.yaml")
    }

    static let shared = GlobalVariablesStore(path: defaultURL)

    init(path: URL) {
        do {
            let g = try XRestGlobals.open(path: path.path)
            globals = g
            rows = g.variables()
            error = nil
        } catch {
            globals = nil
            rows = []
            self.error = "Could not read global variables: \(describe(error))"
        }
    }

    func save(_ rows: [NameValue]) throws {
        guard let globals else { throw XRestError.Io(message: error ?? "Global variables are unavailable.") }
        try globals.save(variables: rows)
        self.rows = globals.variables()
        error = nil
    }
}

/// Pure presentation of `variable_report` for the Variables panel, unit tested without UI.
struct VariableRow: Identifiable, Equatable {
    let name: String
    /// What the request will use, as shown (secrets masked).
    let display: String
    /// The layer the value comes from; `nil` when every row is disabled.
    let source: VariableLayer?
    /// A value from a less specific layer that this one hides, shown struck through.
    let shadowed: String?
    let isSecret: Bool
    let layers: [LayerValue]

    var id: String { name }

    static let secretMask = "••••••"

    init(_ report: VariableReport) {
        name = report.name
        layers = report.values
        source = report.winner
        let winner = report.values.first { $0.layer == report.winner && $0.enabled }
        isSecret = winner?.secretKey != nil
        display = winner.map(Self.show) ?? "Not set"
        if let w = winner, w.secretKey == nil,
            let below = report.values.drop(while: { $0.layer != w.layer }).dropFirst()
                .first(where: { $0.enabled && $0.secretKey == nil })
        {
            shadowed = below.value
        } else {
            shadowed = nil
        }
    }

    static func show(_ v: LayerValue) -> String {
        if let key = v.secretKey { return "\(secretMask) (Keychain: \(key))" }
        return v.value.isEmpty ? "(empty)" : v.value
    }

    var localOverride: LayerValue? { layers.first { $0.layer == .local } }
}

/// Sources in the order the inspector lists them, with each one's row if present.
enum VariableInspector {
    static let order: [VariableLayer] = [.local, .environment, .workspace, .global]

    struct Entry: Identifiable, Equatable {
        let layer: VariableLayer
        let value: LayerValue?
        let inUse: Bool
        var id: VariableLayer { layer }
        /// Set but hidden by a more specific layer (or disabled).
        var overridden: Bool { value != nil && !inUse }
    }

    static func entries(for row: VariableRow) -> [Entry] {
        order.map { layer in
            let value = row.layers.first { $0.layer == layer }
            return Entry(layer: layer, value: value, inUse: row.source == layer && value?.enabled == true)
        }
    }
}
