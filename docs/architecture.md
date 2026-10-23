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
is false, before anything touches the network. Apps use `send_policy().requires_confirmation` to
decide whether to show a dialog, but a buggy app cannot bypass the check.

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
| 2 | this core | `pathParams` on requests |

* A workspace with a **newer** version is refused (`UnsupportedFormat`) instead of being partially
  read and then rewritten with data loss.
* An **older** workspace opens as-is; opening never rewrites tracked files. The first save of an
  endpoint, scratch request or the environments stamps the current version (and moves a legacy
  `selectedEnvironment` into `.xrest/state.yaml` so the user's choice survives).
* Readers stay backward compatible: a plain-string `body` (format 0) still loads, in endpoint
  files, scratch files and version snapshots alike.
* Bump `FORMAT_VERSION` whenever a change would make an older XRest misread or drop data.

```text
<workspace>/
├── service.yaml
├── environments.yaml
├── endpoints/<id>.yaml
├── scratchpad/<id>.yaml
├── .xrest/state.yaml
├── .xrest/versions/<endpoint-id>.json
└── .gitignore
```

### `service.yaml`

```yaml
formatVersion: 2
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
  variables:
    - { name: BASE_URL, value: "https://api.example.com", enabled: true, type: plain }
    - { name: API_KEY, value: "", enabled: true, secretKey: prod-api-key, type: secret }
```

A variable with a non-empty `secretKey` resolves to `{{secret.<secretKey>}}`, which is looked up in
the `SecretStore`. Disabled variables are not used.

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

* `state.yaml`: `selectedEnvironment`, `gitMode` (`auto|manual`, default manual, omitted when manual),
  `lastSync` (unix seconds of the last successful pull/push/sync). Not shared through git.
* `versions/<endpoint-id>.json`: endpoint version history, newest `maxVersions` (5-50, default 50).

### Differences from the legacy Go format

| Change | Why |
| --- | --- |
| `selectedEnvironment` moved from `service.yaml` to `.xrest/state.yaml`; still read from `service.yaml` as a fallback, never written | Per-user state should not churn a git-tracked file |
| `directory` and `endpoints:` stub list in `service.yaml` are read but no longer written | The workspace root is authoritative; the list duplicated `endpoints/` |
| Inline `versions:` in endpoint files are ignored on read and never written | The Go app already treated them as legacy; history lives in `.xrest/versions/` |
| Version history in `.xrest/versions/*.json` instead of global SQLite (`~/.xrest/history.db`) | SQLite is deferred; the per-workspace store follows the same `MAX+1`, FIFO-prune semantics |
| `scratchpad/` is new; "collections" are gone | Product decision |
| Variables with `enabled: false` are not applied | The Go frontend ignored the flag; honoring it is the evident intent |
| Preflight token cache is scoped by service and environment; the cache unit is honored; `Bearer ` prefix only for the `Authorization` header | The Go code could replay a DEV token against PROD and prefixed every header |
| Preflight `enabled` and `cacheToken` are honored | Ignored by the Go conversion |
| Endpoint file `lastVersion` is omitted when 0 | Cosmetic |
| `formatVersion` added to `service.yaml` (format 1) | Future format changes are detectable |
| `body` is typed (`raw`/`json`/`form`/`multipart`/`binary`); per-request `options` added (format 1) | The Go HTTP layer supported these but the files could not express them |

cURL import maps `-d` to `json` (JSON content) or `raw` with curl's default
`application/x-www-form-urlencoded`, `--data-urlencode name=value` to `form`, `-F` to `multipart`
(`@path` is a file field), and `-k`, `-m/--max-time`, `-x/--proxy` to `options`.

Preflight token expiry may be a relative lifetime or an absolute time (`epoch`, `epoch_ms`,
`iso8601`), as in the Go adapter.

## Request resolution (what `send` does)

1. Check the unsafe policy for the selected environment (`ConfirmationRequired` unless `confirmed`).
2. Build the variable map from the enabled variables of the selected environment.
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
    variable_names() -> [String]                      // enabled variables of the selected environment (autocomplete)
    preview(text) -> InterpolationPreview             // { resolved, unresolved }; secrets shown as ••••••, never their value
    preview_request_url(spec) -> InterpolationPreview // the URL as send builds it (variables, path params, enabled query params), secrets masked; no I/O
    send_policy() -> SendPolicy                       // { environment, requires_confirmation }
  endpoints
    list_endpoints() / get_endpoint(id) -> Endpoint
    create_endpoint(name, spec) / update_endpoint(id, name, spec) -> Endpoint   // update snapshots a version when the request changed
    delete_endpoint(id)
    endpoint_versions(id, limit) -> [EndpointVersion] // newest first
    restore_endpoint_version(id, version) -> Endpoint // past config becomes the current request, saved as a NEW version (MAX+1); restoring the current config is a no-op
    diff_versions(id, from, to) -> [FieldChange]      // field-level changes from -> to; NotFound for an unknown version
    set_max_versions(n)                               // retained versions per endpoint, clamped 5-50
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
`HeaderPair`, `HttpResponse`, `PreflightTestResult`.

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
* `status` is local only (no fetch): `ahead`/`behind` reflect the last fetch.
* FFI: `git_status`, `git_init`, `git_commit`, `git_pull`, `git_push`, `git_sync` are async and run
  git off the workspace lock (serialised by a separate gate); `git_mode`, `set_git_mode`, `reload`
  are synchronous.

## Deferred (stubbed with TODOs)

* `history`: request history (legacy: SQLite at `~/.xrest/history.db`).
* `swagger`: Swagger/OpenAPI import (legacy: `internal/import/swagger.go`).
* cURL `-u/--user` (deliberately dropped, as in Go, so credentials do not end up in committed files).
* App Sandbox + security-scoped bookmarks for the macOS app.
