# `hp41-gui` — Desktop GUI

The HP-41 desktop app is a [Tauri v2](https://v2.tauri.app/) application
(Rust backend + React 18 / TypeScript / Vite frontend). It is a **nested
standalone Cargo workspace**: the `tauri` / `tauri-build` dependencies live
only here and never enter the root workspace resolver, so
`cargo build --workspace` from the repo root does not touch this crate.

All commands below are driven through `just` from the **repository root**.

## Prerequisites

Common to every platform:

- **Rust** stable, MSRV **1.88** (`rustup` recommended)
- **`just`** — `cargo install just`
- **Node.js 18+** and **npm**

In addition, Tauri needs OS-specific system libraries. The lists below cover
the common cases; the authoritative, always-current list is the
[Tauri v2 prerequisites guide](https://v2.tauri.app/start/prerequisites/).

### macOS

Install the Xcode Command Line Tools (provides the C toolchain and WebKit):

```bash
xcode-select --install
```

No additional packages are required — macOS ships WKWebView system-wide.

### Windows

- **Microsoft C++ Build Tools** (the "Desktop development with C++" workload
  from the Visual Studio Build Tools installer).
- **WebView2 runtime** — preinstalled on Windows 10 21H2+ and Windows 11. On
  older builds, install the
  [Evergreen WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

### Linux (Debian / Ubuntu)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

For Fedora, Arch, and other distributions, see the per-distro package lists in
the [Tauri v2 prerequisites guide](https://v2.tauri.app/start/prerequisites/).

## Build & run

Run these from the repository root:

```bash
just gui-install        # install GUI deps (npm); also pulls in the Tauri CLI
just gui-dev            # launch the Tauri dev window (Rust hot-reload + Vite HMR)
just gui-build          # release build (produces a native bundle)
just gui-ci             # GUI gate: permission check + npm ci + audit + tsc --noEmit
just gui-check          # fast Rust type-check (cargo check on src-tauri)
```

> Run `just gui-install` (or `just gui-build`) **before** `just gui-dev` on a
> fresh checkout: the Tauri CLI is provided by `@tauri-apps/cli` via npm, so the
> dev window cannot launch until the GUI dependencies have been installed.

## Notes

- The GUI and CLI share state via `~/.hp41/autosave.json` (auto-saved every
  30 s; each loads the other's state on launch).
- Bundle identifier: `ch.talent-factory.hp41`.
