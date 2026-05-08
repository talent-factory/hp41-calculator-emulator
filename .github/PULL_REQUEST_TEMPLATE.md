## Description

<!-- Briefly describe what this PR does and why. -->

## Related Issue

Closes #<!-- issue number -->

## Type of Change

- [ ] Bug fix
- [ ] New feature / operation
- [ ] Refactor / cleanup
- [ ] Documentation
- [ ] CI / tooling

## Checklist

- [ ] `just ci` passes locally (lint → test → coverage ≥80%)
- [ ] New operations declare stack-lift behaviour (Enable / Disable / Neutral)
- [ ] No panics introduced in `hp41-core`
- [ ] `hp41-core` does not import from `hp41-cli` or `hp41-gui`
- [ ] Snapshot tests updated if output changed (`cargo insta review`)
