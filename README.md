# XRest
**The Service‑First REST Client for Microservices.**

XRest organizes APIs as versioned "services" with built-in environment management, safe/unsafe execution controls, and optional auth pre‑flight—making microservice workflows repeatable, shareable, and safe.

---

## 🚀 Key Features

- **Service‑First Hierarchy**: Organize APIs as first-class Services instead of loose collections. Each service carries its own environments and authentication logic.
- **Git‑Native Collaboration**: All data (services, environments, endpoints) is stored in human-readable YAML. Sync changes across your team using git as your source of truth.
- **Guardrails for Production**: Mark environments (like `PROD`) as **Unsafe**. XRest enforces visual cues (red UI) and mandatory confirmation dialogs before executing destructive requests.
- **Pre‑flight Auth**: Stop copy-pasting tokens. Define auth endpoints that automatically acquire, cache, and inject Bearer tokens into your requests.
- **Request Versioning**: Track the evolution of your API contracts with built-in versioning for every endpoint.
- **Zero-Cloud Privacy**: XRest is local-first. Your API keys, internal URLs, and payloads never leave your machine or your private Git repository.

## 🛠 Tech Stack

- **Core**: [Wails v3](https://wails.io/) & [Go](https://go.dev/)
- **Frontend**: [Vue 3](https://vuejs.org/), [TypeScript](https://www.typescriptlang.org/)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/), [Shadcn UI Vue](https://www.shadcn-vue.com/)
- **State Management**: [Pinia](https://pinia.vuejs.org/)
- **Build Tool**: [Vite](https://vitejs.dev/)

## 📁 Storage & Configuration

XRest follows a "Configuration as Code" philosophy.

### Global Settings
Stored in the user's home directory:
- `~/.xrest/settings.yaml` (all operating systems)

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
- Go (v1.25+)
- Wails v3 CLI
- Node.js (v18+)
- pnpm
- [go-task](https://taskfile.dev/) (recommended build tool)

### Setup
Using [go-task](https://taskfile.dev/) (Recommended):
```bash
# Install dependencies
pnpm --filter frontend install

# Run in development mode
task dev

# Build production bundle
task build
```

Using Wails v3 CLI directly:
```bash
# Run in development mode
wails3 dev -config ./build/config.yml

# Build production bundle (via Go build with tags)
go build -tags production -o bin/xrest ./cmd/wails
```

## 📦 Release (macOS)
```bash
# Package the application into a .dmg installer
task package:dmg
```

---

*Note: xrest is currently in active development and is not yet ready for production use. Use at your own risk.*
