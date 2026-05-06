# Technology Stack

**Project:** HP-41 Calculator Emulator
**Researched:** 2026-05-06

---

## Recommended Stack

### Rust Toolchain

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust stable | 1.85+ | Language | 2024 edition required for async closures, let-chains, improved temporary scoping. Matches clap 4.6 and proptest 1.11 MSRV. PROJECT.md specifies 1.78+ minimum; bumping to 1.85 costs nothing for a greenfield project. |
| Cargo workspace | resolver v3 (2024 edition) | Monorepo | Enforces `hp41-core` / `hp41-cli` / `hp41-gui` boundary at the build system level. Dependency deduplication cuts clean build time 40-60% on medium projects. |
| Edition | 2024 | Language semantics | Stabilised in Rust 1.85 (2025-02-20). Use it from day one — migrating later is painful. |

### TUI Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **ratatui** | 0.30.0 | Terminal UI rendering | The de-facto successor to the abandoned `tui-rs`. Immediate-mode rendering with intermediate buffers. Rich widget set (Block, Paragraph, Table, List, Gauge, Sparkline). 30-40% less memory and 15% lower CPU vs. equivalent Go Bubbletea apps. Actively maintained with a large ecosystem (awesome-ratatui). CrosstermBackend is the default. |

**Confidence: HIGH** — verified against crates.io (released 2025-12-26), official docs at ratatui.rs, and ecosystem research.

Do NOT use: `tui-rs` (unmaintained, archived), `cursive` (retained-mode, harder to adapt to HP-41 frame-by-frame redraw pattern).

### Terminal Backend / Input Handling

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **crossterm** | 0.29.0 | Terminal raw mode, keyboard events, cursor, color | Default and recommended backend for ratatui. The only backend with full Windows 10+ support (termion is Unix-only). Ratatui's own docs flowchart says "choose Crossterm if Windows matters." Consistent API across Windows, macOS, Linux. |

**Confidence: HIGH** — confirmed by ratatui.rs/concepts/backends/comparison/, crates.io (released 2025-04-05).

Do NOT use: `termion` (Unix-only, would break NFR-5 Windows 10+ requirement), `termwiz` (WezTerm-specific; over-engineered for this use case).

Note on version pinning: Ratatui 0.30 warns against pulling in semver-incompatible crossterm versions in the same binary (causes race conditions and raw-mode restore failures). Use `crossterm = "0.29"` and let ratatui re-export what it needs.

### CLI Argument Parsing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **clap** | 4.6.1 | `--help`, `--version`, `--state-file`, `--program` flags | The standard Rust CLI parser. Derive macro (`#[derive(Parser)]`) keeps arg definitions next to structs. Excellent shell-completion generation via `clap_complete`. No realistic alternative with the same ecosystem maturity. |

**Confidence: HIGH** — verified against crates.io (released 2026-04-15).

Do NOT use: `structopt` (merged into clap 3+, deprecated), `pico-args` (too minimal for the help/completion story).

Use features: `derive` (required), `env` (for `HP41_STATE_FILE` env var override), `wrap_help` (nicer terminal wrapping).

### Serialization

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **serde** | 1.0.228 | Derive `Serialize`/`Deserialize` on all state types | Industry standard. Zero-copy deserialization where needed. |
| **serde_json** | 1.0.149 | State files, program files, save/load (FR-10) | Human-readable format is non-negotiable for a calculator emulator — users share programs as text files. JSON is universally understood. The format must be versioned (`"schema_version": 1` field) for forward-compatible migration. |

**Confidence: HIGH** — verified against crates.io.

Do NOT use: `bincode`/`postcard` as the primary format — binary formats are unreadable, complicating the program-sharing use case. Fine as a secondary fast-path cache if ever needed.

Consider: wrapping the JSON schema version in a top-level envelope so `serde_json::from_value` can branch on version before full deserialization (avoids breaking existing save files on schema changes).

### Async Runtime

**Verdict: Do NOT add tokio for v1.0.**

Ratatui's own FAQ states: "Ratatui isn't a native async library." For a keyboard-driven calculator emulator, the event loop is:

```
poll crossterm event (blocking, timeout = 30 s auto-save interval)
→ update hp41-core state (pure, synchronous, < 1 ms)
→ render frame (ratatui draw, < 1 ms)
```

There are no concurrent background operations in v1.0. Auto-save can run in the same thread after a poll timeout. Adding tokio for this use case imposes compile-time overhead (adds ~3 s to cold builds on CI), binary size overhead (~500 KB), and async function coloring complexity with zero payoff.

**If v2.0 Tauri requires async channels** between the Rust backend and web frontend, tokio enters via the Tauri dependency — but that stays in `hp41-gui`, not `hp41-core` or `hp41-cli`.

**Confidence: HIGH** — ratatui FAQ explicitly confirmed, event model analysis matches the HP-41 interaction model.

### Error Handling

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **thiserror** | 2.0.18 | Error types in `hp41-core` | Derive macro for `std::error::Error`. Keeps core error types ergonomic without pulling in heavy dependencies. |
| `anyhow` | (if needed) | Top-level error propagation in `hp41-cli` | For `main()` and CLI glue code where detailed error types are less important than useful messages. Evaluate at implementation time — may not be needed. |

**Confidence: HIGH** — thiserror 2.x verified against crates.io (released 2026-01-18).

---

## Testing Stack

### Unit and Integration Testing

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Rust built-in `#[test]` | stdlib | Unit tests in `hp41-core` | All pure computation tests: stack operations, arithmetic, trig, register operations, stack-lift semantics |
| **proptest** | 1.11.0 | Property-based testing for numerical correctness | Testing that HP-41 arithmetic invariants hold across arbitrary f64 inputs (e.g., `sin(x)^2 + cos(x)^2 ≈ 1.0`); regression to NFR-7 (≥98% numerical agreement with 500 reference cases) |
| **insta** | 1.47.2 | Snapshot testing | Testing display formatting (FIX/SCI/ENG modes), annunciator state strings, program listing output — any "string that should look exactly like this" scenario |
| **approx** (0.5.1) or `float-cmp` (0.10.0) | stable | Floating-point approximate equality | Asserting results within HP-41's 10-digit BCD precision tolerance. Use `assert_relative_eq!` from `approx` or `assert_approx_eq!` from `float-cmp`. |
| `ratatui::backend::TestBackend` | 0.30.0 | TUI widget rendering tests | Rendering `Stack`, `Display`, `Annunciator` widgets to a buffer and asserting cell content — avoids needing a real terminal in CI |

**Why proptest over quickcheck:** proptest 1.11 requires Rust 1.85, has better shrinking, and is more actively maintained than quickcheck 1.1. For the 500-case HP-41 reference test suite (NFR-7), proptest's strategy combinators let you express "any valid HP-41 number" precisely.

**Confidence: HIGH for proptest/insta** — verified against crates.io. **MEDIUM for approx** — 0.5.1 is the stable release but 0.6.0-rc2 exists; 0.5.1 is safe to pin.

### Coverage

| Tool | Version | Purpose |
|------|---------|---------|
| **cargo-llvm-cov** | 0.8.5 | LLVM source-based coverage, targeting ≥80% on `hp41-core` (NFR-4) |

Run in CI with:
```bash
cargo llvm-cov --workspace --exclude hp41-cli --lcov --output-path lcov.info
```

Note: `x86_64-pc-windows-gnu` target does not work with cargo-llvm-cov — use `x86_64-pc-windows-msvc` on Windows CI runners.

---

## CI and Release Tooling

| Tool | Purpose | Why |
|------|---------|-----|
| GitHub Actions | CI runner | Native Windows, macOS, Linux runners; matrix strategy for cross-platform |
| `dtolnay/rust-toolchain` action | Pin Rust stable | Reproducible builds |
| `houseabsolute/actions-rust-cross` | Cross-compilation | ARM Linux targets (aarch64-unknown-linux-musl) if desired; native cargo for Windows/macOS |
| **cargo-dist** 0.31.0 | Release packaging | Automated GitHub Release creation with .tar.gz (Linux/macOS) and .zip (Windows) artifacts; installer scripts; designed for Rust CLI tools |
| `taiki-e/install-action@cargo-llvm-cov` | Install coverage tool in CI | Fastest way to get cargo-llvm-cov on runners |

**Confidence: MEDIUM** — cargo-dist 0.31.0 verified; cross-compilation matrix pattern verified via multiple 2025 blog posts. Exact workflow YAML requires validation against runner images at implementation time.

---

## Cargo.toml Dependency Summary

```toml
# hp41-core/Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"

[dev-dependencies]
proptest = "1.11"
approx = "0.5"
insta = { version = "1.47", features = ["json"] }

# hp41-cli/Cargo.toml
[dependencies]
hp41-core = { path = "../hp41-core" }
ratatui = { version = "0.30", default-features = true }   # pulls in crossterm 0.29 backend
crossterm = "0.29"
clap = { version = "4.6", features = ["derive", "env", "wrap_help"] }
serde_json = "1.0"
thiserror = "2.0"

[dev-dependencies]
insta = "1.47"
```

Do NOT add `tokio` to either crate for v1.0.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| TUI | ratatui 0.30 | cursive | Retained-mode; less suited to HP-41's fixed-frame layout; smaller ecosystem |
| TUI | ratatui 0.30 | tui-rs | Archived/unmaintained; ratatui is its direct successor |
| TUI | ratatui 0.30 | iocraft | Experimental React-like model; immature, no production track record |
| Backend | crossterm | termion | Unix-only; breaks Windows support (NFR-5) |
| Backend | crossterm | termwiz | WezTerm-specific; no meaningful advantage here |
| CLI args | clap 4 | structopt | Merged into clap 3+; use clap directly |
| CLI args | clap 4 | pico-args | Too minimal; no help generation or shell completion |
| Serialization | serde_json | bincode | Binary format; users can't inspect/share program files |
| Serialization | serde_json | ron | Rust-centric; lower tool support for non-Rust users |
| Async | none (sync) | tokio | Zero benefit for synchronous keyboard-driven emulator; adds compile overhead |
| Error handling | thiserror | anyhow in core | anyhow erases error types; unacceptable in a library crate |
| Property testing | proptest | quickcheck | proptest has better shrinking and is more actively maintained |
| Coverage | cargo-llvm-cov | tarpaulin | tarpaulin is Linux-only; cargo-llvm-cov works on all three target platforms |

---

## Sources

- ratatui crates.io: https://crates.io/crates/ratatui (v0.30.0, released 2025-12-26)
- crossterm crates.io: https://crates.io/crates/crossterm (v0.29.0, released 2025-04-05)
- clap crates.io: https://crates.io/crates/clap (v4.6.1, released 2026-04-15)
- serde crates.io: https://crates.io/crates/serde (v1.0.228)
- serde_json crates.io: https://crates.io/crates/serde_json (v1.0.149)
- thiserror crates.io: https://crates.io/crates/thiserror (v2.0.18)
- proptest crates.io: https://crates.io/crates/proptest (v1.11.0, released 2026-03-24)
- insta crates.io: https://crates.io/crates/insta (v1.47.2, released 2026-03-30)
- approx crates.io: https://crates.io/crates/approx (v0.5.1 stable)
- cargo-llvm-cov crates.io: https://crates.io/crates/cargo-llvm-cov (v0.8.5, released 2026-03-20)
- cargo-dist crates.io: https://crates.io/crates/cargo-dist (v0.31.0, released 2026-02-23)
- ratatui backend comparison: https://ratatui.rs/concepts/backends/comparison/
- ratatui FAQ (async): https://ratatui.rs/faq/
- Rust 1.85 / 2024 edition: https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/
- ratatui vs tui-rs ecosystem: https://github.com/ratatui/ratatui
- Cross-platform CI patterns: https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/
