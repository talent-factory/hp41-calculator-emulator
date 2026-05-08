# Requirements: HP-41 Calculator Emulator v1.1

**Defined:** 2026-05-08
**Core Value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

## v1.1 Requirements

### Input Behavior (EEX Fix)

- [ ] **INPUT-01**: User pressing EEX then an op key (without typing exponent digits) commits the number with exponent 00 (e.g. `1.5e` + ENTER → `1.5`)
- [ ] **INPUT-02**: User pressing EEX on an empty entry buffer sees `1` inserted as implicit mantissa (matches HP-41 hardware: `1   _` in exponent entry mode)
- [ ] **INPUT-03**: TUI displays a placeholder cursor during partial exponent state (e.g. `1.5E_ _`) to signal exponent entry is pending

### STO Arithmetic Modals

- [ ] **STOA-01**: User can perform STO+, STO-, STO×, STO÷ to R00–R99 via 3-step keyboard modal (`S` → arithmetic op key → two-digit register)
- [ ] **STOA-02**: User can press Esc at any step of the STO arithmetic modal to cancel without side effects
- [ ] **STOA-03**: User can perform STO arithmetic to stack registers (STO+ Y, STO- Z, STO× T, STO÷ L)

### Print Emulation

- [ ] **PRNT-01**: `PRX` prints X register in current display format (FIX/SCI/ENG), right-aligned to 24 chars, to console
- [ ] **PRNT-02**: `PRA` prints ALPHA register contents, left-aligned to 24 chars, to console
- [ ] **PRNT-03**: `PRSTK` prints full stack in order T→Z→Y→X→LASTX→ALPHA to console
- [ ] **PRNT-04**: User can pass `--print-log <path>` to `hp41-cli` to append all print output to a file in addition to console

### Synthetic Programming

- [ ] **SYNT-01**: `GETKEY` instruction pushes the last key code (HP-41 row-column encoding) to X register
- [ ] **SYNT-02**: `NULL` instruction executes as a no-op with Neutral stack-lift effect
- [ ] **SYNT-03**: Hidden registers M, N, O are accessible in programs via `STO M`/`RCL M`, `STO N`/`RCL N`, `STO O`/`RCL O`
- [ ] **SYNT-04**: User can insert a synthetic op into the current program via a 2-digit hex byte modal (curated safe subset; rejects mode-mutating byte codes)

### Infrastructure

- [ ] **INFRA-01**: Project MSRV formally bumped to Rust 1.85 in `Cargo.toml` and CI/docs (resolves `clap` 4.6.1 discrepancy); `rust_decimal` bumped 1.41 → 1.42

## v2+ Requirements

### Print Emulation (deferred)

- **PRNT-05**: Scrollable print history panel in TUI — complexity vs. v1.1 scope
- **PRNT-06**: `ADV` (paper advance), `PRREG` (print all registers), Flag 26 / TRACE mode — niche printer peripheral ops

### Synthetic Programming (deferred)

- **SYNT-05**: Full FOCAL byte-code table (~200 codes) — byte-code reference not fully available at v1.1 planning
- **SYNT-06**: `GETKEY` interrupt-style key capture during program execution — requires event loop redesign

### STO Arithmetic (deferred)

- **STOA-04**: STO arithmetic via indirect addressing (e.g. `STO+ IND NN`) — v1.2+

## Out of Scope

| Feature | Reason |
|---------|--------|
| Tauri v2 GUI (hp41-gui) | Deferred to v2.0; this milestone is CLI-only |
| Full FOCAL / Nut CPU emulation | Architecture decision: behavioral emulation, not cycle-accurate |
| HP-copyrighted ROM images | Legal risk; excluded permanently |
| HP-IL / peripheral bus emulation | Niche, high complexity |
| Cloud sync or telemetry | Privacy; local-only data storage |
| Byte-grabber / ROM exploit replication | Hardware-only technique; not reproducible in behavioral emulator |
| `println!` / direct I/O in `hp41-core` | Enforced invariant: zero UI dependencies in core library |

## Traceability

Populated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| INFRA-01    | —     | Pending |
| INPUT-01    | —     | Pending |
| INPUT-02    | —     | Pending |
| INPUT-03    | —     | Pending |
| STOA-01     | —     | Pending |
| STOA-02     | —     | Pending |
| STOA-03     | —     | Pending |
| PRNT-01     | —     | Pending |
| PRNT-02     | —     | Pending |
| PRNT-03     | —     | Pending |
| PRNT-04     | —     | Pending |
| SYNT-01     | —     | Pending |
| SYNT-02     | —     | Pending |
| SYNT-03     | —     | Pending |
| SYNT-04     | —     | Pending |

**Coverage:**
- v1.1 requirements: 15 total
- Mapped to phases: 0 (pending roadmap)
- Unmapped: 15 ⚠️

---
*Requirements defined: 2026-05-08*
*Last updated: 2026-05-08 after initial definition*
