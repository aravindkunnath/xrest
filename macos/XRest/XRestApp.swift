import SwiftUI

@main
struct XRestApp: App {
    var body: some Scene {
        // One window per workspace (service directory). The window's value is the workspace URL;
        // a window without a value shows the welcome screen.
        WindowGroup("XRest", for: URL.self) { $workspaceURL in
            WorkspaceWindow(url: $workspaceURL)
        }
        .defaultSize(width: 1180, height: 760)
        .commands { WorkspaceCommands() }
    }
}
