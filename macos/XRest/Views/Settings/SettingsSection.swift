import SwiftUI

enum SettingsSection: String, CaseIterable, Identifiable {
    case appearance, requests, variables, versions, safety

    var id: String { rawValue }

    var title: String {
        switch self {
        case .appearance: "Appearance"
        case .requests: "Requests"
        case .variables: "Variables"
        case .versions: "Versions"
        case .safety: "Safety"
        }
    }

    var symbol: String {
        switch self {
        case .appearance: "circle.lefthalf.filled"
        case .requests: "arrow.right.to.line"
        case .variables: "curlybraces"
        case .versions: "clock.arrow.circlepath"
        case .safety: "shield"
        }
    }
}
