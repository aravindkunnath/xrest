# XRest
**The Service‑First REST Client for Microservices.**

XRest organizes APIs as versioned "services" with built-in environment management, safe/unsafe execution controls, and optional auth pre‑flight—making microservice workflows repeatable, shareable, and safe.

---

## 🚀 Key Features

- **Service‑First Hierarchy**: Organize APIs as first-class Services instead of loose collections. Each service carries its own environments and authentication logic.
- **Git‑Native Collaboration**: All data (services, environments, endpoints) is stored in human-readable YAML. Sync changes across your team using GitHub/GitLab as your source of truth.
- **Guardrails for Production**: Mark environments (like `PROD`) as **Unsafe**. XRest enforces visual cues (red UI) and mandatory confirmation dialogs before executing destructive requests.
- **Automated Pre‑flight Auth**: Stop copy-pasting tokens. Define auth endpoints that automatically acquire, cache, and inject Bearer tokens into your requests.
- **Request Versioning**: Track the evolution of your API contracts with built-in versioning for every endpoint.
- **Zero-Cloud Privacy**: XRest is local-first. Your API keys, internal URLs, and payloads never leave your machine or your private Git repository.

## 🛠 Tech Stack

- **Core**: [Tauri](https://tauri.app/) & [Rust](https://www.rust-lang.org/)
- **Frontend**: [Vue 3](https://vuejs.org/), [TypeScript](https://www.typescriptlang.org/)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/), [Shadcn UI Vue](https://www.shadcn-vue.com/)
- **State Management**: [Pinia](https://pinia.vuejs.org/)
- **Build Tool**: [Vite](https://vitejs.dev/)

## 📁 Storage & Configuration

XRest follows a "Configuration as Code" philosophy.

### Global Settings
Stored in the application directory:
- **macOS**: `~/Library/Application Support/io.ak.xrest/`
- **Windows**: `%APPDATA%/io.ak.xrest/`
- **Linux**: `~/.config/io.ak.xrest/`

### Service Data
Each service stores its data in a dedicated directory of your choice:
```text
your-service-directory/
├── service.yaml       # Core service configuration
├── environments.yaml  # Environment variables (DEV, STAGE, PROD)
└── endpoints/         # API request templates (*.yaml)
```

## 🛠 Development

### Prerequisites
- Node.js (v18+)
- Rust (latest stable)
- pnpm

### Setup
```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build production bundle
pnpm tauri build
```

## 📦 Release (macOS)
```bash
codesign --force --sign - --options runtime --timestamp src-tauri/target/release/bundle/dmg/xrest_*.dmg
codesign --verify --deep --strict --verbose src-tauri/target/release/bundle/dmg/xrest_*.dmg
```

---

*Note: xrest is currently in active development and is not yet ready for production use. Use at your own risk.*
