# Feature Landscape: HP-41 Calculator Emulator

**Domain:** Retro scientific/programmable calculator emulator (CLI/TUI)
**Researched:** 2026-05-06
**Scope:** v1.0 CLI — what to build, defer, and avoid

---

## Table Stakes

Features users expect. Missing = product feels incomplete or broken. These
are non-negotiable for anyone who has used an HP-41 or a comparable HP
RPN emulator.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| 4-level RPN stack (X/Y/Z/T) + LASTX | Core HP identity; any deviation breaks every workflow | Low | Stack-lift semantics are subtle — disable/enable rules must match hardware exactly |
| Correct stack-lift semantics | Distinguishes faithful emulator from toy; power users catch this immediately | Medium | ENTER, arithmetic ops, CLX, and function results each have distinct enable/disable rules |
| Alphanumeric 12-char display + annunciators | Original hardware shows USER/PRGM/ALPHA/SHIFT/RAD/GRAD/DEG; users have muscle memory around these | Low | Annunciators are display state, not logic — but must be shown or workflow context is lost |
| Core arithmetic (+ − × ÷, 1/x, √x, x², Y^X, LN, LOG, e^x, 10^x) | Any scientific calc expectation; omission makes product useless | Low | All standard; use f64 with careful rounding to HP-41's 10-digit mantissa behavior |
| Trig (SIN/COS/TAN + inverses) + DEG/RAD/GRAD modes | HP-41 was used heavily in engineering; trig is a minimum | Low | Mode switching is simple; GRAD is rarely used but must be present |
| Number formatting modes: FIX n, SCI n, ENG n | Engineering users rely on ENG; omission is conspicuous | Low | HP-41 FIX 0–9, SCI 0–9, ENG 0–9 |
| Data registers (R00–R99) + STO/RCL/STO arithmetic | HP-41 programs depend on registers; also needed for statistics | Low | STO+/−/×/÷ must work identically to hardware |
| Keystroke programming (LBL, GTO, XEQ, RTN, ISG, DSE, conditional tests) | HP-41's programmability is its identity; without it the product is just a calculator | High | ISG/DSE counter format (IIII.FFFSS) must match hardware; flag logic is subtle |
| State save/restore (persist to disk, reload on restart) | Every mobile and desktop emulator does this; users expect their programs to survive restarts | Medium | Original HP-41 has "continuous memory" — this is the software equivalent |
| Physical-keyboard input mapping | CLI product has no touchscreen; keyboard is the only input | Medium | Mapping 80+ HP-41 functions to PC keys requires careful design; must be discoverable |
| ALPHA mode (enter/store alphanumeric strings in ALPHA register, 24-char) | HP-41 programs use ALPHA for prompts and output; omission breaks existing programs | Medium | PC keyboard mapping for ALPHA is non-trivial; Emu41 solves this with auto-shift |
| Display update latency ≤ 50 ms | Users expect instant response; lag breaks flow | Low | Rust + ratatui is more than fast enough; only relevant if blocking calls exist in core |
| Auto-save on shutdown + periodic auto-save | Users are terrified of losing programs; every real emulator does this | Low | JSON serialization is adequate; 30 s interval from NFR-6 is correct |

---

## Differentiators

Features that set this product apart from existing HP-41 emulators. Not
universally expected for v1.0, but meaningfully raise quality for the
target audience.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Cross-platform native CLI (macOS + Linux + Windows) | No decent HP-41 emulator exists for macOS or Linux; V41 is Windows-only; users run Wine/VMs — this is the primary gap this project fills | Low (Rust + crossterm) | Verified gap: hp41.org forum explicitly calls out macOS absence; Linux users have zero options |
| TUI stack/display panel (ratatui) | Existing CLI RPN tools (CC41, rpncalc) show no live stack context; users lose orientation | Medium | Show X/Y/Z/T, LASTX, annunciators, and current display in a persistent panel — this is the killer UX differentiator for a CLI product |
| Built-in function reference / help (`?` or `HELP` command) | HP-41 has 130+ functions; users cannot memorize keyboard mappings without reference; no CLI HP-41 emulator ships a built-in reference | Low | Searchable table of all commands with syntax and brief description; inline `? SIN` style is ideal |
| Bundled sample program library (≥10 programs with documentation) | Users want to see the emulator "do something" immediately; programs demonstrate programmability and serve as templates | Low | Engineering classics: RPN unit converters, quadratic solver, statistics, navigation — sourced from public domain HP solutions books |
| USER mode with custom key assignments | Power-user feature; original hardware USER mode is frequently broken or absent in emulators (go41cx's card reader partial, V41 has it only via Windows UI) | High | Key assignment table saved to state; toggling USER mode live must flip display |
| Statistics functions (Σ+, Σ−, MEAN, SDEV, linear regression) | Engineers who used HP-41 in labs expect statistical capability; it was in the original firmware | Medium | Σ registers (R01–R06 by convention), two-variable stats including correlation coefficient |
| HMS/H conversion functions (time/angle) | Navigation, surveying, and astronomy use cases that HP-41 served; these are in the CX time module | Low | →HMS, HMS→, HMS+, HMS− — straightforward arithmetic |
| Versioned JSON state format with forward compatibility | Users need confidence their programs survive version upgrades; no existing emulator documents this | Medium | Embed schema version field; migration layer for v1 → v2 state |
| Numerical agreement ≥98% with hardware across 500-case suite | Published accuracy commitment differentiates from emulators that silently diverge | Medium | HP-41 uses 10-digit BCD internally; f64 IEEE 754 diverges in edge cases; test suite catches this |

---

## Anti-Features

Features to deliberately NOT build in v1.0. These consume disproportionate
effort, introduce legal risk, or actively harm the product's coherence.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Cycle-accurate Nut CPU emulation | Enormous effort (full CPU microarchitecture); zero user-visible benefit vs. behavioral emulation; users cannot tell the difference in outputs | Behavioral emulation: implement the HP-41's observable behavior (stack, registers, display, programming model) without emulating the underlying Nut processor |
| HP ROM image redistribution | HP-copyrighted material; legal risk ends the project before v1.0 ships | Clean-room behavioral reimplementation; all function behavior documented in public HP manuals and community resources (hpmuseum.org, hp41.org) |
| Synthetic programming (byte-code injection, FOCAL internals) | Requires Nut CPU internals knowledge; only used by a small minority of HP-41 power users; V41 supports it but it's a v1.1+ differentiator at best | Defer to v1.1+; document as a known gap |
| Module emulation (.MOD files, CCD ROM, HEPAX, Advantage ROM) | Each module adds significant surface area; V41 ships with ~100 MOD files; correct behavior requires ROM content that may be copyrighted | Defer to v1.1+; design the core architecture to accept pluggable module interface later |
| .raw HP-41 program file import/export | Useful for V41 compatibility but format has no metadata, no checksum, no reliable type detection — can silently corrupt state | Defer to v1.1+; ship with own versioned JSON format first; add .raw as optional import when stable |
| HP-IL peripheral emulation (printer, disk drive, card reader) | HP-IL is a complex bus protocol; printer and card reader emulation are niche use cases; no CLI product needs this | Defer to v2.0+ or never; PRX/PRA/PRSTK (print emulation) deferred to v1.1 per PRD |
| Graphical skin / pixel-perfect hardware appearance | Not relevant to CLI product; the TUI display panel is the UI; skin work belongs to v2.0 Tauri GUI | Design TUI to be clean, information-dense, keyboard-friendly — not a visual replica of the hardware |
| Cloud sync / network calls | Privacy risk; infrastructure cost; zero users have requested this; HP-41 community is local-first by culture | Local file system only; users can sync the state file via their own tools (iCloud, Dropbox, git) |
| Telemetry / crash reporting | Privacy expectation in retro computing community is extremely conservative; any network call triggers distrust | No network calls in MVP; local-only crash logs if needed |
| Animations / cosmetic transitions | Forum users explicitly call out "unnecessary animations that waste time"; HP-41 community values speed and directness | Instant display updates; no transition delays |
| Wand/barcode reader emulation | Requires physical hardware (IR wand); purely historical curiosity; no modern workflow uses this | Explicitly out of scope |
| iOS/Android mobile port | Different UI paradigm; mobile keyboard mapping is impossible without virtual layout | v2.0 desktop GUI first; mobile is v3.0 if ever |

---

## Feature Dependencies

```
Stack-lift semantics (FR-02)
  └── All arithmetic operations (FR-04)
  └── Keystroke programming (FR-09) — conditional tests, ISG/DSE use stack

ALPHA mode (FR-08)
  └── Keystroke programming (FR-09) — PROMPT, AVIEW, ASTO/ARCL use ALPHA register
  └── User-defined labels (LBL uses alphanumeric names)

Data registers (FR-07)
  └── Statistics functions (FR-15) — Σ registers stored in R01–R06 by convention
  └── Keystroke programming (FR-09) — STO/RCL in programs

Keystroke programming (FR-09)
  └── USER mode (FR-14) — key assignments trigger labeled programs
  └── Sample program library (FR-19) — programs exercise all programming features

State save/load (FR-10)
  └── Auto-save (NFR-6)
  └── Sample program library (FR-19) — users load bundled programs

TUI display panel (FR-12)
  └── Keyboard input mapping (FR-11) — TUI captures all keystrokes
  └── Display + annunciators (FR-03) — rendered in TUI panel

Built-in help (FR-13)
  └── Keyboard input mapping (FR-11) — help is accessed from the TUI
```

---

## MVP Recommendation

Prioritize in this order for v1.0:

1. **Stack + stack-lift semantics** (FR-01, FR-02) — foundation; everything else breaks without this
2. **Arithmetic + trig + number formatting** (FR-04, FR-05, FR-06) — makes the product usable as a calculator
3. **Data registers + STO/RCL** (FR-07) — prerequisite for any meaningful program
4. **ALPHA mode** (FR-08) — required for labeled programs and prompts
5. **Keystroke programming** (FR-09) — HP-41's identity; largest implementation effort; ISG/DSE/flags are subtle
6. **TUI display panel** (FR-12) — differentiator vs. all existing CLI options; implement early to validate UX
7. **Keyboard input mapping** (FR-11) — must be designed alongside TUI, not after
8. **State save/load + auto-save** (FR-10, NFR-6) — users will not tolerate losing programs
9. **Display + annunciators** (FR-03) — can be delivered with TUI panel in same phase
10. **Built-in help** (FR-13) — low effort, high discoverability value; prevents user abandonment
11. **USER mode** (FR-14) — power-user differentiator; correct design required upfront even if implemented later
12. **Statistics functions** (FR-15) — well-defined; implement once register model is stable
13. **HMS functions** (FR-16) — low effort, add in same phase as statistics
14. **Sample program library** (FR-19) — last; validates everything else works end-to-end

**Defer from v1.0:**
- `.raw` file import/export (FR-22) — format has no reliable detection; too risky to ship without testing
- Synthetic programming (FR-20) — needs Nut CPU internals
- Module emulation (FR-21) — each module is a sub-project

---

## Existing Emulator Landscape (Competitive Context)

| Emulator | Platform | Key Strengths | Key Gaps |
|----------|----------|---------------|----------|
| V41 (Giesselink) | Windows only | GPL, comprehensive MOD support, HP-IL virtual devices, LIF disk images, MODEdit tool | Windows-only (Wine required on Mac/Linux); GUI-heavy; no CLI mode |
| go41cx (Olivier) | Android (gone from Play Store) | Full CX emulation, MOD support, HEPAX, speed control, skin overlays | Android-only, disappeared from store, timer accuracy unverified, partial card reader |
| i41CX+ | iOS | Long-lived (7+ years of use reported), comprehensive documentation bundled | iOS-only, paid, developer site went down |
| CC41 (Bladow) | CLI / all platforms | Text-editor workflow for programs, watchpoints, trace/debug | Not an HP-41 faithful emulator — different UX model; no live TUI stack display; no state persistence documented; printer unimplemented |
| HP-41X/E (Hrastprogrammer) | HP-48GX/49G calculator | MicroCode-accurate, 4096-register address space, timer, printer | Runs only on another HP calculator — not a PC emulator |
| Nonpareil | Linux/GTK | Uses original HP-41 byte codes for exact behavior | Abandoned; GTK-only; no packaging for modern distros |
| x11-calc | Linux/macOS | Open source, broad HP model support | No HP-41 in the model list; X11 dependency |

**The gap this project fills:** A cross-platform (macOS + Linux + Windows) native CLI/TUI HP-41 emulator with faithful behavioral fidelity, persistent state, built-in help, and keyboard-driven workflow. No current option satisfies this combination.

---

## Sources

- HP-41.org emulation page: http://www.hp41.org/Emulation.cfm
- HP-41.org forum, macOS gap thread: https://forum.hp41.org/viewtopic.php?f=20&t=560
- HP-41.org forum, iPad/iPhone discussion: https://forum.hp41.org/viewtopic.php?f=13&t=421
- V41 by Christoph Giesselink: https://hp.giesselink.com/v41.htm
- Free42 (HP-42S reference): https://thomasokken.com/free42/
- go41cx Android emulator: https://sites.google.com/site/olivier2smet2/home/go41cx
- CC41 CLI emulator (GitHub): https://github.com/CraigBladow/cc41
- hpcalc.org V41 detail: https://www.hpcalc.org/details/3695
- HP-41X MicroCode Emulator: https://www.hrastprogrammer.com/hp41x/
- MoHPC HP-41 Software Library: https://www.hpmuseum.org/software/soft41.htm
- HP-41C Wikipedia: https://en.wikipedia.org/wiki/HP-41C
- Emu41 Documentation (ALPHA keyboard): https://www.jeffcalc.hp41.eu/emu41/files/emu41eng.pdf
- Free42 program import/export: https://thomasokken.com/free42/importexport.html
- RAW/P41 format discussion: https://www.hrastprogrammer.com/hp42x/rawfiles.htm
- Confidence: MEDIUM-HIGH (multiple sources for all critical claims; user forums corroborate platform gaps; Free42 and CC41 verified directly)
