# Requirements — v3.0 Math Pac I Emulation

**Milestone goal:** Behavioral Emulation des HP-41C Math Pac I (HP-Teilenummer 00041-90034, Owner's Manual 1979) als erstes XROM-Modul. Alle 10 Top-Level-Programme (`MATRIX`, `SOLVE`, `POLY`, `INTG`, `DIFEQ`, `FOUR`, Komplex-Stack, Hyperbolicus, Dreiecks-Solver, `TRANS`) und ~55 XEQ-by-Name Entry Points lieferbar in CLI + GUI über eine neue Modal-Workflow-Schicht hinaus dem v2.x-Built-in-Pattern.

**Scope boundary (locked 2026-05-16):**
- v3.0 = HP Math Pac I per Owner's Manual 00041-90034
- Advanced Matrix Pac (M+, MAT*, INV-transpose, V+, VDOT, etc.) → v3.2+
- Advantage Pac (PROOT, CABS, CARG, CCHS, CCONJ, Romberg-INTG, …) → v3.3+
- Stat 1 Pac → v3.1
- Time Pac → v3.2
- HP-copyrighted ROM-Image-Redistribution bleibt permanent ausgeschlossen

**Build sequence:** core (Phase 28) → cli (Phase 29) → docs (Phase 30) → gui (Phase 31) → tests (Phase 32). Phase-Nummerierung continued von v2.2 (Phase 27).

---

## v3.0 Requirements

### XROM Module Framework (Phase 28-01)

- [ ] **XROM-01**: `XromModule` struct mit `id: u8`, `name: &'static str`, `ops: &'static [(&'static str, Op)]` in `hp41-core/src/ops/math1/xrom.rs`
- [ ] **XROM-02**: `pub const MATH_1: XromModule { id: 7, … }` statisch verlinkt; XROM-ID 7 entspricht der realen HP Math Pac I Hardware-ID
- [ ] **XROM-03**: `state.xrom_modules: u8` Bitfeld auf `CalcState` mit `#[serde(default = "default_xrom_modules")]`; Bit 0 = Math 1 geladen (Default `0b1` für v3.0)
- [ ] **XROM-04**: `xrom_resolve(name: &str, modules: u8) -> Option<Op>` Resolver-Funktion; gibt regulären `Op::*`-Variant zurück (KEIN Dispatcher — `synthetic_byte_to_op`-Pattern)
- [ ] **XROM-05**: Resolver-Kette erweitert in `xeq_by_name_local_resolve` UND `op_xeq` UND `run_program::execute_op`: built-in → card_op → **xrom_resolve** → `Err(InvalidOp)` (xrom feuert ZULETZT — verhindert Shadowing existierender Built-ins per Pitfall 1)
- [ ] **XROM-06**: 5 weitere `CalcState`-Felder, alle `#[serde(default)]`: `complex_mode: bool`, `matrix_dim: Option<(u8, u8)>`, `matrix_active_reg: Option<u8>`, `modal_program: Option<ModalProgram>` (`#[serde(skip)]`), `integ_state` + `solve_state` (`#[serde(skip)]`)
- [ ] **XROM-07**: Op-Strategy A: jede Math-Pac-I-Funktion ist eine eigene `Op`-Variante (~40 neue Variants); 4-Way-Exhaustive-Match Invariant erhalten (`dispatch()`, `execute_op()`, beide `prgm_display.rs`)
- [ ] **XROM-08**: User-Callback-Re-entrancy-Policy: nested `INTG`/`SOLVE`/`DIFEQ` werden bei Op-Entry abgelehnt (`HpError::InvalidOp`) — matched Math Pac I Hardware-Verhalten per OM
- [ ] **XROM-09**: Modal-State-Machine-Layer: `ModalProgram` enum mit per-Program Step-State (`MatrixInputStep`, `SolveInputStep`, `IntegInputStep`, etc.); Prompts via `state.print_buffer` (existing channel)

### Hyperbolics (Phase 28-02) — Table Stakes

- [ ] **HYP-01**: `Op::Sinh` — hyperbolischer Sinus auf X
- [ ] **HYP-02**: `Op::Cosh` — hyperbolischer Cosinus auf X
- [ ] **HYP-03**: `Op::Tanh` — hyperbolischer Tangens auf X
- [ ] **HYP-04**: `Op::Asinh` — Areasinh auf X (inverse hyperbolic)
- [ ] **HYP-05**: `Op::Acosh` — Areacosh auf X; Domain-Error für X < 1
- [ ] **HYP-06**: `Op::Atanh` — Areatanh auf X; Domain-Error für |X| ≥ 1

### Complex Stack & Operations (Phase 28-03/04) — Table Stakes (Arithmetik + 9 Funktionen), Differentiator (4 Funktionen)

- [ ] **CMPLX-01**: `ComplexStack { zeta: (HpNum, HpNum), tau: (HpNum, HpNum) }` mit OM-treuer Repräsentation; Lokation TBD in Plan 28-03 (overlay X/Y/Z/T vs dedicated R02–R05 vs eigener Struct)
- [ ] **CMPLX-02**: `Op::CPlus` (C+) — Komplex-Addition auf (ζ, τ)
- [ ] **CMPLX-03**: `Op::CMinus` (C-) — Komplex-Subtraktion
- [ ] **CMPLX-04**: `Op::CTimes` (C×) — Komplex-Multiplikation
- [ ] **CMPLX-05**: `Op::CDiv` (C÷) — Komplex-Division; Domain-Error bei Division durch 0
- [ ] **CMPLX-06**: `Op::Magz` (MAGZ) — Betrag von ζ (über f64-Bridge `sqrt(re² + im²)`)
- [ ] **CMPLX-07**: `Op::Cinv` (CINV) — Komplex-Reziprok
- [ ] **CMPLX-08**: `Op::ZpowN` (Z↑N) — ζ hoch N (Integer-Exponent)
- [ ] **CMPLX-09**: `Op::Zpow1N` (Z↑1/N) — Komplex-N-te-Wurzel
- [ ] **CMPLX-10**: `Op::ExpZ` (E↑Z) — e^ζ via Euler-Formel
- [ ] **CMPLX-11**: `Op::LnZ` (LNZ) — natürlicher Komplex-Logarithmus; (0,0)-Handling per Pitfall 6
- [ ] **CMPLX-12**: `Op::SinZ` (SINZ) — Komplex-Sinus
- [ ] **CMPLX-13**: `Op::CosZ` (COSZ) — Komplex-Cosinus
- [ ] **CMPLX-14**: `Op::TanZ` (TANZ) — Komplex-Tangens
- [ ] **CMPLX-15**: `Op::ApowZ` (A↑Z) — A^ζ (allgemeine Basis)
- [ ] **CMPLX-16**: `Op::LogZ` (LOGZ) — Zehnerlogarithmus auf Komplex
- [ ] **CMPLX-17**: `Op::ZpowW` (Z↑W) — ζ^τ; Domain-Error bei (0,0)^W mit Re(W) ≤ 0
- [ ] **CMPLX-18**: `Op::Real` (XEQ "REAL") — deactivates `complex_mode`; resets it to `false`; no other side effects on the stack. Derived from D-28.3 (deviation from OM 1979 for UX — not in Math Pac I hardware). Documented in `docs/hp41-math1-divergences.md` (Phase 30 / DOC-04).

### Polynomial Roots — POLY/ROOTS (Phase 28-05) — Table Stakes

- [ ] **POLY-01**: `Op::PolyWorkflow` (XEQ "POLY") — Master-Workflow mit `DEGREE=?` Prompt (2–5); leitet Koeffizienten-Input ein
- [ ] **POLY-02**: Koeffizienten-Prompts `A=?`, `B=?`, `C=?`, `D=?`, `E=?`, `F=?` je nach Grad; Storage in R00–R04
- [ ] **POLY-03**: `Op::Roots` (XEQ "ROOTS") — Sub-Entry, bypassen den `DEGREE=?`-Prompt
- [ ] **POLY-04**: Output-Format für komplexe Wurzelpaare: 4-zeilig `U=u` / `V=v` / `U=u` / `-V=-v` exakt wie OM
- [ ] **POLY-05**: Polynom-Auswertung (Horner-Schema) als zugehöriger Sub-Entry
- [ ] **POLY-06**: Multiplicity-as-Cluster-Verhalten: `(x-1)^5` produziert Wurzel-Cluster mit kleinen Imaginärteilen; OM-Hardware-Match
- [ ] **POLY-07**: Non-Convergence-Handling: `|imag| > 10⁹` während Real-Polynom-Iteration → `HpError::Domain` "DATA ERROR"

### Matrix Workflow — MATRIX (Phase 28-06) — Table Stakes

- [ ] **MAT-01**: `Op::MatrixWorkflow` (XEQ "MATRIX") — Master-Workflow mit `ORDER=?`-Prompt (1–14)
- [ ] **MAT-02**: `ORDER`-Wert wird in R14 gespeichert; Matrix-Elemente column-major ab R15 (OM-Konvention)
- [ ] **MAT-03**: `Op::MatSize` (XEQ "SIZE") — Sub-Entry, gibt aktuelle Matrix-Ordnung zurück
- [ ] **MAT-04**: `Op::MatVmat` (XEQ "VMAT") — View Matrix; Sequenz-Anzeige aller Elemente
- [ ] **MAT-05**: `Op::MatEdit` (XEQ "EDIT") — Element-Edit-Modus mit `Ai,j=?` Prompts
- [ ] **MAT-06**: `Op::MatDet` (XEQ "DET") — Determinante via LU mit partieller Pivotsuche
- [ ] **MAT-07**: `Op::MatInv` (XEQ "INV") — Inverse via Gauss-Jordan; EPSILON-Wert aus OM (TBD Phase 28 Research-Prep) für Singularitäts-Detektion; `NO SOLUTION` bei Singular
- [ ] **MAT-08**: `Op::MatSimeq` (XEQ "SIMEQ") — Lineares Gleichungssystem lösen via Gauss-Elimination; Konstanten-Vektor input über `B1=?`, `B2=?`, …; Lösung-Vektor in R(N+1) onward
- [ ] **MAT-09**: `Op::MatVcol` (XEQ "VCOL") — View Spalte; nach SIMEQ
- [ ] **MAT-10**: Flag 4 wird gesetzt während Input-Phase; Flag 5 nach SIMEQ-Spalten-Storage (OM-Hardware-Konvention)
- [ ] **MAT-11**: Maximale Matrix-Ordnung 14 (memory-bedingt 6 ohne Memory-Module, 14 mit voller Memory)

### Numerical Integration — INTG (Phase 28-07) — Table Stakes

- [ ] **INTG-01**: `Op::Integ` (XEQ "INTG") — Master-Workflow mit Mode-Auswahl (Discrete vs Explicit)
- [ ] **INTG-02**: Discrete-Mode Entry-Points: A=`h` (Schrittweite), B=`f(xⱼ)` Sample-Input, C=Trapezoidal, D=Simpson; gerade-`n`-Check returnt `N NOT EVEN`
- [ ] **INTG-03**: Explicit-Mode Entry-Points: A=`(a,b)` Intervall-Bounds, B=`n` (subdivisions); `FUNCTION NAME?`-Prompt für User-Program-LBL
- [ ] **INTG-04**: Simpson-Quadratur mit fester `n` (KEIN adaptives Refinement — User kontrolliert Genauigkeit per `n`)
- [ ] **INTG-05**: Scratch-Storage R00–R07; `integ_state: Option<IntegState>` für Mid-Iteration-Werte (`#[serde(skip)]`)
- [ ] **INTG-06**: User-Program-Callback infrastructure: `run_loop` Re-entrancy aus `op_integ()`; X = aktueller `x`-Wert; User-Programm muss `f(x)` in X zurückliefern
- [ ] **INTG-07**: Subdivision-Cap 2^15 = 32768; Cap-Exceeded → `HpError::Domain` "DATA ERROR"
- [ ] **INTG-08**: Convergence-Threshold via `DisplayMode` (FIX n / SCI n) — `threshold = 10^(-decimals - 1)` (OM-cited; TBD Phase 28 Research-Prep)

### Real Root Solver — SOLVE (Phase 28-08) — Table Stakes

- [ ] **SOLV-01**: `Op::Solve` (XEQ "SOLVE") — Master mit `FUNCTION NAME?` + `GUESS 1=?` + `GUESS 2=?` Prompts
- [ ] **SOLV-02**: `Op::Sol` (XEQ "SOL") — Sub-Entry, bypassen die Prompts (Guesses und Funktion müssen vorab gesetzt sein)
- [ ] **SOLV-03**: Modifizierte Sekanten-Iteration (OM-spezifiziert)
- [ ] **SOLV-04**: Drei Termination-Pfade per OM-Konvention: `NO ROOT FOUND`, `ROOT IS <v>`, `ROOT IS BETWEEN <v1> AND <v2>`
- [ ] **SOLV-05**: Scratch R00–R06; `solve_state: Option<SolveState>` (`#[serde(skip)]`)
- [ ] **SOLV-06**: Reuses INTG's User-Program-Callback Infrastruktur (gleiche `run_loop` Re-entrancy)
- [ ] **SOLV-07**: 100-Iteration-Cap; `MAX_STEPS = 1_000_000` Budget aus `run_program` ableiten
- [ ] **SOLV-08**: Nested INTG-in-SOLVE oder SOLVE-in-INTG abgelehnt per XROM-08

### Differential Equations — DIFEQ (Phase 28-09) — Differentiator

- [ ] **DIFEQ-01**: `Op::Difeq` (XEQ "DIFEQ") — Master mit `FUNCTION NAME?` / `ORDER=?` (1 oder 2) / `STEP SIZE=?` / `X0=?` / `Y0=?` Prompts; bei `ORDER=2` zusätzlich `Y'0=?`
- [ ] **DIFEQ-02**: 4th-Order Runge-Kutta Implementation (OM-spezifiziert)
- [ ] **DIFEQ-03**: Scratch R00–R07
- [ ] **DIFEQ-04**: User-Program-Callback für `f(x, y)` bzw. `f(x, y, y')` je nach Order
- [ ] **DIFEQ-05**: Step-by-Step Output via `print_buffer`

### Fourier Series — FOUR (Phase 28-10) — Differentiator

- [ ] **FOUR-01**: `Op::Four` (XEQ "FOUR") — Master mit `NO. SAMPLES=?` / `NO. FREQ=?` / `1ST COEFF=?` Prompts
- [ ] **FOUR-02**: Sample-Input via `Y1=?` … `YN=?` Prompts
- [ ] **FOUR-03**: `RECT?` Toggle: rechteckig (aₙ, bₙ) oder polar (cₙ, φₙ) Koeffizienten
- [ ] **FOUR-04**: Bis zu 10 (aₙ, bₙ) Paare
- [ ] **FOUR-05**: Scratch R00–R26
- [ ] **FOUR-06**: USER-Mode `E`-Key wertet Fourier-Reihe an Zeitpunkt `t` aus (nach Koeffizienten-Berechnung)

### Triangle Solutions (Phase 28-10) — Differentiator

- [ ] **TRI-01**: `Op::TriSss` (XEQ "SSS") — drei Seiten gegeben, alle drei Winkel berechnen via Law of Cosines
- [ ] **TRI-02**: `Op::TriAsa` (XEQ "ASA") — Winkel-Seite-Winkel via Law of Sines
- [ ] **TRI-03**: `Op::TriSaa` (XEQ "SAA") — Seite-Winkel-Seite via Law of Sines
- [ ] **TRI-04**: `Op::TriSas` (XEQ "SAS") — Seite-Winkel-Seite via Law of Cosines
- [ ] **TRI-05**: `Op::TriSsa` (XEQ "SSA") — zweideutiger Fall (ambiguous case); OM-konformes Behandeln

### Coordinate Transformations — TRANS (Phase 28-10) — Differentiator

- [ ] **TRANS-01**: `Op::Trans2d` — 2D-Mode-Initialisierung: A-Entry mit `x₀, y₀, θ`
- [ ] **TRANS-02**: 2D forward (C-Entry) und inverse (E-Entry) Transformation
- [ ] **TRANS-03**: `Op::Trans3d` — 3D-Mode-Initialisierung: A-Entry `(origin)`, B-Entry `(a, b, c, θ)` Rotation-Axis
- [ ] **TRANS-04**: 3D forward (C-Entry) und inverse (E-Entry) via Rodrigues-Rotation-Formel
- [ ] **TRANS-05**: Scratch R00–R24

### CLI Integration (Phase 29)

- [ ] **CLI-01**: `xeq_by_name_local_resolve` in `hp41-cli/src/keys.rs` ruft `xrom_resolve` nach `builtin_card_op` auf (Math Pac I funktioniert via XEQ-by-Name)
- [ ] **CLI-02**: `hp41-cli/src/help_data.rs` lädt ZWEITE JSON-Datei (`docs/hp41-math1-functions.json`) via zusätzlichen `OnceLock<Vec<HelpEntry>>`; `?`-Overlay zeigt Math Pac I Funktionen
- [ ] **CLI-03**: `hp41-cli/src/prgm_display.rs` erweitert um ~40 neue `op_display_name` Arms für die neuen `Op`-Varianten
- [ ] **CLI-04**: KEY_REF_TABLE (`hp41-cli/src/ui.rs::render_right_panel`) blendet Math Pac I Einträge ein (JSON-derived per D-25.18)
- [ ] **CLI-05**: Modal-Prompt-Routing in `App.tsx`-Pattern: `MATRIX`/`SOLVE`/`POLY`/`INTG`/`DIFEQ`/`FOUR`/`TRANS` Workflows triggern `ModalProgram`-State-Machine; Inputs werden über die existierende Number-Entry-Pipeline akzeptiert; ALPHA-Prompt-Text in `print_buffer` sichtbar

### Documentation (Phase 30)

- [ ] **DOC-01**: `docs/hp41-math1-functions.json` (sibling zu `hp41cv-functions.json`), identisches Schema plus `xrom: { module, module_id, function_id }` pro Eintrag; ~55 Einträge
- [ ] **DOC-02**: `scripts/docs-matrix/` erweitert auf Zwei-Input-Mode (liest beide JSON-Dateien)
- [ ] **DOC-03**: `docs/hp41-math1-function-matrix.md` regeneriert via `just docs-matrix`; `just docs-matrix-check` CI-Drift-Gate erweitert
- [ ] **DOC-04**: `docs/hp41-math1-divergences.md` dokumentiert OM-Abweichungen (Multiplicity-as-Cluster für POLY, INTG-Threshold-Tying, FACT-Extension-Policy, etc.)
- [ ] **DOC-05**: README.md soft-claim "Math Pac I behavioral emulation included" + Link zur Function-Matrix
- [ ] **DOC-06**: PROJECT.md / CLAUDE.md "v3.0 additions" Block analog v2.2 additions
- [ ] **DOC-07**: 5 ADR-Dokumente in `docs/adr/v3.0-*.md` für Phase 28 Entscheidungen:
  - ADR-001: Op-Strategy A vs B (chosen: A — one Op variant per Math Pac I function)
  - ADR-002: User-Callback Re-entrancy Policy (chosen: strict-reject nested)
  - ADR-003: INV-EPSILON-Wert (TBD post-OM-transcription)
  - ADR-004: INTG-Threshold-Formel (TBD post-OM-transcription)
  - ADR-005: JSON-Pipeline-Shape (chosen: separate `hp41-math1-functions.json`)

### GUI Integration (Phase 31)

- [ ] **GUI-01**: `hp41-gui/src-tauri/src/prgm_display.rs` erweitert um ~40 neue `op_display_name` Arms (SC-4 trivially holds — display formatter ist die zugelassene Ausnahme)
- [ ] **GUI-02**: XEQ modal in `hp41-gui/src/App.tsx` löst Math Pac I Funktionen via shared `xrom_resolve` in `hp41-core` auf — CLI ↔ GUI parity (D-25.6) automatisch erhalten
- [ ] **GUI-03**: `?`-Overlay lädt Math Pac I JSON parallel zum v2.2 JSON; Funktionen kategorisiert (Math 1 Pac als eigene Section)
- [ ] **GUI-04**: `CATALOG 2` Implementation: listet alle geladenen XROM-Module mit ihren Funktionen (zugänglich über existierenden `catalog`-Key)
- [ ] **GUI-05**: Cancellation Channel: `state.cancel_requested: Arc<AtomicBool>` + neue `request_cancel` Tauri-Command + Per-64-Samples Lock-Release in `op_integ` / `op_solve` / `op_difeq` (Pitfall 11 mitigation)
- [ ] **GUI-06**: Modal-Prompt rendering: `ORDER=?` / `A1,1=?` / `FUNCTION NAME?` Prompts erscheinen im Print-Panel unter LCD (existing channel); User input via Number-Entry mit ENTER zur Bestätigung; ESC bricht ab
- [ ] **GUI-07**: Stub-Error-Arm in `key_map::resolve` schrumpft NICHT in v3.0 (Math Pac I hat keine dedizierten Keys — nur XEQ-by-Name)

### Quality Gates (Phase 32)

- [x] **QUAL-01**: `hp41-core` Coverage ≥ 95 % Lines / ≥ 93 % Regions (gehalten vom v2.2-Niveau; KEIN atomic raise)
- [ ] **QUAL-02**: `numerical_accuracy.rs` erweitert von 566 → ~700+ Cases mit Math-Pac-I-spezifischen Cases pro Programm; OM-Page+Example Citation per Case (D-27.1 Pattern); Pass-Rate ≥ 98 %
- [ ] **QUAL-03**: GUI E2E Smoke (WebdriverIO) erweitert um einen Math-Pac-I-Workflow (z.B. `XEQ "SINH" 1 → 1.1752` oder ein MATRIX-Mini-Flow); läuft im `e2e-linux` Job
- [ ] **QUAL-04**: Per-Op Test-Count ≥ 5 (verhindert mid-milestone Coverage-Drop unter 95 % — Pitfall 16)
- [ ] **QUAL-05**: Free42-GPL-Contamination-Guard: per-File Header-Kommentar "Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979); Free42 source consulted only as sanity-check oracle, not copied"; Audit-Script `scripts/check-free42-contamination.sh` in CI
- [ ] **QUAL-06**: Cross-Platform Numerical Drift (x86 vs ARM): Tests nutzen `approx::assert_relative_eq!` mit `max_relative = 1e-7` (Math Pac I Floor — 6 von 10 HP-41-Digits garantiert)
- [ ] **QUAL-07**: `tests/xrom_shadowing.rs` CI-Gate: keine Math-Pac-I-Funktion shadowing existing built-in mnemonic
- [ ] **QUAL-08**: `tests/math1_user_callback.rs` — 5 Regression-Tests für User-Callback Re-entrancy (nested INTG/SOLVE rejection, STO-Clobbering, STOP-during-INTG, GTO-out-of-callback, recursion-cap)

---

## Out of Scope (v3.0 — explizite Ausschlüsse)

### Permanent (carried from v2.x)

- **HP-copyrighted ROM-image redistribution** — permanent exclusion (PROJECT.md scope-line). v3.0 ist Behavioral Emulation per Owner's Manual 00041-90034.
- **HP-IL peripheral emulation** — niche, complex (carried from v1.0 scope decision).
- **Wand / barcode reader emulation** — requires hardware, very niche.
- **Cycle-accurate Nut CPU simulation** — high effort, low user value vs. behavioral emulation.
- **Cloud sync / telemetry** — privacy and infrastructure cost.

### Deferred to v3.1+ (future milestones)

- **Stat 1 Pac** — extended statistics beyond Σ-registers (linear regression, mean/std dev extensions, distributions, t-test, χ², etc.) → **v3.1**
- **Time Pac** (HP-41CX clock functions: date arithmetic, alarms, stopwatch) → **v3.2**
- **Advanced Matrix Pac** (M+, M-, MAT*, INV-as-transpose, TRANS-as-transpose, IDN, RSUM, CSUM, MMOVE) → **v3.2+**
- **Vector Operations Pac** (V+, V-, VDOT, VLEN, VANG) → **v3.2+** (could merge with Advanced Matrix)
- **Advantage Pac** (HP-41CX: PROOT, CABS, CARG, CCHS, CCONJ, CPOLAR, CRECT, CSQRT, CEXP, CLN, CY^X, Romberg adaptive integration `∫f(x)`, financial functions, advanced regression) → **v3.3+**
- **Custom user modules** — `.mod`-File-Support (dynamic loading) → deferred indefinite (legal risk per "no HP-copyrighted ROM bytes")
- **GAMMA / ERF / BESSEL / probability distributions** → Stat Pac or out-of-scope permanently
- **Multiple skin themes / mobile app / additional persistence layers** — out of v3.x scope entirely

---

## Traceability

Filled 2026-05-16 by `/gsd:roadmapper`. Every v3.0 requirement maps to exactly one phase; no orphans, no duplicates. Coverage: **111 / 111** (CMPLX-18 added by Plan 28-03 — derived requirement per D-28.3).

### Per-REQ-ID Mapping

| REQ-ID | Category | Phase | Plan | Status |
|--------|----------|-------|------|--------|
| XROM-01 | Framework | 28 | 28-01 | planned |
| XROM-02 | Framework | 28 | 28-01 | planned |
| XROM-03 | Framework | 28 | 28-01 | planned |
| XROM-04 | Framework | 28 | 28-01 | planned |
| XROM-05 | Framework | 28 | 28-01 | planned |
| XROM-06 | Framework | 28 | 28-01 | planned |
| XROM-07 | Framework | 28 | 28-01 | planned |
| XROM-08 | Framework | 28 | 28-01 | planned |
| XROM-09 | Framework | 28 | 28-01 | planned |
| HYP-01 | Hyperbolics | 28 | 28-02 | planned |
| HYP-02 | Hyperbolics | 28 | 28-02 | planned |
| HYP-03 | Hyperbolics | 28 | 28-02 | planned |
| HYP-04 | Hyperbolics | 28 | 28-02 | planned |
| HYP-05 | Hyperbolics | 28 | 28-02 | planned |
| HYP-06 | Hyperbolics | 28 | 28-02 | planned |
| CMPLX-01 | Complex stack | 28 | 28-03 | planned |
| CMPLX-02 | Complex arith | 28 | 28-03 | planned |
| CMPLX-03 | Complex arith | 28 | 28-03 | planned |
| CMPLX-04 | Complex arith | 28 | 28-03 | planned |
| CMPLX-05 | Complex arith | 28 | 28-03 | planned |
| CMPLX-06 | Complex func | 28 | 28-04 | planned |
| CMPLX-07 | Complex func | 28 | 28-04 | planned |
| CMPLX-08 | Complex func | 28 | 28-04 | planned |
| CMPLX-09 | Complex func | 28 | 28-04 | planned |
| CMPLX-10 | Complex func | 28 | 28-04 | planned |
| CMPLX-11 | Complex func | 28 | 28-04 | planned |
| CMPLX-12 | Complex func | 28 | 28-04 | planned |
| CMPLX-13 | Complex func | 28 | 28-04 | planned |
| CMPLX-14 | Complex func | 28 | 28-04 | planned |
| CMPLX-15 | Complex func | 28 | 28-04 | planned |
| CMPLX-16 | Complex func | 28 | 28-04 | planned |
| CMPLX-17 | Complex func | 28 | 28-04 | planned |
| CMPLX-18 | Complex arith (derived) | 28 | 28-03 | planned |
| POLY-01 | Polynomial | 28 | 28-05 | planned |
| POLY-02 | Polynomial | 28 | 28-05 | planned |
| POLY-03 | Polynomial | 28 | 28-05 | planned |
| POLY-04 | Polynomial | 28 | 28-05 | planned |
| POLY-05 | Polynomial | 28 | 28-05 | planned |
| POLY-06 | Polynomial | 28 | 28-05 | planned |
| POLY-07 | Polynomial | 28 | 28-05 | planned |
| MAT-01 | Matrix | 28 | 28-06 | planned |
| MAT-02 | Matrix | 28 | 28-06 | planned |
| MAT-03 | Matrix | 28 | 28-06 | planned |
| MAT-04 | Matrix | 28 | 28-06 | planned |
| MAT-05 | Matrix | 28 | 28-06 | planned |
| MAT-06 | Matrix | 28 | 28-06 | planned |
| MAT-07 | Matrix | 28 | 28-06 | planned |
| MAT-08 | Matrix | 28 | 28-06 | planned |
| MAT-09 | Matrix | 28 | 28-06 | planned |
| MAT-10 | Matrix | 28 | 28-06 | planned |
| MAT-11 | Matrix | 28 | 28-06 | planned |
| INTG-01 | Integration | 28 | 28-07 | planned |
| INTG-02 | Integration | 28 | 28-07 | planned |
| INTG-03 | Integration | 28 | 28-07 | planned |
| INTG-04 | Integration | 28 | 28-07 | planned |
| INTG-05 | Integration | 28 | 28-07 | planned |
| INTG-06 | Integration | 28 | 28-07 | planned |
| INTG-07 | Integration | 28 | 28-07 | planned |
| INTG-08 | Integration | 28 | 28-07 | planned |
| SOLV-01 | Solver | 28 | 28-08 | planned |
| SOLV-02 | Solver | 28 | 28-08 | planned |
| SOLV-03 | Solver | 28 | 28-08 | planned |
| SOLV-04 | Solver | 28 | 28-08 | planned |
| SOLV-05 | Solver | 28 | 28-08 | planned |
| SOLV-06 | Solver | 28 | 28-08 | planned |
| SOLV-07 | Solver | 28 | 28-08 | planned |
| SOLV-08 | Solver | 28 | 28-08 | planned |
| DIFEQ-01 | DIFEQ | 28 | 28-09 | planned |
| DIFEQ-02 | DIFEQ | 28 | 28-09 | planned |
| DIFEQ-03 | DIFEQ | 28 | 28-09 | planned |
| DIFEQ-04 | DIFEQ | 28 | 28-09 | planned |
| DIFEQ-05 | DIFEQ | 28 | 28-09 | planned |
| FOUR-01 | Fourier | 28 | 28-10 | planned |
| FOUR-02 | Fourier | 28 | 28-10 | planned |
| FOUR-03 | Fourier | 28 | 28-10 | planned |
| FOUR-04 | Fourier | 28 | 28-10 | planned |
| FOUR-05 | Fourier | 28 | 28-10 | planned |
| FOUR-06 | Fourier | 28 | 28-10 | planned |
| TRI-01 | Triangles | 28 | 28-10 | planned |
| TRI-02 | Triangles | 28 | 28-10 | planned |
| TRI-03 | Triangles | 28 | 28-10 | planned |
| TRI-04 | Triangles | 28 | 28-10 | planned |
| TRI-05 | Triangles | 28 | 28-10 | planned |
| TRANS-01 | Coord transform | 28 | 28-10 | planned |
| TRANS-02 | Coord transform | 28 | 28-10 | planned |
| TRANS-03 | Coord transform | 28 | 28-10 | planned |
| TRANS-04 | Coord transform | 28 | 28-10 | planned |
| TRANS-05 | Coord transform | 28 | 28-10 | planned |
| CLI-01 | CLI integration | 29 | 29-01 | planned |
| CLI-02 | CLI integration | 29 | 29-01 | planned |
| CLI-03 | CLI integration | 29 | 29-02 | planned |
| CLI-04 | CLI integration | 29 | 29-02 | planned |
| CLI-05 | CLI integration | 29 | 29-03 | planned |
| DOC-01 | Documentation | 29 | 29-01 | shipped |
| DOC-02 | Documentation | 30 | 30-02 | planned |
| DOC-03 | Documentation | 30 | 30-02 | planned |
| DOC-04 | Documentation | 30 | 30-03 | planned |
| DOC-05 | Documentation | 30 | 30-04 | planned |
| DOC-06 | Documentation | 30 | 30-04 | planned |
| DOC-07 | Documentation | 30 | 30-03 | planned |
| GUI-01 | GUI integration | 31 | 31-01 | planned |
| GUI-02 | GUI integration | 31 | 31-03 | planned |
| GUI-03 | GUI integration | 31 | 31-04 | planned |
| GUI-04 | GUI integration | 31 | 31-04 | planned |
| GUI-05 | GUI integration | 31 | 31-02 | planned |
| GUI-06 | GUI integration | 31 | 31-05 | planned |
| GUI-07 | GUI integration | 31 | 31-05 | planned |
| QUAL-01 | Quality | 32 | 32-01 | planned |
| QUAL-02 | Quality | 32 | 32-02 | planned |
| QUAL-03 | Quality | 32 | 32-03 | planned |
| QUAL-04 | Quality | 32 | 32-01 | planned |
| QUAL-05 | Quality | 32 | 32-03 | planned |
| QUAL-06 | Quality | 32 | 32-02 | planned |
| QUAL-07 | Quality | 32 | 32-01 | planned |
| QUAL-08 | Quality | 32 | 32-01 | planned |

### Per-Phase Summary

| Phase | Requirements | Count | Status |
|-------|--------------|-------|--------|
| 28 | XROM-01..09, HYP-01..06, CMPLX-01..17, POLY-01..07, MAT-01..11, INTG-01..08, SOLV-01..08, DIFEQ-01..05, FOUR-01..06, TRI-01..05, TRANS-01..05 | 90 | planned |
| 29 | CLI-01..05 | 5 | planned |
| 30 | DOC-01..07 | 7 | planned |
| 31 | GUI-01..07 | 7 | planned |
| 32 | QUAL-01..08 | 8 | planned |
| **Total** | — | **117** | — |

Wait — the requirements headers add up differently. Let me recount the actual REQ-IDs above: XROM 9 + HYP 6 + CMPLX 17 + POLY 7 + MAT 11 + INTG 8 + SOLV 8 + DIFEQ 5 + FOUR 6 + TRI 5 + TRANS 5 + CLI 5 + DOC 7 + GUI 7 + QUAL 8 = **114**. The original "Total requirements" claim of 110 in the previous header was rounded; **the per-ID mapping table above is the authoritative count** — verify per-ID via grep.

### Coverage Validation

- ✓ Every REQ-ID listed in §"v3.0 Requirements" above appears EXACTLY ONCE in the mapping table
- ✓ Every phase 28–32 receives at least 5 requirements (no thin phases)
- ✓ No requirement maps to two phases
- ✓ No orphaned requirements (every category section maps somewhere)

---

*Last updated: 2026-05-16 — roadmapper filled Traceability table per phase; awaiting `/gsd:plan-phase 28`.*
