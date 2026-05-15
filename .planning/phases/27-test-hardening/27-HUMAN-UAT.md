---
status: partial
phase: 27-test-hardening
source: [27-VERIFICATION.md]
started: 2026-05-15T14:15:00Z
updated: 2026-05-15T14:15:00Z
---

## Current Test

[awaiting human observation of GitHub Ubuntu CI run]

## Tests

### 1. e2e-linux job green on Ubuntu CI
expected: `GUI E2E (Ubuntu only — WebdriverIO + tauri-driver) / e2e-linux` workflow exits 0; cold ~6–8 min, cached ~3–5 min; spec reporter prints `HP-41 GUI smoke (FN-QUAL-05, D-27.13 literal ROADMAP scope) ✓ 2 ENTER 3 + displays 5.0000`; no `WebKitWebDriver not found`; no Xvfb display error
result: [pending — first push triggers the new job]

### 2. Branch protection for e2e-linux as required-for-merge
expected: GitHub repo Settings → Branches → Branch protection rules for `develop` and `main` add `GUI E2E (Ubuntu only — WebdriverIO + tauri-driver) / e2e-linux` as a required status check, so PRs can no longer merge without it green
result: [pending — manual repo setting outside YAML/code; tracked in 27-04-SUMMARY.md line 244]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
