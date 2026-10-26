# Settings Window Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the plain Settings `Form` with the design's toolbar-tab, card-style window and add the editor, density and safety preferences, each wired to real behavior.

**Architecture:** Typed prefs live in `AppPreferences` over `UserDefaults`; a new observable `PreferencesStore` republishes the live ones. Behavior rules are pure types (`ConfirmationPolicy`, `UnresolvedGuard`, `DensityMetrics`) tested first. Thin SwiftUI views in `Views/Settings/` read prefs through `@AppStorage`.

**Tech Stack:** Swift 5.9, SwiftUI + AppKit, XCTest, XcodeGen (`macos/project.yml`). No Rust or FFI changes.

**Spec:** `docs/superpowers/specs/2026-10-25-settings-window-design.md`

## Global Constraints

- Four tabs only: Appearance, Requests, Versions, Safety. No General or Variables tab.
- Prefs stay in `UserDefaults` via `AppPreferences`; never in a workspace, never `settings.yaml`.
- Defaults: `editorFontSize` 12 (range 9...24), `wrapLines` true, `showLineNumbers` true, `density` `.regular`, `confirmStyle` `.countdown`, `confirmCountdownSeconds` 3 (range 1...10), `returnCancels` true, `warnUnresolved` true, `editorFontFamily` `""` (system mono).
- Out-of-range values clamp on read; unknown enum raw values and unknown font families fall back to the default.
- Never hand-edit `macos/XRest/Bindings` or `macos/XRest.xcodeproj`; after adding files run `cd macos && xcodegen` and do not commit the generated project.
- Match surrounding code: `///` doc comments only where non-obvious, `@MainActor` for model types, tests in `macos/XRestTests/`.
- Test command: `cd macos && xcodebuild test -project XRest.xcodeproj -scheme XRest -only-testing:XRestTests/<Class>`; "passes" means `** TEST SUCCEEDED **`.
- Each task ends with a commit; commit messages end with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.

## Review Focus

- Corrupt or hand-edited defaults (font size 999, empty or unknown font family, bogus enum strings): clamp or fall back, never crash.
- Type-the-name confirmation: surrounding whitespace is trimmed, case must match exactly, empty never passes.
- Countdown finishing must unlock Send and leave the dialog open; it must not cancel or auto-send.
- A request with no unresolved variables, or with `warnUnresolved` off, sends exactly as before.
- Wrap off with a very long line, and line numbers on an empty body, must not crash or mis-draw the editor.

---

## File Structure

- Modify `macos/XRest/AppPreferences.swift`: new enums and accessors; add `PreferencesStore` below `ZoomStore`.
- Create `macos/XRest/ConfirmationPolicy.swift`: pure confirmation rules.
- Modify `macos/XRest/ConfirmationCountdown.swift`: unlock instead of cancel; `PendingConfirmation` gains `style`, `reason`.
- Create `macos/XRest/DensityMetrics.swift`, `macos/XRest/UnresolvedGuard.swift`.
- Create `macos/XRest/Views/Settings/`: `SettingsSection.swift`, `SettingsComponents.swift`, `AppearanceTab.swift`, `RequestsTab.swift`, `VersionsTab.swift`, `SafetyTab.swift`, `SettingsView.swift` (moved from `Views/SettingsView.swift`).
- Modify `WorkspaceModel.swift`, `Views/UnsafeConfirmationSheet.swift`, `Views/CodeEditor.swift`, `Views/KeyValueEditor.swift`, `Views/SidebarView.swift`, `Views/WorkspaceView.swift` (tabs).
- Tests (create): `ConfirmationPolicyTests`, `DensityMetricsTests`, `UnresolvedGuardTests`, `PreferencesStoreTests`; extend `AppPreferencesTests`, `ConfirmationCountdownTests`, `WorkspaceModelTests`.

---

### Task 1: New preferences in AppPreferences

**Files:**
- Modify: `macos/XRest/AppPreferences.swift`
- Test: `macos/XRestTests/AppPreferencesTests.swift`

**Interfaces:**
- Produces: `enum Density: String, CaseIterable, Identifiable { case compact, regular }`, `enum ConfirmStyle: String, CaseIterable, Identifiable { case countdown, confirm, typeName }` (both with `title`), new `AppPreferences.Key` constants `editorFontFamily, editorFontSize, wrapLines, showLineNumbers, density, confirmStyle, confirmCountdownSeconds, returnCancels, warnUnresolved` (string values `"editor.fontFamily"`, `"editor.fontSize"`, `"editor.wrapLines"`, `"editor.showLineNumbers"`, `"density"`, `"safety.confirmStyle"`, `"safety.countdownSeconds"`, `"safety.returnCancels"`, `"safety.warnUnresolved"`), static ranges `editorFontSizeRange = 9...24`, `countdownRange = 1...10`, and instance properties with the types in the spec table.

- [ ] **Step 1: Write failing tests** in `AppPreferencesTests`: `testNewDefaults` (asserts every Global Constraints default), `testNewRoundTrip` (set each, read back), `testNewClamping` (`editorFontSize = 999` reads 24, `= 1` reads 9; `confirmCountdownSeconds = 0` reads 1, `= 99` reads 10), `testBogusEnumFallsBack` (raw `"nonsense"` stored under `density` and `confirmStyle` reads `.regular` and `.countdown`).
- [ ] **Step 2: Run** `-only-testing:XRestTests/AppPreferencesTests`. Expected: build FAIL (members missing).
- [ ] **Step 3: Implement** the enums, keys, ranges and clamped accessors in `AppPreferences.swift`, following the existing `maxVersions` pattern (`defaults.object(forKey:) as? Int ?? default`, clamp, nonmutating setter).
- [ ] **Step 4: Run** the same command. Expected: PASS.
- [ ] **Step 5: Commit** `feat(macos): editor, density and safety preferences`.

### Task 2: ConfirmationPolicy and unlocking countdown

**Files:**
- Create: `macos/XRest/ConfirmationPolicy.swift`
- Modify: `macos/XRest/ConfirmationCountdown.swift`
- Test: `macos/XRestTests/ConfirmationPolicyTests.swift`, `macos/XRestTests/ConfirmationCountdownTests.swift`

**Interfaces:**
- Consumes: `ConfirmStyle` from Task 1.
- Produces:
  - `enum ConfirmationPolicy` with `static func countdownDuration(style: ConfirmStyle, seconds: Int) -> Int` (seconds for `.countdown`, else 0) and `static func canSend(style: ConfirmStyle, remaining: Int, typed: String, environment: String) -> Bool`.
  - `ConfirmationCountdown.State` becomes `running(remaining:)`, `ready`, `accepted`, `cancelled`; `tick()` moves `running(1)` to `.ready` (no longer `.cancelled`); `accept()` succeeds from `.running` or `.ready`; `var isReady: Bool` (true for `.ready` and for a duration-0 countdown, which starts `.ready`); `isRunning` stays true only while counting.

- [ ] **Step 1: Write failing tests.** `ConfirmationPolicyTests`: `.countdown` blocks while `remaining > 0` and allows at 0; `.confirm` always allows; `.typeName` allows only when `typed.trimmingCharacters(in: .whitespaces) == environment` (cases: `" PROD "` passes for `PROD`, `"prod"` fails, `""` fails); `countdownDuration` returns the seconds only for `.countdown`. `ConfirmationCountdownTests`: replace the "reaching zero cancels" test with `testReachingZeroUnlocksWithoutCancelling` (after `duration` ticks `state == .ready`, `accept()` returns true); `testZeroDurationStartsReady`.
- [ ] **Step 2: Run** both classes. Expected: FAIL.
- [ ] **Step 3: Implement** `ConfirmationPolicy` and the countdown state change; update any caller that switched on `.cancelled` after a final tick (`UnsafeConfirmationSheet`'s `.task`) so it no longer dismisses on zero.
- [ ] **Step 4: Run** both classes plus `WorkspaceModelTests`. Expected: PASS.
- [ ] **Step 5: Commit** `feat(macos): confirmation policy; countdown unlocks instead of cancelling`.

### Task 3: DensityMetrics and UnresolvedGuard

**Files:**
- Create: `macos/XRest/DensityMetrics.swift`, `macos/XRest/UnresolvedGuard.swift`
- Test: `macos/XRestTests/DensityMetricsTests.swift`, `macos/XRestTests/UnresolvedGuardTests.swift`

**Interfaces:**
- Consumes: `Density` (Task 1).
- Produces: `struct DensityMetrics: Equatable { let rowHeight: CGFloat; let rowPadding: CGFloat; let tabHeight: CGFloat; init(_ density: Density) }` with regular = 24 / 4 / 32 and compact = 20 / 2 / 26 (the current key/value row is 24 high with 4 padding); `enum UnresolvedGuard { static func shouldAsk(unresolved: [String], enabled: Bool) -> Bool }`.

- [ ] **Step 1: Write failing tests**: `testRegularMatchesCurrentLayout` and `testCompactIsStrictlySmaller` for each field; `testAsksOnlyWhenEnabledAndNonEmpty` (four combinations).
- [ ] **Step 2: Run** both classes. Expected: FAIL.
- [ ] **Step 3: Implement** both types.
- [ ] **Step 4: Run.** Expected: PASS.
- [ ] **Step 5: Commit** `feat(macos): density metrics and unresolved-variable guard`.

### Task 4: PreferencesStore

**Files:**
- Modify: `macos/XRest/AppPreferences.swift`
- Test: `macos/XRestTests/PreferencesStoreTests.swift`

**Interfaces:**
- Consumes: Task 1 accessors.
- Produces: `@Observable final class PreferencesStore` with `static let shared`, `init(defaults: UserDefaults = .standard)`, observed `editorFontFamily: String`, `editorFontSize: Int`, `wrapLines: Bool`, `showLineNumbers: Bool`, `density: Density`, and `func refresh(from defaults: UserDefaults = .standard)`. It subscribes to `UserDefaults.didChangeNotification` exactly like `ZoomStore` and only assigns a property when its value changed.

- [ ] **Step 1: Write failing tests**: `testReflectsDefaults`; `testRefreshPicksUpChange` (set via `AppPreferences(defaults:)`, call `refresh(from:)`, assert); `testNoRepublishWhenUnchanged` (use `withObservationTracking` on `density`, call `refresh` with an unchanged value, assert the change handler did not fire).
- [ ] **Step 2: Run** `-only-testing:XRestTests/PreferencesStoreTests`. Expected: FAIL.
- [ ] **Step 3: Implement** modelled on `ZoomStore`.
- [ ] **Step 4: Run.** Expected: PASS.
- [ ] **Step 5: Commit** `feat(macos): observable preferences store`.

### Task 5: Settings components

**Files:**
- Create: `macos/XRest/Views/Settings/SettingsComponents.swift`

**Interfaces:**
- Produces (all `View`s, colors from `Theme`, accessible labels required): `SettingsCard<Content>(@ViewBuilder content)` (rounded 10, `Theme.panel` fill, `Theme.border` stroke, rows separated by hairlines); `SettingsRow<Trailing>(_ title: String, hint: String? = nil, @ViewBuilder trailing:)` (min height 44); `SettingsSectionHeader(_ title: String)` (11pt, uppercase tracking); `SettingsSwitch(_ label: String, isOn: Binding<Bool>)` (role-switch toggle, 38x22, `Theme.amber` on); `SettingsStepper(_ label: String, value: Binding<Int>, range: ClosedRange<Int>, unit: String)` (− value + control); `SettingsSegmented<Value: Hashable>(_ label: String, selection: Binding<Value>, options: [(Value, String)])`; `ThemeSwatchPicker(selection: Binding<String>)` (three 84x56 previews for System/Light/Dark bound to `AppearanceMode` raw values).

- [ ] **Step 1: Implement** the components per the interfaces; add a SwiftUI `#Preview` for each in one `#Preview("Settings components")`.
- [ ] **Step 2: Run** `cd macos && xcodegen && xcodebuild -project XRest.xcodeproj -scheme XRest build`. Expected: `** BUILD SUCCEEDED **`.
- [ ] **Step 3: Commit** `feat(macos): settings card components`.

### Task 6: Settings shell with Appearance, Requests, Versions tabs

**Files:**
- Create: `macos/XRest/Views/Settings/SettingsSection.swift`, `AppearanceTab.swift`, `RequestsTab.swift`, `VersionsTab.swift`
- Move/Modify: `macos/XRest/Views/SettingsView.swift` to `Views/Settings/SettingsView.swift` (use `git mv`)
- Test: `macos/XRestTests/SettingsSectionTests.swift`

**Interfaces:**
- Consumes: Task 5 components, Task 1 keys.
- Produces: `enum SettingsSection: String, CaseIterable, Identifiable { case appearance, requests, versions, safety }` with `title` and `symbol` (SF Symbols `circle.lefthalf.filled`, `arrow.right.to.line`, `clock.arrow.circlepath`, `shield`); `SettingsView` (no parameters, still `Settings { SettingsView() }`): a top tab strip of 76pt-wide icon+label buttons, selection stored in `@AppStorage("settings.lastTab")`, fixed width 640, tab content in a scroll view on `Theme.bg`, `.preferredColorScheme` from the appearance pref. `AppearanceTab` holds the theme picker and zoom (editor and density controls arrive in Tasks 10 and 11). `RequestsTab` holds timeout, redirects, TLS (with its warning), proxy. `VersionsTab` holds the versions stepper. `SafetyTab` is a placeholder `Text` until Task 9.

- [ ] **Step 1: Write failing test** `SettingsSectionTests`: `testOrderAndTitles` (`allCases.map(\.title) == ["Appearance", "Requests", "Versions", "Safety"]`), `testUnknownStoredTabFallsBackToAppearance` (`SettingsSection(rawValue: "general") ?? .appearance == .appearance`).
- [ ] **Step 2: Run** the class. Expected: FAIL.
- [ ] **Step 3: Implement** the files above, moving every existing control from the old `Form` into its tab with identical `@AppStorage` keys and bounds.
- [ ] **Step 4: Run** the test class and the full build. Expected: PASS and `** BUILD SUCCEEDED **`. Launch the app, open ⌘, and confirm each existing control still changes its behavior (theme, zoom).
- [ ] **Step 5: Commit** `feat(macos): tabbed settings window`.

### Task 7: Model passes confirmation style and length

**Files:**
- Modify: `macos/XRest/ConfirmationCountdown.swift` (`PendingConfirmation`), `macos/XRest/WorkspaceModel.swift` (`presentConfirmation`)
- Test: `macos/XRestTests/WorkspaceModelTests.swift`

**Interfaces:**
- Consumes: `ConfirmationPolicy.countdownDuration`, `AppPreferences` (injected).
- Produces: `PendingConfirmation` gains `let style: ConfirmStyle` and `let returnCancels: Bool` (constructor params with defaults `.countdown` and `true`); `WorkspaceModel.init` gains `preferences: AppPreferences = AppPreferences()` stored as `private let preferences`; `presentConfirmation` builds the countdown with `ConfirmationPolicy.countdownDuration(style:seconds:)` from the prefs.

- [ ] **Step 1: Write failing tests** with a throwaway `UserDefaults` suite passed to `makeModel`: `testConfirmationUsesPreferenceStyleAndLength` (style `.countdown`, seconds 5: the pending confirmation's `countdown.duration == 5`, `style == .countdown`), `testConfirmStyleHasZeroDelay` (`duration == 0`, `countdown.isReady`). Use whatever existing helper puts the workspace in an unsafe environment in these tests; if none exists, add one to `makeModel`'s test file that selects an environment with `unsafe: true`.
- [ ] **Step 2: Run** `-only-testing:XRestTests/WorkspaceModelTests`. Expected: FAIL.
- [ ] **Step 3: Implement** the interfaces; the default `AppPreferences()` keeps `XRestApp` and existing call sites unchanged.
- [ ] **Step 4: Run** the class. Expected: PASS.
- [ ] **Step 5: Commit** `feat(macos): unsafe confirmation follows safety preferences`.

### Task 8: Unresolved-variable warning in the send path

**Files:**
- Modify: `macos/XRest/ConfirmationCountdown.swift` (`PendingConfirmation.Reason`), `macos/XRest/WorkspaceModel.swift` (`runGuarded`)
- Test: `macos/XRestTests/WorkspaceModelTests.swift`

**Interfaces:**
- Consumes: `UnresolvedGuard.shouldAsk`, `previewRequestURL(_:)`.
- Produces: `PendingConfirmation.Reason { case unsafeEnvironment; case unresolved([String]) }` with `let reason` (default `.unsafeEnvironment`). In `runGuarded` with no explicit `url` (the open draft): when the draft URL preview has unresolved names and the guard says ask, present a `.confirm`-style, zero-delay confirmation with `.unresolved(names)`; this runs before and in addition to the unsafe check, and accepting an unresolved prompt in an unsafe environment then shows the unsafe confirmation. Scope: the URL only (Test Preflight passes its own url and is not guarded).

- [ ] **Step 1: Write failing tests**: `testUnresolvedVariablePromptsWhenEnabled` (draft URL `{{MISSING}}/x`, pref on: `pendingConfirmation?.reason == .unresolved(["MISSING"])` and the action has not run), `testUnresolvedDoesNotPromptWhenDisabled`, `testNoPromptWhenAllResolved` (action runs immediately in a safe environment).
- [ ] **Step 2: Run** the class. Expected: FAIL.
- [ ] **Step 3: Implement** the guard call at the top of `runGuarded`.
- [ ] **Step 4: Run** the class. Expected: PASS.
- [ ] **Step 5: Commit** `feat(macos): warn before sending unresolved variables`.

### Task 9: Confirmation sheet by style, Safety tab, Preview Dialog

**Files:**
- Modify: `macos/XRest/Views/UnsafeConfirmationSheet.swift`
- Create: `macos/XRest/Views/Settings/SafetyTab.swift` (replace the placeholder)

**Interfaces:**
- Consumes: `PendingConfirmation` (`style`, `reason`, `returnCancels`, `countdown`), `ConfirmationPolicy.canSend`.
- Produces: the sheet shows, by `reason`, the title "Send to <env>?" or "Send with unresolved variables?" plus the names; by `style`: countdown bar and "Send to <env> (n)" label locked until `countdown.isReady`; confirm active immediately; type-the-name field with the label "Type <env> to confirm". The Send button's `disabled` is `!ConfirmationPolicy.canSend(...)`. With `returnCancels` true Cancel gets `.keyboardShortcut(.defaultAction)`; false gives it to the destructive button. `SafetyTab` holds: a `SettingsSegmented`-style radio list for `confirmStyle`; a stepper for `confirmCountdownSeconds` disabled unless countdown; `SettingsSwitch` for `returnCancels`; a separate card with `SettingsSwitch` for `warnUnresolved`; a "Preview Dialog" button presenting `UnsafeConfirmationSheet` in a `.sheet` with a sample `PendingConfirmation` (environment "PROD", `POST {{BASE_URL}}/v1/refunds`, no-op action) built from the current prefs.

- [ ] **Step 1: Implement** the sheet changes and `SafetyTab`.
- [ ] **Step 2: Run** the full test suite and build: `cd macos && xcodebuild test -project XRest.xcodeproj -scheme XRest`. Expected: `** TEST SUCCEEDED **`.
- [ ] **Step 3: Verify in the app:** in Settings › Safety, press Preview Dialog for each style; confirm countdown locks then unlocks without closing, confirm is immediate, type-name only enables on an exact match, and the Return key follows the switch.
- [ ] **Step 4: Commit** `feat(macos): safety settings tab and style-aware confirmation sheet`.

### Task 10: Editor preferences

**Files:**
- Modify: `macos/XRest/Views/CodeEditor.swift`, `macos/XRest/Views/Settings/AppearanceTab.swift`
- Test: `macos/XRestTests/CodeEditorFontTests.swift`

**Interfaces:**
- Consumes: `PreferencesStore.shared`.
- Produces: `CodeEditor.fontSize(zoom:)` gains a `base: Int` parameter (call sites pass the store's size); `static func font(family: String, size: CGFloat) -> NSFont` returning the named family when installed, else `NSFont.monospacedSystemFont` (this is what the highlighter uses); `CodeEditor` reads font, wrap and line numbers from the store during `body` construction (same observation trick as `zoom`); wrap off sets `textView.textContainer?.widthTracksTextView = false`, an unbounded container width and a horizontal scroller; line numbers use an `NSRulerView` subclass `LineNumberRuler`. `AppearanceTab` gains an Editor card (font family menu of installed monospaced families, size stepper 9...24, two switches).

- [ ] **Step 1: Write failing tests** `CodeEditorFontTests`: `testUnknownFamilyFallsBack` (`font(family: "No Such Font", size: 12).isFixedPitch`), `testSizeScalesWithZoom` (`fontSize(zoom: 1.5, base: 12) == 18`), `testBaseMatchesPreviousDefault` (`fontSize(zoom: 1, base: 12) == 12`).
- [ ] **Step 2: Run** the class. Expected: FAIL.
- [ ] **Step 3: Implement** the font helper and parameter; update `JSONHighlighter.apply`.
- [ ] **Step 4: Run** the class. Expected: PASS.
- [ ] **Step 5: Implement** wrap and `LineNumberRuler`, handling empty text (one line number) and wrap off with a very long line (no layout of the full line); add the Editor card.
- [ ] **Step 6: Verify in the app:** change font, size, wrap and line numbers in Settings with a request body and a response open; both update live.
- [ ] **Step 7: Commit** `feat(macos): editor font, wrap and line number preferences`.

### Task 11: Density

**Files:**
- Modify: `macos/XRest/Views/KeyValueEditor.swift`, `macos/XRest/Views/SidebarView.swift`, `macos/XRest/Views/Theme.swift` or the tab-strip view that owns `UnderlineTabs`, `macos/XRest/Views/Settings/AppearanceTab.swift`

**Interfaces:**
- Consumes: `DensityMetrics(PreferencesStore.shared.density)`.
- Produces: row heights and vertical padding in `KeyValueEditor`, sidebar rows and `UnderlineTabs` come from `DensityMetrics`; `AppearanceTab` gains a Layout card with a Compact/Regular `SettingsSegmented` for `density`.

- [ ] **Step 1: Implement** by replacing the hard-coded heights (24 / 4 / tab height) with the metrics, reading the store in each view's `body` so changes apply live.
- [ ] **Step 2: Run** the full tests and build. Expected: `** TEST SUCCEEDED **`.
- [ ] **Step 3: Verify in the app:** toggling Compact visibly tightens headers rows, sidebar and tabs; Regular restores today's layout pixel-for-pixel.
- [ ] **Step 4: Commit** `feat(macos): compact density preference`.

### Task 12: Final verification

- [ ] **Step 1:** `cd macos && xcodegen && xcodebuild test -project XRest.xcodeproj -scheme XRest`. Expected: `** TEST SUCCEEDED **`, no failures.
- [ ] **Step 2:** Launch the app and walk each Review Focus line: set a corrupt `editor.fontSize` with `defaults write dev.xrest.XRest editor.fontSize -int 999` and confirm it clamps; run each confirmation style against an unsafe environment; send a request with `{{MISSING}}`.
- [ ] **Step 3:** Confirm `git status` shows no stray generated files and report results to the user.
