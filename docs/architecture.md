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

## On-disk format

Field names are `camelCase`, matching the legacy Go app. Unknown fields are ignored on read.

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
headers: []
body: ""
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
body: '{"a": 1}'
createdAt: 1700000000
updatedAt: 1700000000
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
cacheDurationKey: expires_in   # JSON key holding the lifetime
cacheDurationUnit: seconds     # seconds | minutes | hours | days
```

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

* `state.yaml`: `selectedEnvironment`. Not shared through git.
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

## Request resolution (what `send` does)

1. Check the unsafe policy for the selected environment (`ConfirmationRequired` unless `confirmed`).
2. Build the variable map from the enabled variables of the selected environment.
3. Interpolate URL, params, headers, body, auth fields and preflight fields: `{{ NAME }}`
   (whitespace trimmed), recursive up to 10 levels, unknown names left verbatim and reported in
   `unresolved_variables`.
4. Resolve `{{secret.KEY}}` through the `SecretStore`; any missing key fails the send with
   `MissingSecrets` before a request is made.
5. Merge params into the URL query (a param replaces a same-named pair already in the URL;
   repeated params are kept). Only `http`/`https` URLs are accepted.
6. Execute: preflight token (cached) is written to its header, then service auth
   (bearer/basic/apikey header or query) is applied, then a default `Content-Type` is added if there
   is a body and none was set (`application/json` for `{`/`[` bodies, else `text/plain`).
7. Return the response. Non-2xx statuses are responses, not errors. Binary content types and
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
    service() -> ServiceInfo
    warnings() -> [String]                            // unparsable files skipped on load
  environments
    environments() -> [EnvironmentConfig]
    selected_environment() -> String?
    select_environment(name)                          // persisted in .xrest/state.yaml
    send_policy() -> SendPolicy                       // { environment, requires_confirmation }
  endpoints
    list_endpoints() / get_endpoint(id) -> Endpoint
    create_endpoint(name, spec) / update_endpoint(id, name, spec) -> Endpoint   // update snapshots a version when the request changed
    delete_endpoint(id)
    endpoint_versions(id, limit) -> [EndpointVersion] // newest first
  scratchpad
    list_scratch() / get_scratch(id) -> ScratchRequest
    create_scratch(name, spec) / update_scratch(id, name, spec) -> ScratchRequest
    delete_scratch(id)
    promote_scratch(id, name?) -> Endpoint            // removes the scratch file
    import_curl_as_scratch(command) -> ScratchRequest
  secrets / caches
    set_secret(key, value)                            // in memory; Keychain is a TODO
    clear_token_cache()
  sending
    async send(request: RequestSpec, confirmed: Bool) -> HttpResponse
```

Records: `NameValue`, `Preflight`, `RequestSpec`, `Endpoint`, `ScratchRequest`, `EndpointVersion`,
`EnvironmentConfig` (not `Environment`, which would clash with SwiftUI's), `ServiceInfo`, `SendPolicy`,
`HeaderPair`, `HttpResponse`.

Errors (`XRestError`, a Swift `Error` enum): `ConfirmationRequired(environment)`, `NotFound(message)`,
`InvalidInput(message)`, `Io(message)`, `Http(message)`, `Preflight(message)`, `MissingSecrets(keys)`,
`NotImplemented(message)`.

`send` takes the request spec rather than an id so the app can send the unsaved draft it is editing.
The object is thread-safe; `send` does not hold the internal lock while waiting on the network.

### Generating the Swift bindings

`macos/scripts/generate-bindings.sh` builds `xrest-ffi`, runs the workspace's pinned `uniffi-bindgen`
in library mode, and writes `macos/XRest/Bindings/xrest_ffi.swift` (committed). With `HEADERS_DIR` set it
also emits the C header and `module.modulemap` that `build-xcframework.sh` packs into
`XRestCore.xcframework`. CI runs it with `--check`.

## Deferred (stubbed with TODOs)

* `history`: request history (legacy: SQLite at `~/.xrest/history.db`).
* `git`: status/commit/sync (legacy: `internal/adapters/git.go`).
* `swagger`: Swagger/OpenAPI import (legacy: `internal/import/swagger.go`).
* Keychain-backed `SecretStore` (needs a foreign-implementable callback interface in `xrest-ffi`).
* cURL `-u/--user` (deliberately dropped, as in Go, so credentials do not end up in committed files).
* App Sandbox + security-scoped bookmarks for the macOS app.
