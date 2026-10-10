# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v4.2 — Help Search Enrichment

**Shipped:** 2026-06-05
**Phases:** 4 (58–61) | **Plans:** 11 | **Sessions:** ~3

### What Was Built
- Invisible `search_aliases` match surface on both help-entry mirrors (Rust + TS), serde-default back-compat, populated across all six JSON pools (364 implemented entries, DE+EN).
- Alias-aware tiered matcher (exact > prefix > substring > fuzzy) with a hand-rolled bounded Levenshtein — zero new runtime deps — mirrored CLI↔GUI; empty query keeps the category view, active query switches to a ranked flat list.
- Offline `scripts/help-aliases/` generator + LLM runner producing a committed alias-only diff (no runtime ML).
- Quality layer: Rust 10 + TS 36 real-data tests, a CLI↔GUI parity fixture (top-1 drift guard), and a six-pool `search_aliases` schema CI gate wired into `ci.yml`.

### What Worked
- **Wave-based parallel execution (Phase 61)** — Wave 2's three disjoint-file plans (Rust tests / TS tests / schema gate) ran concurrently, then a single re-run of the full suite from final HEAD caught any integration issue. Disjoint `files_modified` kept conflicts to git-index timing only.
- **Drift-guard-by-fixture** — one committed `help_search_parity.json` asserted by BOTH frontends turned "CLI and GUI must agree" from a hope into a CI-enforced invariant (HSQUAL-02).
- **Schema gate that bites** — the `--self-test` flag in `check-aliases-schema.sh` proved the gate rejects an emptied alias list, so the gate itself is trustworthy.
- **Zero-dep discipline held** — fuzzy matching hand-rolled rather than pulling a crate, consistent with the since-v3.0 no-new-deps invariant.

### What Was Inefficient
- **Parallel shared-index git contention** — with `branching_strategy: none` + `parallelization: true`, Wave 2's three executors raced on the single git index; each had to recover via pathspec-scoped / reset-soft commits. No work lost, but it cost retries. (Worktree isolation was deliberately avoided here — it branches from origin/main and would miss develop-only predecessor work.)
- **Phase 60 Wave-2 writeback defect** — the alias generator's first writeback pass was buggy; caught only by human review before commit. A pre-commit alias-only-diff assertion would have caught it automatically.
- **Bookkeeping drift surfaced only at close** — the requirements traceability sat at "Pending" for all 18 reqs, Phases 60 & 61 produced no VERIFICATION.md, and Phase 59's frontmatter stayed `human_needed` after its UAT passed. All cleaned up at milestone close, but the milestone audit had to reconstruct the truth from three sources.

### Patterns Established
- **Invisible data field + DE-in-search exception** — `search_aliases` is search *input* vocabulary (never rendered), the one sanctioned place German lives in committed data; documented as a carve-out from the English-only rule.
- **Mirrored matcher + shared parity fixture** — the established CLI↔GUI duplication pattern (cf. `op_display_name`) now has a fixture-based drift guard as its load-bearing safety net.
- **Six-pool schema gate, never globbed** — pools enumerated explicitly because `docs/hp41cv-functions.json` has no dash before "functions"; the gate lives in `ci.yml` (not `ci-gui.yml`, whose path filter excludes `docs/*.json`).

### Key Lessons
1. **`branching_strategy: none` + parallel waves → commit with explicit pathspec** (`git commit -- <files>`) and verify per-commit file lists at the post-merge gate; agent green-runs mid-race are not authoritative.
2. **Verification-shaped phases don't self-document** — a data-generation phase (60) and a verification phase (61, whose deliverables ARE the gates) naturally produce no VERIFICATION.md, leaving bookkeeping debt; backfill or accept explicitly at audit time.
3. **Milestone release mechanics are squash-fatal** — the develop→main milestone PR must merge with `--merge` (merge commit) so the `vX.Y` tag stays reachable from main and `release.yml` auto-publishes; a squash silently suppressed the v4.0 Release.
4. **Tick the traceability as phases land** — leaving all rows "Pending" forces the milestone audit to reconstruct status from VERIFICATION + SUMMARY + integration evidence.

### Cost Observations
- Model mix: orchestration + audit + close on Opus; phase executors and the integration checker on Sonnet.
- Sessions: ~3 (execute Phase 61 → ship → audit + complete-milestone).
- Notable: the integration checker (read-only, Sonnet) confirmed all 18 requirements wired end-to-end without re-running suites — cheap, high-signal verification.

---

## Milestone: v4.3 — Hardware Fidelity

**Shipped:** 2026-06-10
**Phases:** 6 (62–67) | **Plans:** 26 | **Sessions:** ~6

### What Was Built
- A synchronous **run-loop yield/interrupt engine** at the `run_loop` boundary (no threads, no new `Op`): interrupting control alarms execute mid-run and resume the host program (D-40-04 closed), and PSE/VIEW/AVIEW yield mid-run. Side effect: the **Tauri GUI gained its first continuous program run loop** (`run_program`/`resume_program` + a no-poll TS yield driver, D-11).
- **Interactive GETKEY** built on the same primitive (`WaitForKey` yield + `resume_program_with_key`, row×col→X), CLI↔GUI parity.
- **Standalone fidelity fixes**: two-tier `HpNum{mantissa,exponent}` over the full ±9.999999999E±99 range (`FACT(27..69)`, ADR v4.3-005), 12-cell LCD overflow render (v4.3-006), CLI `display_override`, CHS in-buffer sign-flip, AON auto-display.
- **Reset Escape Hatch** (the post-completion reopen): two-tier `soft_reset()`/`memory_lost()` **outside** the dispatch path so it works even when input is trapped; CLI `Ctrl+R`, GUI/iOS ON-key tap/long-press; autosave overwritten under-mutex so recovery survives restart (ADR v4.3-007).

### What Worked
- **Sequential-on-develop execution (no worktrees)** — every phase's plan files were local-develop-only, and `isolation:worktree` branches from `origin/main`, so a worktree executor couldn't read its own PLAN.md. Running executors directly on the develop tree avoided the base-ref-stale trap that bit earlier milestones.
- **In-cycle code review caught real bugs** — CR-01 (Phase 64: missing guard let a racing resume clobber a live PSE/VIEW yield) and CR-02 (Phase 67: `cancel_requested` Arc swapped on reset → GUI cancellation silently broken forever) were both found *and fixed within the phase*, with regression tests. The shared-Arc orphan was invisible to JSON-equality tests (serde-skip).
- **A dedicated verification phase (66)** folded UNC-01/02/03 verification + divergence-doc sweep + all quality gates into one gate, so the milestone closed with green CI rather than a trailing audit.

### What Was Inefficient
- **The release was tagged too early.** The `v4.3` tag was first cut at the Phase-66 commit; the milestone was then **reopened for Phase 67**, but the tag (and the GitHub Release the binary workflows auto-publish on tag-push *from develop*) were never moved — so the published Release served Windows/Linux installers **without Phase 67**. Recovery cost a tag force-move + a rerun of both binary workflows.
- **A GitHub API-Requests major outage** landed mid-release, 401-ing the first asset reruns and leaving a partial set — required polling githubstatus at the *component* level (not the global indicator) before the reruns could complete.
- **Quick-task cruft accumulated** — 13 v4.1-era quick-tasks with missing STATUS frontmatter resurfaced in the close audit; they should have been swept at their own milestone close.

### Patterns Established
- **One yield primitive, many features** — alarms, PSE/VIEW/AVIEW, and GETKEY all ride the same `pending_yield`/resume mechanism rather than bespoke paths.
- **Escape hatches live outside dispatch** — a recovery action must not route through the very machinery that may be stuck; reset is deliberately *not* an `Op` (no 4-way match).
- **Never reassign an Arc shared with a long-lived clone** — clear it in place (`store(false)`); a fresh-Arc swap orphans every existing clone (e.g. the GUI's startup-cloned cancel flag).

### Key Lessons
- **Do not create the milestone tag until the milestone is actually closed.** Reopening a milestone after tagging invalidates the tag and silently ships stale release assets. If you must reopen: move the tag to the new HEAD **and rerun both binary workflows**, then verify every asset `updated_at` is post-move.
- **The GitHub Release is published on tag-push from develop, before the develop→main merge** — the binary workflows are not gated on main-reachability (only `release.yml`'s belt-and-suspenders publish job is). The merge does not fix stale assets; the workflow rerun does.
- **During an outage, gate on the specific status-page component** (`API Requests`), not the global indicator — an unrelated "Issues: degraded" pins the global indicator red long after writes recover.

### Cost Observations
- Milestone closed + released in a single working session; the bulk of *this* session's effort was release recovery (stale-tag diagnosis, outage wait, asset rebuild), not the close itself.
- Model: opus throughout; long poll-waits were backgrounded rather than burning interactive turns.

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v4.2 | ~3 | 4 | First milestone closed with a formal cross-phase integration audit before archive; surfaced parallel-wave git-index contention as a recurring cost of `branching_strategy: none`. |

### Cumulative Quality

| Milestone | Tests added | Coverage | Zero-Dep Additions |
|-----------|-------------|----------|--------------------|
| v4.2 | +46 (Rust 10 + TS 36) | hp41-core unchanged (UI/help-only) | hand-rolled fuzzy matcher (0 new crates) |
