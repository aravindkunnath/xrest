# Settings window redesign (macOS) — design

Date: 2026-10-25 · Status: awaiting review

## Purpose

Replace the plain SwiftUI `Form` in `SettingsView` with the toolbar-tab, card-style Settings window
from the "XRest Settings" design canvas, and add the preferences that design shows: editor font,
wrap and line numbers, density, and the unsafe-environment confirmation options. Every new control
must change real app behavior.

## Scope

In:
- Four tabs: **Appearance, Requests, Versions, Safety**. The existing prefs (theme, zoom, timeout,
  redirects, TLS, proxy, max versions) move into them unchanged.
- New prefs: editor font family and size, wrap long lines, line numbers, density, confirmation style,
  countdown length, "Return key cancels", "warn about unresolved variables".
- A "Preview Dialog" button that opens the real confirmation sheet with a sample request.

Out:
- `~/.xrest/settings.yaml` storage. Prefs stay in `UserDefaults` (see Storage).
- Workspace-level settings window (environment colors/icons, minimum countdown, drag-reorder).
- The "Response syntax colors" picker and accent color (follows macOS).
- General and Variables tabs: the design never draws them; Variables stays workspace-level.
- The Rust core and FFI. Nothing here changes cross-platform behavior.

## Storage

Keep `AppPreferences` as the single typed accessor over `UserDefaults`. New values are added with the
same pattern: a `Key`, a default, clamping on read, and a nonmutating setter. A new observable
`PreferencesStore` (modelled on `ZoomStore`) republishes the values views need live (editor font,
wrap, line numbers, density) so only the views that show them re-render and open panels keep their
unsaved edits. Zoom stays in `ZoomStore`.

## Units

### Pure model (no UI, unit-tested first)

- `AppPreferences` additions:

  | Pref | Type | Default | Range / values |
  |---|---|---|---|
  | `editorFontFamily` | String | `""` (system mono) | any installed family; unknown falls back |
  | `editorFontSize` | Int | 12 | 9...24, replaces the hard-coded `12` in `CodeEditor.fontSize` |
  | `wrapLines` | Bool | true | |
  | `showLineNumbers` | Bool | true | |
  | `density` | `Density` | `.regular` | `.compact`, `.regular` |
  | `confirmStyle` | `ConfirmStyle` | `.countdown` | `.countdown`, `.confirm`, `.typeName` |
  | `confirmCountdownSeconds` | Int | 10 | 1...10 (10 keeps today's behavior) |
  | `returnCancels` | Bool | true | matches today's `.defaultAction` on Cancel |
  | `warnUnresolved` | Bool | true | |

- `ConfirmStyle` and `Density` are `String` raw-value enums like `AppearanceMode`.
- `ConfirmationPolicy`: a pure value type taking `style`, `countdownSeconds`, `elapsed` and
  `typedText` against the environment name, and answering `canSend`, `sendLabelSuffix` and
  `progress`. It replaces the countdown-only logic that `UnsafeConfirmationSheet` encodes today.
  `ConfirmationCountdown` stays for the `.countdown` style and is constructed with the pref's length.
- `DensityMetrics`: a table mapping `Density` to row height, vertical padding and tab height.
- `UnresolvedGuard`: given a request's `preview.unresolved` and `warnUnresolved`, returns whether to
  ask before sending.

### Settings shell (`macos/XRest/Views/Settings/`)

- `SettingsSection` enum (`appearance, requests, versions, safety`) with title and SF Symbol.
- `SettingsView` is the window root: a toolbar tab strip bound to a `SettingsSection` selection
  (last tab remembered with `@AppStorage("settings.lastTab")`) and the selected tab's view below.
- Reusable components following the design tokens: `SettingsCard`, `SettingsRow` (title, optional
  hint, trailing control), `SettingsSwitch`, `SettingsStepper`, `SettingsSegmented`,
  `ThemeSwatchPicker`. Colors come from `Theme` so light and dark both work.
- One file per tab: `AppearanceTab`, `RequestsTab`, `VersionsTab`, `SafetyTab`. Each binds to
  `AppPreferences` through `@AppStorage` with the same keys.

### Behavior wiring

- **Safety.** `WorkspaceModel.presentConfirmation` builds the `PendingConfirmation` with the pref
  style and length. `UnsafeConfirmationSheet` renders by style: countdown bar and a locked
  "Send (n)" button; confirm with an immediately active button; type-the-name with a text field
  that must equal the environment name. With `returnCancels` on, Cancel is the default action;
  off, the destructive button is. Preview Dialog presents the same sheet with a sample request and
  a no-op action.
- **Unresolved variables.** Before the unsafe confirmation and the send, `UnresolvedGuard` is
  consulted; when it says ask, the existing confirmation flow is reused with an "unresolved:" line.
  Requests with no unresolved variables are unaffected.
- **Editor.** `CodeEditor` reads font family and size from `PreferencesStore` (times zoom), toggles
  `textContainer.widthTracksTextView` for wrap, and shows a line-number ruler when enabled. The
  response body view reads the same prefs.
- **Density.** `KeyValueEditor` rows, sidebar rows and `UnderlineTabs` take their heights and
  padding from `DensityMetrics`.

## Error handling

- Out-of-range or corrupt stored values clamp on read; an unknown font family falls back to the
  system monospaced face; unknown enum raw values fall back to the default.
- Turning TLS verification off keeps today's inline warning.

## Testing

Test-first, in `macos/XRestTests/`:
- `AppPreferencesTests`: defaults, round-trip, clamping for each new pref.
- `ConfirmationPolicyTests`: each style's `canSend` through its states, countdown length, name
  match trimming and case sensitivity.
- `UnresolvedGuardTests`.
- `DensityMetricsTests`.
- `PreferencesStoreTests`: publishes on change, no republish when unchanged.
- `WorkspaceModelTests`: confirmation built with pref style and length; unresolved warning gates a
  send only when enabled.

Views stay thin and are verified by building and launching the app.

## Delivery order

1. Pure model and its tests.
2. Settings shell and components, existing prefs moved in.
3. Safety tab and its behavior wiring.
4. Editor prefs.
5. Density.

Each step builds, passes tests, and is its own commit.
