import SwiftUI

/// Lets menu commands reach the model of the focused window.
private struct WorkspaceModelKey: FocusedValueKey {
    typealias Value = WorkspaceModel
}

/// Lets "Open Workspace…" reuse a focused welcome window instead of opening a second one.
struct WindowURLSetter {
    let set: (URL) -> Void
}

private struct WindowURLSetterKey: FocusedValueKey {
    typealias Value = WindowURLSetter
}

extension FocusedValues {
    var workspaceModel: WorkspaceModel? {
        get { self[WorkspaceModelKey.self] }
        set { self[WorkspaceModelKey.self] = newValue }
    }

    var windowURLSetter: WindowURLSetter? {
        get { self[WindowURLSetterKey.self] }
        set { self[WindowURLSetterKey.self] = newValue }
    }
}
