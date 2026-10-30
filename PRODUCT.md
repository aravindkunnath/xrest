# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

(Native desktop, not web. A shared Rust core with per-platform native apps; each app owns its UI in its own platform's design language. macOS first, SwiftUI, macOS 14+. Other platforms are planned, not built.)

## Users

Full-stack and API developers who test and exercise REST APIs. Microservice work is the positioning, not a restriction on who can use it. They hit multiple environments (DEV / STAGE / PROD) and want requests kept in their repo.

## Product Purpose

A REST client that organizes APIs as versioned services, with environment management, safe/unsafe execution controls and optional auth pre-flight, so microservice workflows are repeatable, shareable and safe. Status: pre-production rebuild; not ready for production use.

## Positioning

Two co-equal pillars that a neighbor (Postman, Bruno, Insomnia) could not truthfully both claim:

1. **Service-first with core-enforced production safety.** A workspace is one service. Unsafe environments are refused by the core without explicit confirmation, identically on every platform.
2. **Native and git-native.** A truly native app over plain, human-readable YAML that lives in the user's git repo.

## Operating Context

- A workspace is a service directory: `service.yaml`, `environments.yaml`, `endpoints/` (versioned requests, one YAML each), `scratchpad/` (loose requests, gitignored), `.xrest/` (per-user state, gitignored).
- Sync is via git; nothing leaves the machine or the repo.
- Auth tokens are acquired by a pre-flight request, cached per environment and injected.

## Capabilities and Constraints

- Typed request bodies (text, JSON, form, multipart, file); per-request overrides for timeout, redirects, TLS verification, proxy. Uploaded files must live inside the workspace.
- Endpoint versioning: every change to an endpoint's request is snapshotted.
- Scratchpad requests can be promoted into versioned endpoints.
- cURL import, Swagger import.
- Environments can be marked unsafe (e.g. PROD); the app shows red cues, but enforcement lives in the core, not the UI.
- Architecture: Rust core (`xrest-core`) plus UniFFI bindings (`xrest-ffi`); the macOS app is SwiftUI. See `docs/architecture.md`.
- The prior Wails/Go/Vue app lives in `legacy/` as a porting reference only.
- Undecided: timing and scope of Linux/Windows apps.

## Brand Commitments

Name: XRest. Tagline: "The Service-First REST Client for Microservices."

## Evidence on Hand

Working core, tests and fixtures (`core/xrest-core/tests/`), a legacy app for reference. No testimonials, customers, benchmarks or pricing exist; do not fabricate any.

## Product Principles

- Safety is a property of the system, not the screen: guardrails must be visible in the UI but never depend on it.
- The files are the product: plain YAML in git stays readable, diffable and shareable.
- Local-first and private: no URLs, keys or payloads leave the user's machine or repo.
- Native on every platform: the core guarantees identical behavior; each app feels at home on its OS.
- The service is the unit of work: everything is organized around one service, its environments and its versioned endpoints.
