# GitHub Status Monitor — Implementation Plan & Specification

This repository contains **GitHub Status Monitor**, a native macOS menubar and desktop dashboard application built with **Tauri**, **Rust**, **Lit (Web Components)**, **TypeScript**, **Tailwind CSS**, and **DaisyUI**.

Below is the comprehensive architectural plan, implementation detailed specification, and complete task checklist for all three development phases.

---

## 🏛️ System Architecture Overview

```
                          ┌──────────────────────────────────────────────────┐
                          │               GitHub Status API                  │
                          │  https://{region}.githubstatus.com/api/v2/       │
                          │                 summary.json                     │
                          └────────────────────────┬─────────────────────────┘
                                                   │ HTTP GET (reqwest)
                                                   ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   Rust Core Backend (Tauri)                                     │
│                                                                                                 │
│  ┌───────────────────────────┐     ┌────────────────────────────┐     ┌──────────────────────┐  │
│  │      Settings Store       │ ──► │       Shared Monitor       │ ──► │ macOS Notifications  │  │
│  │  (tauri-plugin-store)     │     │ (Tokio Polling Loop/State) │     │ (plugin-notification)│  │
│  └───────────────────────────┘     └──────────────┬─────────────┘     └──────────────────────┘  │
│                                                   │                                             │
│                                    State Update & │ Diffing                                     │
│                                                   ▼                                             │
│                                    ┌────────────────────────────┐                               │
│                                    │   macOS System Tray Menu   │                               │
│                                    │ (Pre-rendered Status Icons │                               │
│                                    │   & Native Status Items)   │                               │
│                                    └────────────────────────────┘                               │
│                                                   │                                             │
│                                                   │ Tauri IPC Events / Commands                 │
└───────────────────────────────────────────────────┼─────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               Lit / TypeScript Frontend Window                                  │
│                                                                                                 │
│  ┌─────────────────────────────────┐   ┌────────────────────────────────┐  ┌─────────────────┐  │
│  │         <app-root>              │   │     <status-banner>            │  │ <settings-view> │  │
│  │ (Theme, Routing, State Store)   │──►│ (Overall Status & Downtime)    │  │(Interval, Region│  │
│  └─────────────────────────────────┘   └────────────────────────────────┘  │ Theme Selection)│  │
│                                                        │                   └─────────────────┘  │
│                                                        ▼                                        │
│                                        ┌────────────────────────────────┐                       │
│                                        │       <component-list>         │                       │
│                                        │  (Individual Feature Grid,     │                       │
│                                        │   rendered from API data)      │                       │
│                                        └────────────────────────────────┘                       │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Technology Stack

| Domain | Technology / Library | Purpose |
| :--- | :--- | :--- |
| **Desktop Runtime** | Tauri v2 (`tauri`, `tauri-build`) | Lightweight macOS desktop runtime |
| **Backend Language** | Rust 2024 edition (`tokio`, `reqwest`, `serde`, `chrono`) | Business logic, status fetching, state management |
| **System Tray & Autostart** | Tauri Tray API, `tauri-plugin-autostart` | macOS menubar item & macOS login item toggle |
| **Notifications** | `tauri-plugin-notification` | macOS system alerts on status change |
| **Logging** | `tauri-plugin-log` (`log` crate) | Leveled logs to stdout and `~/Library/Logs/<identifier>/` |
| **Opening URLs** | `tauri-plugin-opener` (already installed) | Open status page in default browser |
| **Auto-Updates** | `tauri-plugin-updater`, `tauri-plugin-process` | Signed in-app updates from GitHub Releases |
| **Persistence** | `tauri-plugin-store` | Persistent user configuration store |
| **Frontend Framework**| Lit (`lit`) | Web Components UI library |
| **Frontend Language** | TypeScript (`typescript`) | Typed frontend scripting |
| **Styling & Theme** | Tailwind CSS v4 (`tailwindcss`, `@tailwindcss/vite`) + DaisyUI v5 (`daisyui`) | Responsive styling and light/dark theme modes |
| **Build Tooling** | Vite (`vite`), pnpm | Fast frontend bundling and dev server |
| **Frontend Testing** | Vitest (`vitest`) + `happy-dom`, `@open-wc/testing-helpers` | Unit and component testing |

---

## 📁 Project Directory Structure

```
github-monitor/
├── package.json
├── pnpm-lock.yaml
├── mise.toml                     # Pinned Node, pnpm, Rust toolchain
├── tsconfig.json
├── vite.config.ts                # Vite + @tailwindcss/vite + Vitest (`test` key)
├── index.html
├── README.md
├── src/                          # Lit Frontend Web Components
│   ├── main.ts                   # Entry point; imports styles.css globally
│   ├── styles.css                # `@import "tailwindcss"; @plugin "daisyui";`
│   ├── components/
│   │   ├── tailwind-element.ts   # LitElement base class that adopts styles.css
│   │   ├── app-root.ts           # Main layout & router container
│   │   ├── status-banner.ts      # Overall status & uptime indicator
│   │   ├── component-list.ts     # Feature status grid/list
│   │   ├── component-card.ts     # Individual feature item element
│   │   ├── settings-view.ts      # Settings form
│   │   └── elapsed-timer.ts      # Elapsed downtime calculator & display
│   ├── services/
│   │   ├── tauri-bridge.ts       # IPC wrapper calls to Rust backend
│   │   └── store.ts              # Reactive Lit state management
│   └── tests/                    # Frontend tests
│       ├── status-banner.test.ts
│       ├── settings-view.test.ts
│       └── elapsed-timer.test.ts
└── src-tauri/                    # Rust Backend Logic
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/
    │   └── default.json          # Plugin permissions for the main window
    ├── icons/                    # App & tray icons (44px, generated by scripts/gen-tray-icons.sh)
    │   ├── tray-healthy.png      # Template image (monochrome)
    │   ├── tray-degraded-{light,dark}.png
    │   ├── tray-unhealthy-{light,dark}.png
    │   ├── tray-unknown-{light,dark}.png
    │   └── dot-{green,yellow,red,gray}.png   # Menu item status dots
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── models.rs             # Status API, Settings & State structs
        ├── monitor.rs            # Shared background polling service + notifications
        ├── tray.rs               # Menubar icon, menu & autostart toggle
        └── commands.rs           # Tauri IPC commands exposed to Lit
```

---

## 📋 Phase-by-Phase Technical Design

### Phase 1: Core Menubar Monitor & Background Engine

#### 1. App Shell
* Menubar-only app: `app.set_activation_policy(ActivationPolicy::Accessory)` hides the Dock icon.
* Main window declared in `tauri.conf.json` with `visible: false`; it is only shown from the tray (Phase 2).

#### 2. System Tray & Visual Theme Adaptation
* **Icon Strategy** (pre-rendered PNGs, no runtime drawing):
  * 🟢 **Healthy**: Monochrome template image (`tray-healthy.png`, `icon_as_template(true)`); auto-matches light/dark menubar.
  * 🟡 **Degraded** / 🔴 **Unhealthy** / ⚪ **Unknown**: Colored non-template images with a status dot (`#eab308`, `#ef4444`, gray). Template mode strips color, so these ship as separate light and dark variants and are swapped on system theme change.
* **Tray Menu Specification**:
  * Item 1: `GitHub status: Healthy` (or `Degraded`, `Unhealthy`, `Unknown`), as an `IconMenuItem` with a colored dot image (green / yellow / red / gray). This is the native `NSMenuItem` image mechanism, as used by Docker Desktop's "Engine running" items.
    * *Click handler*: Opens the status page for the selected region in the default browser via `tauri-plugin-opener`.
  * Item 2: Separator.
  * Item 3: `Start when logging in` (Checkable menuItem).
    * Initial checked state read from `tauri-plugin-autostart` `is_enabled()` at startup.
    * *Click handler*: Toggles autostart via `tauri-plugin-autostart`.
  * Item 4: `Check for updates…`.
  * Item 5: Separator.
  * Item 6: `Quit` -> Terminates application.

#### 3. Rust Shared Monitor Service
* **Polling Loop**:
  * Implemented as an asynchronous `tokio::spawn` background task managed by `SharedMonitor`.
  * Default interval: 60 seconds.
  * Endpoint: `https://www.githubstatus.com/api/v2/summary.json` (region-specific host in Phase 3).
* **Status Categorization**:
  * `indicator` mapping:
    * `none` ➔ `Healthy`
    * `minor` ➔ `Degraded`
    * `major` / `critical` ➔ `Unhealthy`
  * No data yet, or fetch failed (offline, HTTP error, bad payload) ➔ `Unknown`. The last good payload is kept for display.
* **Downtime Calculation**:
  * Derived from the payload, not tracked locally: the earliest `created_at` among unresolved `incidents[]` in `summary.json`. This survives app restarts.
  * Fallback when non-healthy with no open incident: time the app first observed the non-healthy status.
* **Notification System**:
  * Compares the current status with the last *known* status (`Unknown` never triggers or resets a notification).
  * The first successful fetch sets the baseline without notifying.
  * Triggers native macOS notification when the overall status shifts (e.g. `Healthy` ➔ `Degraded`).
  * Notification text: *"GitHub Status Changed: Service is now Degraded."*
  * **Click-through**: `tauri-plugin-notification` 2.7 reports a click (`tap`) to Rust handlers via `Notification::on_action` on macOS "as far as the notification server reports it", so a click opens the status page. Still to verify on a bundled app. Notifications only display correctly from a bundled app, not `tauri dev`.

---

### Phase 2: Main Dashboard Window & Lit Web Components

#### 1. Main Dashboard Window
* **Tray Action**: Added menu item `Show the dashboard` to native Tray.
* **Window Behavior**: Hidden on start; shown and focused from the tray. Closing the window intercepts `CloseRequested`, calls `prevent_close()`, and hides the window instead of quitting.

#### 2. Dashboard Interface Requirements
* **Overall Status Section**:
  * Visual badge with current status icon & color.
  * Hyperlink button to open the status page in the default browser (`openUrl` from `@tauri-apps/plugin-opener`).
  * **Downtime Display**: When status is `Degraded` or `Unhealthy`, renders the elapsed time since the incident started (see Phase 1 Downtime Calculation), e.g. `Elapsed downtime: 1h 24m`.
* **Feature Components Grid**:
  * Renders whatever services the API returns in `components[]`; no hard-coded service list.
  * Filters out group containers (`group: true`) and the filler `Visit www.githubstatus.com for more information` entry.
  * Each card displays:
    * Service Name
    * Status Badge (`Operational`, `Degraded Performance`, `Partial Outage`, `Major Outage`, `Under Maintenance`)
    * Description / Sub-status text.

#### 3. Frontend Architecture (Lit + TypeScript + DaisyUI)
* **Styling in shadow DOM**: Global CSS does not reach shadow roots, so all components extend a shared base class following the [daisyUI Lit install guide](https://daisyui.com/docs/install/lit/):
  ```ts
  import { LitElement, unsafeCSS } from 'lit';
  import globalStyles from '../styles.css?inline';

  export class TailwindElement extends LitElement {
    static styles = [unsafeCSS(globalStyles)];
  }
  ```
  * The single `CSSResult` is shared, so every shadow root adopts the same constructed stylesheet.
  * Subclasses adding their own styles use `static styles = [...TailwindElement.styles, css\`…\`]`.
  * `main.ts` also imports `styles.css` into the document, so Tailwind v4 `@property` rules (ignored inside shadow roots) and DaisyUI theme variables register globally.
* Web Components implemented using `TailwindElement`:
  * `<app-root>`: Top-level layout with tab navigation (`Dashboard` | `Settings`).
  * `<status-banner>`: Hero section showing current status and elapsed downtime timer.
  * `<component-list>`: Grid displaying status cards for all components.
  * `<elapsed-timer>`: Custom timer component updating elapsed downtime string every minute.
* Styling configured via Tailwind CSS v4 & DaisyUI theme variables (`data-theme`).

---

### Phase 3: Persistent Settings & Themes

#### 1. Persistent Config Engine
* Configuration saved via `tauri-plugin-store` to `settings.json` in the app data folder:
  ```json
  {
    "refresh_interval_minutes": 1,
    "github_region": "global",
    "theme": "system"
  }
  ```
* Exposed via Tauri IPC commands (`get_settings`, `update_settings`).

#### 2. Settings Form (`<settings-view>`)
* **Refresh Interval**:
  * Range/Select dropdown from `1` to `60` minutes (default: `1`).
  * Updating interval updates the Rust `SharedMonitor` timer loop dynamically without needing app restart.
* **GitHub Region**:
  * Options: `Global` (default), `AU` (Australia), `EU` (Europe), `JP` (Japan), `US` (United States).
  * Maps to the status host used for both the API and the "open status page" link: `Global` ➔ `www.githubstatus.com`, others ➔ `{region}.githubstatus.com`. **Verify each regional host exists and serves `/api/v2/summary.json` before implementing.**
  * Changing region triggers an immediate refetch.
* **Theme Selection**:
  * Options: `System` (default), `Light`, `Dark`.
  * `Light` / `Dark` set DaisyUI root attribute `data-theme="light"` or `data-theme="dark"`.
  * `System` removes `data-theme`; DaisyUI follows `prefers-color-scheme` natively via `@plugin "daisyui" { themes: light --default, dark --prefersdark; }`.

---

## 📝 High-Level & Low-Level Development To-Do List

### 🚀 Phase 1: Menubar App & Backend Core

#### Project Setup
- [x] Initialize Tauri v2 project structure.
- [x] Add `mise.toml` (Node 24, pnpm 12, Rust stable).
- [x] Set `packageManager` in `package.json` to match the pnpm version.
- [x] Update all dependencies (npm and Cargo) to current versions.
- [x] Pin all direct dependencies to exact versions (`save-exact=true` in `.npmrc`; `=x.y.z` in `Cargo.toml`). Bump deliberately; lockfiles cover transitive deps.
- [x] Enable the `tray-icon` feature on `tauri` in `Cargo.toml`.
- [x] Hide the Dock icon (`ActivationPolicy::Accessory`) and set the main window to `visible: false`.
- [x] Set `app.withGlobalTauri: false` (the bridge imports `@tauri-apps/api`).
- [x] Replace `app.security.csp: null` with a real CSP.
- [ ] Add plugin permissions to the existing `src-tauri/capabilities/default.json` as each plugin is integrated (`opener:default`, `notification:default`, `autostart:default`, `updater:default`, `process:allow-restart`, `store:default`). App-defined commands are allowed by default.

#### Backend & Rust Monitor (`src-tauri/`)
- [x] Add leveled logging with `tauri-plugin-log` (Info by default, Debug for this crate in dev builds); log file is written under `~/Library/Logs/<identifier>/`.
- [x] Implement Rust data structures in `models.rs`:
  - [x] `GitHubStatusSummary`, `StatusIndicator`, `Component`, `Incident`, `Page` serde structs.
  - [x] `OverallStatus` enum: `Healthy`, `Degraded`, `Unhealthy`, `Unknown`.
  - [x] `AppStatusState` (current status, last known status, last good payload, first-observed-non-healthy timestamp).
- [x] Create HTTP client service in `monitor.rs` using `reqwest`:
  - [x] Fetch JSON from `https://www.githubstatus.com/api/v2/summary.json`.
  - [x] Map fetch failures (offline, HTTP error, bad payload) to `Unknown`, keeping the last good payload.
- [x] Implement shared background polling loop (`tokio` async task):
  - [x] Execute fetch every 60 seconds.
  - [x] Track state transitions (`Healthy` <-> `Degraded` <-> `Unhealthy`, plus `Unknown`).
  - [x] Compute incident start from the earliest unresolved `incidents[].created_at`; fall back to first observed non-healthy time.
- [x] Implement system tray (`tray.rs`):
  - [x] Create 44px tray icons from the Octicons GitHub mark (`scripts/gen-tray-icons.sh`): template `tray-healthy.png`; light/dark colored variants for degraded, unhealthy, unknown.
  - [x] Swap tray icon on status change; swap light/dark variants on system theme change.
  - [x] Status menu item as `IconMenuItem` with colored dot image; update text dynamically (`GitHub status: Healthy`, etc.).
- [x] Implement Native Tray Menu click handlers:
  - [x] Clicking status item opens `https://www.githubstatus.com` via `tauri-plugin-opener`.
- [ ] Implement autostart functionality (in `tray.rs`):
  - [ ] Integrate `tauri-plugin-autostart`.
  - [ ] Add menu item "Start when logging in" (checkbox item); initial state from `is_enabled()`.
  - [ ] Toggle startup item on/off on menu click.
- [x] Implement macOS Notifications (in `monitor.rs`):
  - [x] Integrate `tauri-plugin-notification`.
  - [x] Trigger alert when overall status changes between known states; first fetch sets baseline silently.
  - [x] Research: confirm whether notification click actions work on macOS desktop; if so, open the status page on click. (Rust `on_action` `tap` handler opens the status page; manual verification pending.)

#### Auto-Updates (`src-tauri/`)
- [ ] Integrate `tauri-plugin-updater` and `tauri-plugin-process` (for relaunch after install).
- [ ] Generate updater signing keypair (`pnpm tauri signer generate`); store private key + password as CI secrets.
- [ ] Configure `tauri.conf.json`: `bundle.createUpdaterArtifacts: true`, `plugins.updater.pubkey`, and `plugins.updater.endpoints` (GitHub Releases `latest.json`).
- [ ] Check for updates on launch and periodically (e.g. every 24h) in the background.
- [ ] Add tray menu item `Check for updates…` (between autostart and `Quit`); prompt to install and relaunch when an update is found.
- [ ] Publish signed release artifacts and `latest.json` via GitHub Actions on version tag.

---

### 🎨 Phase 2: Lit Web Components Dashboard & UI Logic

#### Frontend Setup & Tech Stack Configuration
- [ ] Add dependencies with pnpm:
  - [ ] `lit`, `tailwindcss`, `@tailwindcss/vite`, `daisyui`, `vitest`, `happy-dom`, `@open-wc/testing-helpers`.
- [ ] Add `tailwindcss()` to `vite.config.ts` plugins; set `src/styles.css` to `@import "tailwindcss";` and `@plugin "daisyui" { themes: light --default, dark --prefersdark; }`.
- [ ] Configure Vitest via the `test` key in `vite.config.ts` (`environment: 'happy-dom'`); add `test` and `test:watch` scripts.
- [ ] Create `TailwindElement` base class (`src/components/tailwind-element.ts`) adopting `styles.css?inline`; import `styles.css` globally in `main.ts`.

#### Rust Backend Extensions for UI
- [ ] Add tray menu item: `Show the dashboard` -> unhides and focuses main window.
- [ ] Hide (not close) the main window on `CloseRequested` (`api.prevent_close()`).
- [ ] Expose Tauri IPC command in `commands.rs`:
  - [ ] `get_current_status`: returns overall status, incident start time, and component array.
- [ ] Emit Tauri IPC event (`status-updated`) whenever status refreshes in Rust.

#### Frontend Web Components (`src/components/`)
- [ ] Create `tauri-bridge.ts` service to wrap IPC calls and event listeners (subscribe to `status-updated` before the initial `get_current_status` call).
- [ ] Implement `<app-root>` element:
  - [ ] Navigation header / tab system (`Dashboard` / `Settings`).
  - [ ] Event listener for background status updates from Rust backend.
- [ ] Implement `<status-banner>` element:
  - [ ] Status icon badge & text description.
  - [ ] Link button to open the status page via `openUrl` from `@tauri-apps/plugin-opener`.
- [ ] Implement `<elapsed-timer>` element:
  - [ ] Calculate and display formatted elapsed downtime (`X hours Y minutes`) from the incident start time when status is non-healthy.
- [ ] Implement `<component-list>` & `<component-card>` elements:
  - [ ] Grid layout rendering all components from the API, excluding group containers and the filler entry.
  - [ ] Color-coded status badges for each component, including `Under Maintenance`.

#### Frontend Testing (`src/tests/`)
- [ ] Write Vitest tests for `<status-banner>` rendering states (`Healthy`, `Degraded`, `Unhealthy`, `Unknown`).
- [ ] Write Vitest tests for `<elapsed-timer>` downtime calculation logic.
- [ ] Write Vitest tests for `<component-list>` data binding and filtering.

#### Release Signing & Distribution
- [ ] Sign the app with an Apple Developer ID certificate and notarize it in CI (avoids Gatekeeper blocking first installs).
- [ ] Build `universal-apple-darwin` so `latest.json` covers both `darwin-aarch64` and `darwin-x86_64`.

---

### ⚙️ Phase 3: Persistent Settings & Complete Theme System

#### Rust Backend Settings Engine
- [ ] Implement persistent configuration with `tauri-plugin-store`:
  - [ ] Define `AppSettings` struct in `models.rs`: `refresh_interval_minutes` (1–60), `github_region`, `theme`.
  - [ ] Load and save settings to `settings.json` in the app data folder.
- [ ] Expose Tauri IPC commands:
  - [ ] `get_settings`: fetch current persistent configuration.
  - [ ] `update_settings`: save updated settings and re-configure backend components.
- [ ] Dynamically update `SharedMonitor` polling interval when `refresh_interval_minutes` is updated.
- [ ] Map `github_region` to the status host for the API and the status page link; refetch immediately on change.
- [ ] Verify each regional status host exists and serves `/api/v2/summary.json`.

#### Frontend Settings View (`src/components/settings-view.ts`)
- [ ] Implement `<settings-view>` Web Component:
  - [ ] **Refresh Interval**: Select / Number input (1 to 60 minutes, default: 1).
  - [ ] **GitHub Region**: Dropdown selection (`Global`, `AU`, `EU`, `JP`, `US`).
  - [ ] **Theme Selector**: Dropdown or toggle buttons (`System`, `Light`, `Dark`).
- [ ] Implement automatic settings persistence on form change.
- [ ] Write Vitest unit tests for settings form validation and IPC trigger.

#### Theme System Implementation
- [ ] Implement theme manager service in frontend:
  - [ ] `Light`: set `document.documentElement.setAttribute('data-theme', 'light')`.
  - [ ] `Dark`: set `document.documentElement.setAttribute('data-theme', 'dark')`.
  - [ ] `System`: `document.documentElement.removeAttribute('data-theme')`; DaisyUI follows `prefers-color-scheme` via `--prefersdark`.

---

## 🏃 Getting Started & Development Commands

### Prerequisites
* **[mise](https://mise.jdx.dev)**: run `mise install` to get the pinned toolchain from `mise.toml`:
  * **Node.js**: `24`
  * **pnpm**: `12`
  * **Rust**: `stable` (edition 2024 needs `>=1.85`)
* **macOS**: macOS 11+ (Big Sur or newer)

### Development Commands

```bash
# Install dependencies
pnpm install

# Run frontend unit tests
pnpm test

# Run Vitest in watch mode
pnpm test:watch

# Start Tauri app in development mode
pnpm tauri dev

# Build production macOS binary (.app / .dmg)
pnpm tauri build
```
