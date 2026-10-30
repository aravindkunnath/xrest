import CoreGraphics

struct DensityMetrics: Equatable {
    let rowHeight: CGFloat
    let rowPadding: CGFloat
    let tabHeight: CGFloat
    /// Vertical padding inside a sidebar endpoint or scratch row.
    let sidebarRowPadding: CGFloat

    init(_ density: Density) {
        switch density {
        case .regular:
            rowHeight = 24
            rowPadding = 4
            tabHeight = 36
            sidebarRowPadding = 6
        case .compact:
            rowHeight = 20
            rowPadding = 2
            tabHeight = 30
            sidebarRowPadding = 4
        }
    }
}
