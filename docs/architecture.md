# XRest architecture

XRest follows the Ghostty model: a shared Rust core ("libxrest") that owns every behavior which must
be identical on all platforms, and thin native apps that own all UI. macOS (SwiftUI) comes first; a
GTK app is planned, so nothing in the core may assume a platform.

```text
┌────────────────────────────────────┐
│ macOS app (SwiftUI)  /  GTK (later)│  windows, sidebar, editors, dialogs, menus, key shortcuts
├────────────────────────────────────┤
│ xrest-ffi (UniFFI)                 │  coarse object + records, async send, error enum
├────────────────────────────────────┤
│ xrest-core                         │  workspace, YAML format, variables, environments, unsafe
│                                    │  policy, auth, preflight, HTTP, versioning, cURL import
└────────────────────────────────────┘
```

## Product model

* **One service per workspace.** A workspace is one service directory.
* A workspace contains **endpoints** (versioned, committed to git) and a **scratchpad** (loose,
  unversioned, gitignored). This replaces the legacy "collections" concept.
* A scratch request can be **promoted** to an endpoint: it becomes `endpoints/<id>.yaml`, gets
  version 1, and the scratch file is removed.
* Scratch requests use the same environments as endpoints, including the unsafe guardrail.

## The core / app boundary

| Owned by the core | Owned by the app |
| --- | --- |
| Loading/saving the workspace (YAML) | Windows, navigation, sidebar, editors, response rendering |
| `{{VAR}}` interpolation, `{{secret.KEY}}` lookup | Choosing which request is open; the in-memory draft |
| Environment selection (persisted) | Environment picker, red "unsafe" styling |
| Unsafe policy: refuses to send unless `confirmed` | The confirmation dialog (asks the user, passes `confirmed`) |
| Service-level auth injection, preflight, token cache | Native menu commands and key bindings |
| Request execution (reqwest + rustls), response decoding, JSON pretty-printing | Presenting errors |
| Endpoint versioning, scratch promotion, cURL import | OS credential store integration (planned, via a core trait) |

The unsafe guardrail is enforced in the core: `Workspace::prepare_send` returns
`Error::ConfirmationRequired { environment }` when the selected environment is unsafe and `confirmed`
is false, before anything touches the network. Apps use `send_policy_for(method)` to decide whether
to show a dialog (and how long its countdown must be), but a buggy app cannot bypass the check.

## Core crate layout

```text
xrest-core/src/
├── model/        what things are (serde types = the on-disk format)
│   ├── request.rs    RequestSpec, Body, RequestOptions, Preflight, NameValue
│   ├── service.rs    ServiceFile, auth, Environment, WorkspaceState
│   ├── endpoint.rs   Endpoint, ScratchRequest, version snapshots
│   └── response.rs   HttpResponse
├── workspace/    the Workspace facade, one file per concern
│   ├── mod.rs        struct, open/create, settings, environments, format upgrade
│   ├── endpoints.rs  versioned endpoints
│   ├── scratch.rs    scratchpad, promotion, cURL import
│   └── send.rs       hands workspace state to `resolve`
├── variables.rs  variable layers (global, workspace, environment, local, built-ins), report
├── resolve.rs    RequestSpec → PreparedSend: variables, secrets, auth, preflight, body files
├── exec.rs       PreparedSend → HTTP (reqwest), response decoding
├── storage.rs    file layout, atomic writes, loading, format check
└── interpolate.rs, policy.rs, preflight.rs, versions.rs, curl.rs, secrets.rs
```

`resolve` is pure with respect to the workspace (it receives a `SendContext`), so request semantics
can be tested and changed without touching persistence.

## On-disk format

Field names are `camelCase`, matching the legacy Go app. Unknown fields are ignored on read.

### Format version

`service.yaml` carries `formatVersion` for the whole workspace (`FORMAT_VERSION` in the core):

| Version | Written by | Notes |
| --- | --- | --- |
| 0 (key absent) | legacy Go app | bodies are plain strings |
| 1 | earlier core | typed `body`, per-request `options` |
| 2 | earlier core | `pathParams` on requests |
| 3 | this core | `variables.yaml`; environment `color`, `confirm`, `minCountdownSeconds`; `appearance` in `service.yaml` |

* A workspace with a **newer** version is refused (`UnsupportedFormat`) instead of being partially
  read and then rewritten with data loss.
* An **older** workspace opens as-is; opening never rewrites tracked files. The first save of an
  endpoint, scratch request, the environments, the workspace variables or the appearance stamps
  the current version (local overrides are per user and do not). It also moves a legacy
  `selectedEnvironment` into `.xrest/state.yaml` so the user's choice survives).
* Readers stay backward compatible: a plain-string `body` (format 0) still loads, in endpoint
  files, scratch files and version snapshots alike.
* Bump `FORMAT_VERSION` whenever a change would make an older XRest misread or drop data.

```text
<workspace>/
├── service.yaml
├── environments.yaml
├── variables.yaml                     # shared by every environment (optional)
├── endpoints/<id>.yaml
├── scratchpad/<id>.yaml
├── .xrest/state.yaml
├── .xrest/overrides.yaml              # this user's overrides, per environment
├── .xrest/versions/<endpoint-id>.json
└── .gitignore
```

### `service.yaml`

```yaml
formatVersion: 3
id: s-<uuid>
name: Petstore
isAuthenticated: true          # master switch for service-level auth
authType: bearer               # none | basic | bearer | apikey
auth:                          # values may use {{VAR}} / {{secret.KEY}}
  type: bearer
  active: true                 # written for compatibility; ignored
  bearerToken: "{{secret.petstore-token}}"
  basicUser: ""
  basicPass: ""
  apiKeyName: ""
  apiKeyValue: ""
  apiKeyLocation: ""           # header (default) | query
preflight: { ... }             # optional service-level preflight (see below)
gitUrl: ...                    # optional
appearance: { color: blue, icon: server }   # optional; names from the palette / icon list
```

Service auth is injected into a request only when `isAuthenticated` is true **and** the request has
`authenticated: true`.

### `environments.yaml`

```yaml
- name: DEV
  isUnsafe: false
  variables:
    - { name: BASE_URL, value: "http://localhost:8080", enabled: true, type: plain }
- name: PROD
  isUnsafe: true
  color: red                   # optional: red orange amber green teal blue purple pink graphite
  confirm: changes             # every (default, omitted) | changes: GET, HEAD, OPTIONS send unasked
  minCountdownSeconds: 5       # team minimum for the confirmation countdown, 0-30 (default 0)
  variables:
    - { name: BASE_URL, value: "https://api.example.com", enabled: true, type: plain }
    - { name: API_KEY, value: "", enabled: true, secretKey: prod-api-key, type: secret }
```

A variable with a non-empty `secretKey` resolves to `{{secret.<secretKey>}}`, which is looked up in
the `SecretStore`. Disabled variables are not used. New workspaces color DEV green, STAGE amber and
PROD red.

`confirm` and `minCountdownSeconds` only matter when `isUnsafe`. The scope is judged by the request's
own method (for Test Preflight, the preflight's method). Apps run the countdown for the longer of
the user's own setting and `minCountdownSeconds`, whatever confirmation style the user picked.

### Variable layers

A `{{NAME}}` takes its value from the most specific layer that defines it (enabled, non-empty name):

| Layer | Stored in | Shared |
| --- | --- | --- |
| Local override | `.xrest/overrides.yaml`, keyed by environment name | no (gitignored) |
| Environment | `environments.yaml` | yes |
| Workspace | `variables.yaml` (same row shape as environment variables) | yes |
| Global | the app's file (macOS: `Application Support/XRest/variables.yaml`) | no |
| Built-in | computed per send | — |

```yaml
# variables.yaml
- { name: API_VERSION, value: "2024-06-01", enabled: true, type: plain }
# .xrest/overrides.yaml
DEV:
  - { name: USER_ID, value: "7", enabled: true, type: plain }
```

* A disabled row lets the next layer down show through.
* Secret-backed rows work in every layer; secret keys referenced anywhere count as in use.
* Built-ins: `$uuid`, `$timestamp` (Unix seconds), `$isoTimestamp` (UTC, `YYYY-MM-DDTHH:MM:SSZ`),
  `$randomInt` (0-1000). Each gets one value per send. A user variable of the same name wins.
  Previews leave built-ins as written and never list them as unresolved; code export fills them.
* Overrides are keyed by environment name: renaming an environment leaves its overrides behind.
* `variable_report(environment)` returns, per name, every layer's row and the winning layer; the
  macOS Variables panel is built on it.

### `endpoints/<id>.yaml`

```yaml
id: e-<uuid>
serviceId: s-<uuid>
name: List pets
method: GET
url: "{{BASE_URL}}/pets"
authenticated: true
authType: bearer
metadata: { version: "1.0", lastUpdated: 1700000001 }
params:  [ { name: limit, value: "10", enabled: true, type: plain } ]
pathParams: []                 # omitted when empty; values for {name} tokens in the URL path
headers: []
body: { type: json, content: '{"name": "Rex"}' }   # omitted when there is no body
options: { timeoutMs: 5000 }   # omitted when empty; see "Request bodies and options"
preflight: { ... }             # optional, overrides the service preflight
lastVersion: 3                 # save counter; the history is not stored here
```

### `scratchpad/<id>.yaml`

Same request fields as an endpoint, without `serviceId`/`authType`/`metadata`/`lastVersion`:

```yaml
id: r-<uuid>
name: try the thing
method: POST
url: "{{BASE_URL}}/things"
authenticated: false
params: []
headers: []
body: { type: raw, content: 'hello', contentType: text/plain }
createdAt: 1700000000
updatedAt: 1700000000
```

### Request bodies and options

`body` is a map tagged by `type`. Variables are interpolated in every text value and path.

```yaml
body: { type: raw, content: '<a/>', contentType: application/xml }  # no contentType: sniffed (JSON-looking → JSON, else text/plain)
body: { type: json, content: '{"a": 1}' }                          # always application/json
body: { type: form, fields: [ { name: q, value: rust } ] }         # application/x-www-form-urlencoded
body:
  type: multipart                                                   # multipart/form-data
  fields:
    - { name: title, value: hello }
    - { name: upload, value: fixtures/cat.png, type: file }
body: { type: binary, path: fixtures/blob.bin, contentType: image/png }  # default application/octet-stream
```

A `Content-Type` header set on the request wins over the body's, except for multipart, where the
header is dropped because the boundary must come from the body.

**Files must be inside the workspace.** Paths are relative to the workspace root (absolute paths
are accepted if they point inside it). After following symlinks, a file outside the workspace is
refused with `InvalidBody`: endpoints are shared through git, and a shared request must not be able
to upload arbitrary files (for example `~/.ssh/id_rsa`) from a teammate's machine. Files are read
when the request is prepared, up to 64 MiB each.

`options` overrides the app's transport defaults for one request; every key is optional:

```yaml
options:
  timeoutMs: 5000          # 0 or absent: default (30 s)
  followRedirects: false   # default true
  verifyTls: false         # default true; false accepts invalid certificates
  proxy: "http://proxy:8080"   # absent: system proxy; "": direct connection; variables allowed
```

### Preflight

```yaml
enabled: true
method: POST
url: "{{BASE_URL}}/oauth/token"
bodyType: raw                  # raw | urlencoded (uses bodyParams)
body: '{"client":"abc"}'
bodyParams: []
headers: []
tokenKey: data.token           # dot path in the JSON response (default access_token)
tokenHeader: Authorization     # header to write (default Authorization); "Bearer " is added only for Authorization
cacheToken: true
cacheDuration: derived         # "derived" or a number of cacheDurationUnit
cacheDurationKey: expires_in   # JSON key holding the lifetime or expiry
cacheDurationUnit: seconds     # seconds | minutes | hours | days | epoch | epoch_ms | iso8601
```

With `cacheDuration: derived`, the units `seconds`..`days` read a relative lifetime, while `epoch`
(seconds since 1970), `epoch_ms` and `iso8601` (RFC 3339) read an absolute expiry time. Values may
be JSON numbers or numeric strings. A past or unparsable expiry falls back to the default TTL.

Tokens are cached in memory, keyed by service, environment, method and URL (default TTL 5 minutes
when no lifetime is available).

### `.gitignore`

XRest creates (or appends to) `<workspace>/.gitignore` with:

```text
/scratchpad/
/.xrest/
```

when a workspace is created and whenever the scratchpad or `.xrest/` is first written. Existing
content is preserved.

### Per-user state (`.xrest/`)

* `overrides.yaml`: this user's variable overrides per environment (see Variable layers).
* `state.yaml`: `selectedEnvironment`, `gitMode` (`auto|manual`, default manual, omitted when manual),
  `lastSync` (unix seconds of the last successful pull/push/sync). Not shared through git.
* `versions/<endpoint-id>.json`: endpoint version history, newest `maxVersions` (5-50, default 50).

### Differences from the legacy Go format

| Change | Why |
| --- | --- |
| `selectedEnvironment` moved from `service.yaml` to `.xrest/state.yaml`; still read from `service.yaml` as a fallback, never written | Per-user state should not churn a git-tracked file |
| `directory` and `endpoints:` stub list in `service.yaml` are read but no longer written | The workspace root is authoritative; the list duplicated `endpoints/` |
| Inline `versions:` in endpoint files are ignored on read and never written | The Go app already treated them as legacy; history lives in `.xrest/versions/` |
| Version history in `.xrest/versions/*.json` instead of global SQLite (`~/.xrest/history.db`) | Global SQLite is now the execution history (see History); the per-workspace store follows the same `MAX+1`, FIFO-prune semantics |
| `scratchpad/` is new; "collections" are gone | Product decision |
| Variables with `enabled: false` are not applied | The Go frontend ignored the flag; honoring it is the evident intent |
| Preflight token cache is scoped by service and environment; the cache unit is honored; `Bearer ` prefix only for the `Authorization` header | The Go code could replay a DEV token against PROD and prefixed every header |
| Preflight `enabled` and `cacheToken` are honored | Ignored by the Go conversion |
| Endpoint file `lastVersion` is omitted when 0 | Cosmetic |
| `formatVersion` added to `service.yaml` (format 1) | Future format changes are detectable |
| `body` is typed (`raw`/`json`/`form`/`multipart`/`binary`); per-request `options` added (format 1) | The Go HTTP layer supported these but the files could not express them |

cURL import maps `-d` to `json` (JSON content) or `raw` with curl's default
`application/x-www-form-urlencoded`, `--data-urlencode name=value` to `form`, `-F` to `multipart`
(`@path` is a file field), and `-k`, `-m/--max-time`, `-x/--proxy` to `options`. A command containing
`{{` is refused with `InvalidInput` (a pasted command may come from untrusted docs and must not carry
`{{secret.KEY}}` into a workspace; type placeholders in after import).

Methods are validated everywhere (`normalize_method`: trim, uppercase, empty = `GET`, otherwise only
letters A-Z): send, export, saving endpoints and scratch requests, cURL `-X`. Anything else is
`InvalidInput`, so a method can never inject into an export. The curl export also shell-quotes any
method that is not plain letters.

Preflight token expiry may be a relative lifetime or an absolute time (`epoch`, `epoch_ms`,
`iso8601`), as in the Go adapter.

## Request resolution (what `send` does)

1. Check the unsafe policy for the selected environment and the request's method
   (`ConfirmationRequired` unless `confirmed`).
2. Build the variable map from every layer (local override › environment › workspace › global ›
   built-ins).
3. Interpolate URL, params, headers, body, auth fields and preflight fields: `{{ NAME }}`
   (whitespace trimmed), recursive up to 10 levels, unknown names left verbatim and reported in
   `unresolved_variables`.
4. Resolve `{{secret.KEY}}` through the `SecretStore`; any missing key fails the send with
   `MissingSecrets` before a request is made.
5. Replace `{name}` tokens in the URL **path** (not host, query, fragment or `{{...}}`) with the
   enabled `pathParams` entry of that name: its value is interpolated (variables and secrets) and
   percent-encoded as one path segment (`/` and space are escaped). A token with no enabled
   param stays verbatim and is reported in `unresolved_variables` as `{name}`.
6. Merge params into the URL query (a param replaces a same-named pair already in the URL;
   repeated params are kept). Only `http`/`https` URLs are accepted.
7. Execute: preflight token (cached) is written to its header, then service auth
   (bearer/basic/apikey header or query) is applied, then a default `Content-Type` is added from
   the body type if none was set (see "Request bodies and options"). Files referenced by the body
   are read during preparation, inside the workspace only. The request's `options` are merged over
   the app's `HttpOptions`.
8. Return the response. Non-2xx statuses are responses, not errors. Binary content types and
   non-UTF-8 bodies are base64 (`body_is_base64`); JSON bodies also get `pretty_body`.

## FFI API (`xrest-ffi`)

One object, `XRestWorkspace`, plus plain records. Records mirror the core types so the core has no
FFI dependency. Swift names are the camelCase equivalents; Swift sees `send` as `async throws`.

```text
XRestWorkspace
  constructors
    open_workspace(path) -> XRestWorkspace            // Swift: XRestWorkspace.openWorkspace(path:)
    create_workspace(path, name) -> XRestWorkspace
  workspace
    service() -> ServiceInfo                          // incl. auth fields and service preflight
    update_service_settings(settings: ServiceSettings) // writes service.yaml (keeps id/gitUrl), clears token cache
    warnings() -> [String]                            // unparsable files skipped on load
  environments
    environments() -> [EnvironmentConfig]
    selected_environment() -> String?
    select_environment(name)                          // persisted in .xrest/state.yaml
    save_environments(envs: [EnvironmentConfig])      // trimmed, non-empty, unique names else InvalidInput; selection kept or falls back to first
    variable_names() -> [String]                      // every layer's enabled names for the selected environment, then built-ins (autocomplete)
    preview(text) -> InterpolationPreview             // { resolved, unresolved }; secrets shown as ••••••, never their value
    preview_request_url(spec) -> InterpolationPreview // the URL as send builds it (variables, path params, enabled query params), secrets masked; no I/O
    export_code(spec, language: CodeLanguage, include_secrets) -> String // curl | axios | java | kotlin; no I/O, no unsafe-env confirmation
    send_policy() -> SendPolicy                       // is the selected environment unsafe at all (any method)
    send_policy_for(method) -> SendPolicy             // { environment, requires_confirmation, min_countdown_seconds } for one send
  variable layers
    set_global_variables(XRestGlobals)                // attach the app's global list (shared, read live)
    workspace_variables() / save_workspace_variables(vars)   // variables.yaml; names trimmed, unnamed rows dropped
    local_overrides(env) / save_local_overrides(env, vars)   // .xrest/overrides.yaml; empty list removes; UnknownEnvironment
    variable_report(env?) -> [VariableReport]         // { name, values: [LayerValue], winner: VariableLayer? }
  appearance
    appearance() / set_appearance(WorkspaceAppearance)       // { color?, icon? }; unknown names are InvalidInput
  endpoints
    list_endpoints() / get_endpoint(id) -> Endpoint
    create_endpoint(name, spec) / update_endpoint(id, name, spec) -> Endpoint   // update snapshots a version when the request changed
    delete_endpoint(id)
    endpoint_versions(id, limit) -> [EndpointVersion] // newest first
    restore_endpoint_version(id, version) -> Endpoint // past config becomes the current request, saved as a NEW version (MAX+1); restoring the current config is a no-op
    diff_versions(id, from, to) -> [FieldChange]      // field-level changes from -> to; NotFound for an unknown version
    set_max_versions(n)                               // retained versions per endpoint, clamped 5-50
    set_http_options(HttpOptions)                     // app-wide transport defaults (Settings); per-request options still override
  scratchpad
    list_scratch() / get_scratch(id) -> ScratchRequest
    create_scratch(name, spec) / update_scratch(id, name, spec) -> ScratchRequest
    delete_scratch(id)
    promote_scratch(id, name?) -> Endpoint            // removes the scratch file
    import_curl_as_scratch(command) -> ScratchRequest
  secrets / caches
    set_secret_provider(provider: SecretProvider)     // app-implemented (Keychain); consulted first
    set_secret(key, value)                            // in-memory fallback (tests)
    secret_keys_in_use() -> [String]                  // sorted, distinct keys referenced by the workspace
    clear_token_cache()
  sending
    async send(request: RequestSpec, confirmed: Bool) -> HttpResponse
    async test_preflight(preflight: Preflight?, confirmed: Bool) -> PreflightTestResult
```

`RequestSpec.body` is the `RequestBody` enum (`empty`, `raw`, `json`, `form`, `multipart`,
`binary`; `empty` rather than `none` to avoid clashing with Swift's `Optional.none`) and
`RequestSpec.options` is `RequestOptions`. `ServiceInfo.format_version` reports the on-disk format.

Records: `NameValue`, `Preflight`, `RequestSpec`, `RequestOptions`, `Endpoint`, `ScratchRequest`, `EndpointVersion`, `FieldChange`
(`field` such as `url`, `param:<name>`, `header:<name>`, `body.type`, `body.content`, `options.<key>`, `preflight.<field>`;
`change` is `added`/`removed`/`changed`; `before`/`after` optional text),
`EnvironmentConfig` (not `Environment`, which would clash with SwiftUI's), `ServiceInfo`, `ServiceSettings`,
`InterpolationPreview`, `SendPolicy`,
`HeaderPair`, `HttpResponse`, `PreflightTestResult`, `WorkspaceAppearance`, `VariableReport`,
`LayerValue`, `BuiltinVariable`; enums `ConfirmScope` (`every`, `changes`) and `VariableLayer`
(`local`, `environment`, `workspace`, `global`, `builtIn`).

`XRestGlobals` holds the user's global variables: `XRestGlobals.open(path:)` loads a YAML list (a
missing file is empty), `variables()`, `save(variables:)` writes it. One object is shared by every
workspace through `set_global_variables`, so a save reaches every window's next send. Free
functions: `builtin_variables()`, `color_palette()`, `workspace_icons()`.

Errors (`XRestError`, a Swift `Error` enum): `ConfirmationRequired(environment)`,
`UnsupportedFormat(found, supported)`, `NotFound(message)`,
`InvalidInput(message)`, `Io(message)`, `Http(message)`, `Preflight(message)`, `MissingSecrets(keys)`,
`NotImplemented(message)`.

`SecretProvider` is a foreign-implementable protocol (`get(key) -> String?`, `contains(key) -> Bool`); the app implements it over
the Keychain and the FFI adapter (a `SecretStore`) asks it first, then the `set_secret` memory store.
Missing keys still fail the send with `MissingSecrets(keys)` before any request. `secret_keys_in_use`
scans the service, environments, endpoints and scratch requests for `{{secret.KEY}}` and `secretKey`.

`preview` resolves `{{VAR}}` in the selected environment and looks `{{secret.KEY}}` up through the same
chain as `send` (provider, then `set_secret`) only to decide whether a value exists, through
`contains` (`SecretStore::contains`, default `get(..).is_some()`), never `get`, so typing never
reads secret values from the credential store: found secrets
become `••••••`, missing ones stay as `{{secret.KEY}}` and are listed as `secret.KEY` in `unresolved`
(after the plain names). `ServiceSettings.auth_type` is `none|basic|bearer|apikey`; unknown values and
an empty name are `InvalidInput`.

`preview_request_url(spec)` is the same preview applied to a whole request: `{{VAR}}` interpolated, enabled
`{name}` path params substituted (same encoding as `send`), enabled query params merged exactly as `send`
does, secrets shown as `••••••`. Unresolved lists variables, `{name}` path params, then `secret.KEY`. If the
result is not yet a valid URL the partly resolved text is returned. A path param whose value is `.` or `..`
is percent-encoded (`%2E`), but `send` refuses it with `InvalidInput` because the URL layer collapses
the dot-segment even when encoded (the preview shows the collapsed URL). A preflight token lifetime is clamped to 30 days.

`export_code(spec, language, include_secrets)` renders the request resolved for the selected environment
(`CodeLanguage`: `Curl`, `Axios`, `Java` = `java.net.http`, `Kotlin` = OkHttp). Resolution does no I/O: no
preflight request (the preflight header is `Bearer <preflight token>`, or `<preflight token>` for a custom
header), no file reads (multipart/binary file fields keep their path: `-F name=@path` in curl, file reads in
the others), and no unsafe-environment policy. Service auth is applied as a header (basic as
`Basic <base64>`) or query key. By default every secret stays `{{secret.KEY}}` (secrets need not exist; basic
auth with a secret shows `Basic base64(user:{{secret.PW}})`); `include_secrets` looks real values up and a
missing one is `MissingSecrets`. Options: curl `-k -L -m <secs> -x <proxy>`; the others use the closest
equivalent and a comment where there is none (TLS verification in Java/Kotlin). An unparseable URL is
`InvalidUrl`. The curl output round-trips through `import_curl`, whose tokenizer removes shell quotes
(`'it'\''s'` is `it's`).

`send` takes the request spec rather than an id so the app can send the unsaved draft it is editing.
`test_preflight` runs one preflight request (the given one, or the service's when `nil`) in the
selected environment, with the same unsafe-environment confirmation as `send`. It never reads or
writes the token cache. Transport failures, non-2xx statuses and token-extraction failures come back
as a result with `error` set (and `status`/bodies filled in when a response arrived), not as a
thrown error; only policy, missing-secret, invalid-URL and "no preflight configured" errors throw.
The object is thread-safe; `send` and `test_preflight` do not hold the internal lock while waiting on the network.

### Generating the Swift bindings

`macos/scripts/generate-bindings.sh` builds `xrest-ffi`, runs the workspace's pinned `uniffi-bindgen`
in library mode, and writes `macos/XRest/Bindings/xrest_ffi.swift` (committed). With `HEADERS_DIR` set it
also emits the C header and `module.modulemap` that `build-xcframework.sh` packs into
`XRestCore.xcframework`. CI runs it with `--check`.

## Git

`xrest_core::git` shells out to the system `git` (`git -C <root>`, no shell, `GIT_TERMINAL_PROMPT=0`,
`LC_ALL=C`), so credentials and identity come from the user's own git/SSH setup. Failures are
`Error::Git(stderr)` (FFI `XRestError.Git`); a missing binary is `Error::Git("git not found")`.

* Staging only ever names `service.yaml`, `environments.yaml`, `endpoints/`, `.gitignore`;
  `scratchpad/` and `.xrest/` are never staged. Commits use explicit pathspecs, so other files the
  user staged are left alone.
* `gitMode: auto` commits after every successful save of an endpoint (`Create|Update|Delete endpoint:
  <name>`), the environments (`Update environments`) or service settings (`Update service settings`),
  touching only the saved file. A failed auto-commit is a workspace warning, never a failed save.
* Pull is a merge (`--no-rebase`); on conflict it runs `git merge --abort` and the error lists the
  conflicting paths. After a successful pull/push/sync the workspace reloads from disk.
* `status` runs with `GIT_OPTIONAL_LOCKS=0`. Commands that change the repo (commit, pull, push,
  init) take a per-root gate in core; auto-commit on save uses `try_lock` and, if a manual action holds
  the gate, leaves the change uncommitted with a workspace warning instead of racing or blocking the
  save. A remote URL starting with `-` is `InvalidInput` (and `git remote add/set-url` gets `--`).
* `status` is local only (no fetch): `ahead`/`behind` reflect the last fetch.
* FFI: `git_status`, `git_init`, `git_commit`, `git_pull`, `git_push`, `git_sync` are async and run
  git off the workspace lock (serialised by a separate gate); `git_mode`, `set_git_mode`, `reload`
  are synchronous.

## OpenAPI / Swagger import

`xrest_core::swagger::parse` reads OpenAPI 3.x and Swagger 2.0 (JSON or YAML, walking
`serde_json::Value`, local `$ref`s only) into `ImportedSpec { title, base_url, endpoints }`.

* Base URL: Swagger 2 `schemes[0]://host basePath` (scheme defaults to https); OpenAPI 3
  `servers[0].url` with server variables replaced by their defaults. Trailing `/` is trimmed.
* One endpoint per path x method (GET, PUT, POST, DELETE, OPTIONS, HEAD, PATCH). Name: `summary`,
  else `operationId`, else `METHOD path`. URL `{{BASE_URL}}<path>` with `{param}` tokens kept; path,
  query and header parameters (path-item level and `$ref`ed ones included; an operation parameter
  overrides a path-item one) become `pathParams` / `params` / `headers` rows, value = default,
  else example, else empty.
* Body: first JSON media type's example (media `example`, first of `examples`, or the schema's
  `example`) becomes a `json` body; no example means no body. `application/x-www-form-urlencoded`
  (and Swagger 2 `formData`) become a `form` body from the schema properties.
* `Workspace::create_from_openapi` (FFI `XRestWorkspace.createWorkspaceFromOpenapi`): validates
  first, then creates DEV/STAGE/PROD (PROD unsafe) with `BASE_URL`, and saves each endpoint at
  version 1. No server in the spec: `https://api.example.com` plus a workspace warning.
* `Workspace::merge_openapi` (FFI `mergeOpenapi`) returns `ImportSummary { added, skipped }`; it
  matches by method + path (URL after `{{BASE_URL}}`, query ignored) and never modifies existing
  endpoints. In auto git mode the additions are one commit, `Import OpenAPI: N endpoints`.
* Sanitising: any imported string containing `{{` (base URL, title, path, names, parameter names and
  defaults/examples, JSON and form bodies) is dropped (empty; a base URL falls back to the
  `https://api.example.com` placeholder) and a workspace warning names the field, so a hostile spec
  cannot smuggle `{{secret.KEY}}` or `{{VAR}}` into the workspace. `ImportedSpec.warnings` carries them.
* FFI `fetchText(url)` (async, free function): http/https only, 10 MiB cap enforced while
  streaming, 30 s timeout, UTF-8 or an error. Invalid content is `InvalidInput`.

## History

Execution history is one SQLite database (`rusqlite`, `bundled`) at a path the app passes to
`XRestHistory.open(path:)` (macOS: `~/Library/Application Support/XRest/history.sqlite`); never inside
a workspace. One `XRestHistory` object is shared by all workspaces (`XRestWorkspace.setHistory`);
WAL mode, one connection behind a mutex. It does not read the legacy `~/.xrest/history.db`.
Schema versioning: `PRAGMA user_version` is 1 (set on creation; `open` runs a migration hook, a no-op
for 1) and a database with a newer version is refused with a clear error instead of being touched.
`PRAGMA secure_delete=ON` on the connection; `clear()` also runs `wal_checkpoint(TRUNCATE)` and
`VACUUM`, so deleted bodies do not linger in the db or `-wal` file.

Table `history` (`id` autoincrement, newest = highest): `created_at` (unix ms), `service_id`,
`service_name`, `endpoint_id` (null for scratch/unsaved sends), `environment`, `method`, `url` (final),
`request_headers` / `response_headers` (JSON arrays of `[name, value]`), `request_body`, `status`
(null = no response), `status_text`, `response_body`, `body_is_base64`, `truncated`, `time_ms`,
`size_bytes`, `error`. Indexes on `(service_id, id)` and `(endpoint_id, id)`.

* Recorded: every send that reached the network layer (2xx, non-2xx, transport/preflight error).
  Not recorded: refusals before sending (`ConfirmationRequired`, `MissingSecrets`, invalid URL).
  `send` records with no endpoint; `sendForEndpoint(endpointId:request:confirmed:)` records against one.
  Test Preflight is not recorded.
* Masking, before storing: values of `Authorization`, `Proxy-Authorization`, `Cookie`, `Set-Cookie`
  become `••••••`; every resolved secret value (`{{secret.KEY}}`, `secretKey` variables, secrets used
  by preflight, and the preflight token itself) is replaced by `••••••` in the URL, header values and
  text bodies, `status_text` and the stored error text, longest first, also in its form-encoded,
  percent-encoded and `url`-crate spellings (path, path segment, query, fragment, userinfo). Secrets
  shorter than 4 characters are not masked. Base64 (binary) response bodies are not scanned.
  Binary request bodies are stored as `<binary body, N bytes>`; multipart as `name: value` lines.
* Bodies are capped at 1 MiB per entry (text cut on a UTF-8 boundary; base64 cut on the decoded
  bytes); `truncated` is set. `sizeBytes` is the real size.
* A recording failure becomes a workspace warning and never fails the send.
* FFI: `list(filter:limit:offset:)` returns `HistorySummary` (no headers/bodies) newest first,
  `get(id:)` the full `HistoryEntry`, `count(filter:)`, `clear()`. `HistoryFilter` has `serviceId`,
  `endpointId`, `method`, `statusClass` (2-5, or 0 for failed requests). A 50-row first page over
  10,000 entries takes well under 1 ms (index scan).

## Preferences and session restore (macOS app)

* Settings (⌘,) are `UserDefaults` keys (`AppPreferences`): appearance (System/Light/Dark, applied with
  `.preferredColorScheme`; `Theme` tokens are dark/light pairs), zoom (scales monospaced/code text and
  the editors), default HTTP options (`HttpOptions` via `set_http_options`) and max versions. They are
  pushed into every open workspace on open and on change. Never stored in a workspace. Zoom is
  published through the observable `ZoomStore` (read by `Font.mono` and `CodeEditor`), so views re-render
  in place; the view tree is never rebuilt, and open panels keep their unsaved edits.
* Session files: `~/Library/Application Support/XRest/sessions/<sha256(path)>.json` holds the workspace
  path, selected item and the unsaved draft. Written (debounced) on change and on
  `willTerminate`; deleted on a clean window close with no unsaved draft. At launch the first window
  takes the first stored workspace and the rest get their own windows. A draft is stored exactly as
  typed: a plain secret typed into an unsaved draft is persisted locally (use `{{secret.KEY}}`).
  If the item was deleted on disk, the draft is kept as a new scratch request with a notice.

## Deferred (stubbed with TODOs)

* cURL `-u/--user` (deliberately dropped, as in Go, so credentials do not end up in committed files).
* App Sandbox + security-scoped bookmarks for the macOS app.
