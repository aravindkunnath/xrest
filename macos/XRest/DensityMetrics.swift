import CoreGraphics

struct DensityMetrics: Equatable {
    let rowHeight: CGFloat
    let rowPadding: CGFloat
    let tabHeight: CGFloat

    init(_ density: Density) {
        switch density {
        case .regular:
            rowHeight = 24
            rowPadding = 4
            tabHeight = 32
        case .compact:
            rowHeight = 20
            rowPadding = 2
            tabHeight = 26
        }
    }
}
