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

/// Opens the "New Service from OpenAPI" sheet of the focused window (works on the welcome screen too).
struct NewServiceAction {
    let run: () -> Void
}

private struct NewServiceActionKey: FocusedValueKey {
    typealias Value = NewServiceAction
}

extension FocusedValues {
    var newServiceAction: NewServiceAction? {
        get { self[NewServiceActionKey.self] }
        set { self[NewServiceActionKey.self] = newValue }
    }

    var workspaceModel: WorkspaceModel? {
        get { self[WorkspaceModelKey.self] }
        set { self[WorkspaceModelKey.self] = newValue }
    }

    var windowURLSetter: WindowURLSetter? {
        get { self[WindowURLSetterKey.self] }
        set { self[WindowURLSetterKey.self] = newValue }
    }
}
