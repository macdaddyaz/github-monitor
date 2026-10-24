# GitHub Status Monitor — Implementation Plan & Specification

This repository contains **GitHub Status Monitor**, a native macOS menubar and desktop dashboard application built with **Tauri**, **Rust**, **Lit (Web Components)**, **TypeScript**, **Tailwind CSS**, and **DaisyUI**.

Below is the comprehensive architectural plan, implementation detailed specification, and complete task checklist for all three development phases.

---

## 🏛️ System Architecture Overview

```
                          ┌──────────────────────────────────────────────────┐
                          │               GitHub Status API                  │
                          │   https://www.githubstatus.com/api/v2/summary.json│
                          └────────────────────────┬─────────────────────────┘
                                                   │ HTTP GET (reqwest)
                                                   ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   Rust Core Backend (Tauri)                                     │
│                                                                                                 │
│  ┌───────────────────────────┐     ┌────────────────────────────┐     ┌──────────────────────┐  │
│  │     AppSettings Store     │ ──► │       Shared Monitor       │ ──► │  macOS Notifications │  │
│  │ (interval, region, theme) │     │  (Tokio Polling Loop & State)│     │(tauri-plugin-notification)│
│  └───────────────────────────┘     └──────────────┬─────────────┘     └──────────────────────┘  │
│                                                   │                                             │
│                                    State Update & │ Diffing                                     │
│                                                   ▼                                             │
│                                    ┌────────────────────────────┐                               │
│                                    │   macOS System Tray Menu   │                               │
│                                    │ (Dynamic Icon Overlay &    │                               │
│                                    │   Native Status Items)     │                               │
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
│  │ (Theme, Routing, State Store)   │──►│ (Overall Status & Downtime)    │──►│(Interval, Region│  │
│  └─────────────────────────────────┘   └────────────────────────────────┘  │ Theme Selection)│  │
│                                                        │                   └─────────────────┘  │
│                                                        ▼                                        │
│                                        ┌────────────────────────────────┐                       │
│                                        │       <component-list>         │                       │
│                                        │  (Individual Feature Grid)     │                       │
│                                        │  (Git, Actions, Copilot, etc.)  │                       │
│                                        └────────────────────────────────┘                       │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Technology Stack

| Domain | Technology / Library | Purpose |
| :--- | :--- | :--- |
| **Desktop Runtime** | Tauri v2 (`tauri`, `tauri-build`) | Lightweight macOS desktop runtime |
| **Backend Language** | Rust (`tokio`, `reqwest`, `serde`, `chrono`) | Business logic, status fetching, state management |
| **System Tray & Autostart** | Tauri Tray API, `tauri-plugin-autostart` | macOS menubar item & macOS login item toggle |
| **Notifications** | `tauri-plugin-notification` | macOS system alerts on status change |
| **Persistence** | `tauri-plugin-store` or custom JSON file | Persistent user configuration store |
| **Frontend Framework**| Lit (`lit`) | Web Components UI library |
| **Frontend Language** | TypeScript (`typescript`) | Typed frontend scripting |
| **Styling & Theme** | Tailwind CSS (`tailwindcss`) + DaisyUI (`daisyui`) | Responsive styling and light/dark theme modes |
| **Build Tooling** | Vite (`vite`) | Fast frontend bundling and dev server |
| **Frontend Testing** | Vitest (`vitest`), `@open-wc/testing` | Unit and component testing |

---

## 📁 Project Directory Structure

```
github-monitor/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── vitest.config.ts
├── README.md
├── src/                          # Lit Frontend Web Components
│   ├── index.html
│   ├── index.ts
│   ├── styles/
│   │   └── main.css              # Tailwind & DaisyUI entry
│   ├── components/
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
    ├── icons/                    # App & Tray template icons
    │   ├── tray-icon.png
    │   └── tray-icon@2x.png
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── models.rs             # Status API, Config & State structs
        ├── monitor.rs            # Shared background polling service
        ├── tray.rs               # Dynamic Menubar & status overlay engine
        ├── autostart.rs          # Startup launch manager
        ├── config.rs             # Settings persistence layer
        ├── notification.rs       # System notification manager
        └── commands.rs           # Tauri IPC commands exposed to Lit
```

---

## 📋 Phase-by-Phase Technical Design

### Phase 1: Core Menubar Monitor & Background Engine

#### 1. System Tray & Visual Theme Adaptation
* **Icon Strategy**:
  * macOS template image (`tray-icon.png`) for dark/light menubar theme auto-matching.
  * Status indicator overlaid dynamically using Rust drawing/image buffer (`image` crate) or pre-rendered status variants:
    * 🟢 **Healthy**: Green dot (`#22c55e`)
    * 🟡 **Degraded**: Yellow dot (`#eab308`)
    * 🔴 **Severely Degraded / Down**: Red dot (`#ef4444`)
* **Tray Menu Specification**:
  * Item 1: `✅ GitHub status: Healthy` (or `⚠️ Degraded`, `❌ Unhealthy`).
    * *Click handler*: Opens `https://www.githubstatus.com` in default browser using Tauri shell API / `open::that`.
  * Item 2: Separator.
  * Item 3: `Start when logging in` (Checkable menuItem).
    * Default: `Unchecked` (off).
    * *Click handler*: Toggles autostart via `tauri-plugin-autostart`.
  * Item 4: Separator.
  * Item 5: `Quit` -> Terminates application.

#### 2. Rust Shared Monitor Service
* **Polling Loop**:
  * Implemented as an asynchronous `tokio::spawn` background task managed by `SharedMonitor`.
  * Default interval: 60 seconds.
  * Endpoint: `https://www.githubstatus.com/api/v2/summary.json`
* **Status Categorization**:
  * `indicator` mapping:
    * `none` ➔ `Healthy` (`✅`)
    * `minor` ➔ `Degraded` (`⚠️`)
    * `major` / `critical` ➔ `Unhealthy` (`❌`)
* **Notification System**:
  * Compares current payload with `previous_status`.
  * Triggers native macOS notification when `overall_status` shifts (e.g. `Healthy` ➔ `Degraded`).
  * Notification text: *"GitHub Status Changed: Service is now Degraded. Click for details."*

---

### Phase 2: Main Dashboard Window & Lit Web Components

#### 1. Main Dashboard Window
* **Tray Action**: Added menu item `Show the dashboard` to native Tray.
* **Window Configuration**: macOS native window, frameless/styled title bar, hidden on start or toggled via tray icon.

#### 2. Dashboard Interface Requirements
* **Overall Status Section**:
  * Visual badge with current status icon & color.
  * Hyperlink button to open `https://www.githubstatus.com` in default browser.
  * **Downtime Calculation**: When status is `Degraded` or `Unhealthy`, calculates and renders formatted elapsed time since service was last healthy (e.g. `Elapsed downtime: 1h 24m`).
* **Feature Components Grid**:
  * Lists individual GitHub services parsed from `components` array:
    * Git Operations, API Requests, Webhooks, Issues, Pull Requests, Actions, Packages, Pages, GitHub Copilot.
  * Each card displays:
    * Service Name
    * Status Badge (`Operational`, `Degraded Performance`, `Partial Outage`, `Major Outage`)
    * Description / Sub-status text.

#### 3. Frontend Architecture (Lit + TypeScript + DaisyUI)
* Web Components implemented using Lit `LitElement`:
  * `<app-root>`: Top-level layout with tab navigation (`Dashboard` | `Settings`).
  * `<status-banner>`: Hero section showing current status and elapsed downtime timer.
  * `<component-list>`: Grid displaying status cards for all components.
  * `<elapsed-timer>`: Custom timer component updating elapsed downtime string every minute.
* Styling configured via Tailwind CSS & DaisyUI theme variables (`data-theme`).

---

### Phase 3: Persistent Settings & Themes

#### 1. Persistent Config Engine
* Configuration saved to disk in JSON format (`app_settings.json`):
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
* **Theme Selection**:
  * Options: `System` (default), `Light`, `Dark`.
  * System mode listens to macOS system theme preference changes (`prefers-color-scheme`).
  * Updates DaisyUI root attribute `data-theme="light"` or `data-theme="dark"`.

---

## 📝 High-Level & Low-Level Development To-Do List

### 🚀 Phase 1: Menubar App & Backend Core

#### Backend & Rust Monitor (`src-tauri/`)
- [ ] Initialize Tauri v2 project structure with tray support enabled.
- [ ] Implement Rust data structures in `models.rs`:
  - [ ] `GitHubStatusSummary`, `StatusIndicator`, `Component`, `Page` serde structs.
  - [ ] `AppStatusState` (stores current status, previous status, last healthy timestamp).
- [ ] Create HTTP client service in `monitor.rs` using `reqwest`:
  - [ ] Fetch JSON from `https://www.githubstatus.com/api/v2/summary.json`.
  - [ ] Add error handling and offline fallback logic.
- [ ] Implement shared background polling loop (`tokio` async task):
  - [ ] Execute fetch every 60 seconds.
  - [ ] Track state transitions (`Healthy` <-> `Degraded` <-> `Unhealthy`).
  - [ ] Track `last_healthy_at` timestamp.
- [ ] Implement dynamic system tray icon rendering (`tray.rs`):
  - [ ] Load standard monochrome GitHub icon.
  - [ ] Overlay colored indicator dot: Green (Healthy), Yellow (Degraded), Red (Unhealthy).
  - [ ] Update tray menu text dynamically (`✅ GitHub status: Healthy`, etc.).
- [ ] Implement Native Tray Menu click handlers:
  - [ ] Clicking status header launches `https://www.githubstatus.com` in default browser.
- [ ] Implement autostart functionality (`autostart.rs`):
  - [ ] Integrate `tauri-plugin-autostart`.
  - [ ] Add menu item "Start when logging in" (checkbox item, default off).
  - [ ] Toggle startup item on/off on menu click.
- [ ] Implement macOS Notifications (`notification.rs`):
  - [ ] Integrate `tauri-plugin-notification`.
  - [ ] Trigger alert when overall status changes.

---

### 🎨 Phase 2: Lit Web Components Dashboard & UI Logic

#### Frontend Setup & Tech Stack Configuration
- [ ] Configure `package.json` with dependencies:
  - [ ] `lit`, `typescript`, `vite`, `tailwindcss`, `daisyui`, `postcss`, `autoprefixer`, `vitest`, `@open-wc/testing`.
- [ ] Configure `tailwind.config.js` and `postcss.config.js` with DaisyUI plugin and themes (`light`, `dark`).
- [ ] Set up `vitest.config.ts` for unit testing Web Components.

#### Rust Backend Extensions for UI
- [ ] Add tray menu item: `Show the dashboard` -> unhides and focuses main window.
- [ ] Expose Tauri IPC commands in `commands.rs`:
  - [ ] `get_current_status`: returns complete status summary and component array.
  - [ ] `open_status_page`: opens status URL in default browser.
- [ ] Emit Tauri IPC event (`status-updated`) whenever status refreshes in Rust.

#### Frontend Web Components (`src/components/`)
- [ ] Create `tauri-bridge.ts` service to wrap IPC calls and event listeners.
- [ ] Implement `<app-root>` element:
  - [ ] Navigation header / tab system (`Dashboard` / `Settings`).
  - [ ] Event listener for background status updates from Rust backend.
- [ ] Implement `<status-banner>` element:
  - [ ] Status icon badge & text description.
  - [ ] Link button to open `https://www.githubstatus.com` in browser.
- [ ] Implement `<elapsed-timer>` element:
  - [ ] Calculate and display formatted elapsed downtime (`X hours Y minutes`) when status is non-healthy.
- [ ] Implement `<component-list>` & `<component-card>` elements:
  - [ ] Grid layout displaying individual GitHub services (Git, Actions, Copilot, API, Webhooks, Issues, PRs, Packages, Pages).
  - [ ] Color-coded status badges for each component.

#### Frontend Testing (`src/tests/`)
- [ ] Write Vitest tests for `<status-banner>` rendering states (`Healthy`, `Degraded`, `Unhealthy`).
- [ ] Write Vitest tests for `<elapsed-timer>` downtime calculation logic.
- [ ] Write Vitest tests for `<component-list>` data binding.

---

### ⚙️ Phase 3: Persistent Settings & Complete Theme System

#### Rust Backend Settings Engine
- [ ] Implement persistent configuration store in `config.rs`:
  - [ ] Define `AppSettings` struct: `refresh_interval_minutes` (1–60), `github_region`, `theme`.
  - [ ] Load and save settings to local app data folder (`app_settings.json`).
- [ ] Expose Tauri IPC commands:
  - [ ] `get_settings`: fetch current persistent configuration.
  - [ ] `update_settings`: save updated settings to disk and re-configure backend components.
- [ ] Dynamically update `SharedMonitor` polling interval when `refresh_interval_minutes` is updated.

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
  - [ ] `System`: attach listener to `window.matchMedia('(prefers-color-scheme: dark)')` and apply matching theme dynamically.

---

## 🏃 Getting Started & Development Commands

### Prerequisites
* **Node.js**: `^18.0.0` or `^20.0.0`
* **Rust**: `^1.75.0` and `cargo`
* **macOS**: macOS 11+ (Big Sur or newer)

### Development Commands

```bash
# Install dependencies
npm install

# Run frontend unit tests
npm run test

# Run Vitest in watch mode
npm run test:watch

# Start Tauri app in development mode
npm run tauri dev

# Build production macOS binary (.app / .dmg)
npm run tauri build
```
