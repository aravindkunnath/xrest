import SwiftUI

@main
struct XRestApp: App {
    init() {
        // Windows are reopened from XRest's own session files (selection + drafts), so the
        // system's window restoration must not open a second set.
        UserDefaults.standard.set(false, forKey: "NSQuitAlwaysKeepsWindows")
    }

    var body: some Scene {
        // One window per workspace (service directory). The window's value is the workspace URL;
        // a window without a value shows the welcome screen.
        WindowGroup("XRest", for: URL.self) { $workspaceURL in
            WorkspaceWindow(url: $workspaceURL)
        }
        .windowStyle(.hiddenTitleBar)
        .defaultSize(width: 1280, height: 800)
        .commands { WorkspaceCommands() }

        Settings { SettingsView() }
    }
}
