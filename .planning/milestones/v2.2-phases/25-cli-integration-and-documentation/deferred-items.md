# Phase 25 Deferred Items

Out-of-scope discoveries surfaced during Phase 25 execution that are NOT
addressed in any of the four plans. Logged per `execute-plan.md` scope-boundary
rules so a future phase can pick them up.

## Pre-existing rustdoc warnings in `hp41-core`

**Discovered during:** Plan 25-04 Task 3 verification step (`cargo doc --no-deps -p hp41-core`).

**Out of scope because:** Phase 25 has a HARD invariant of "ZERO `hp41-core`
changes" (CLAUDE.md / 25-CONTEXT.md domain). The 10 `unresolved link to ...`
warnings predate Plan 25-04 and live in `hp41-core/src/ops/alpha.rs`,
`hp41-core/src/state.rs`, and a handful of other Phase 21–24 doc-comments.
They are doc-comment uses of `chars[0]` / `regs[5]` / `[0..=99]`-style
code references that rustdoc interprets as intra-doc links. They are NOT
broken links to actual items — purely cosmetic.

**Suggested fix (Phase 27 territory):** wrap the offending `[N]` /
`[0..=99]` snippets in backticks or escape the brackets with `\[` / `\]`
per rustdoc's suggested fix. ≤ 30 minutes of mechanical edits across
~10 doc-comments.

**Tracking:**

```text
warning: unresolved link to `0`         hp41-core/src/ops/alpha.rs:144:27
warning: unresolved link to `pc`        (similar position)
warning: unresolved link to `n`         (×2)
warning: unresolved link to `1` .. `6`  (register-index references)
```

The full list reproduces via `cargo doc --no-deps -p hp41-core 2>&1 |
grep "unresolved link"`.
