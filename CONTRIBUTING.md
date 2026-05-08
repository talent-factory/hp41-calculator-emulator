# Contributing to hp41-calculator-emulator

Thank you for your interest in contributing!

## Branch Model

| Branch    | Purpose                          | Who can push directly |
|-----------|----------------------------------|-----------------------|
| `main`    | Stable releases                  | Maintainer only       |
| `develop` | Integration / next release       | Maintainer only       |
| `feat/*`  | Feature branches                 | Anyone (via fork)     |
| `fix/*`   | Bug-fix branches                 | Anyone (via fork)     |

**All external contributions must go through a Pull Request targeting `develop`.**  
Direct pushes to `develop` and `main` are blocked by branch protection for everyone except the maintainer.

## Workflow

```
fork → feat/your-feature → PR → develop → (release) → main
```

1. **Fork** the repository.
2. Create a branch: `git switch -c feat/my-feature`.
3. Make your changes — see Quality Gates below.
4. Push to your fork and open a PR targeting `develop`.
5. CI must pass and the maintainer must approve before merging.

## Quality Gates (must pass locally before opening a PR)

```bash
just ci          # lint → test → coverage ≥80%
```

Individual steps:

```bash
just lint        # cargo clippy -D warnings + rustfmt check
just test        # cargo test --workspace
just coverage    # cargo llvm-cov --fail-under-lines 80 -p hp41-core
```

Install prerequisites:

```bash
cargo install just cargo-llvm-cov
```

## HP-41 Fidelity Rules

- **≥98% numerical agreement** vs. real HP-41 hardware (500-case test suite).
- Every new operation **must** declare its stack-lift flag: `Enable`, `Disable`, or `Neutral`.
- ISG/DSE counter fields are extracted by **string-splitting at the decimal point** — never with `floor()`/`fmod()` on f64.
- `hp41-core` must never import from `hp41-cli` or `hp41-gui`.

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/) with emoji prefixes:

```
✨ feat(core): add ASIN operation
🐛 fix(cli): correct cursor wrapping on last menu row
📚 docs(contributing): add ISG/DSE rule
```

## Code of Conduct

This project follows the [Contributor Covenant v2.1](CODE_OF_CONDUCT.md).

## Questions?

Open a [Discussion](https://github.com/talent-factory/hp41-calculator-emulator/discussions) — issues are reserved for bugs and feature requests.
