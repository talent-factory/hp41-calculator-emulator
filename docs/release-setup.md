# Release infrastructure — secrets + workflow setup

This file documents the GitHub Actions workflows and repository secrets that
power binary release publication. The workflows fire automatically on every
`v*` tag push; this document is for the maintainer who needs to (re-)configure
the secrets after a key rotation or onboarding setup.

## Workflow inventory

| Workflow | Trigger | What it does |
|----------|---------|--------------|
| `.github/workflows/release.yml` | tag push + push to main | Creates the GitHub Release from the annotated tag's message |
| `.github/workflows/release-cli-binaries.yml` | tag push + manual | Builds `hp41-cli` for Linux (x86_64), macOS (arm64 + x86_64), Windows (x86_64); signs + notarizes macOS; uploads tarballs/zip to the Release |
| `.github/workflows/release-gui-binaries.yml` | tag push + manual | Builds `hp41-gui` via `tauri-apps/tauri-action`: macOS universal .dmg/.app, Windows .msi + portable .exe, Linux .deb + .AppImage; signs + notarizes macOS; uploads to the Release |

All three workflows share the same `tags: ['v[0-9]+.[0-9]+*']` trigger pattern,
so a single `git push origin v3.0.1` fires the complete pipeline. Each
workflow is idempotent — re-running via `workflow_dispatch` with an explicit
`tag` input is safe.

## Required GitHub Secrets

The macOS signing + notarization steps in BOTH binary workflows read the same
six secrets. Without them, the workflows still ship — but the macOS binaries
will be unsigned and trigger a Gatekeeper warning on first launch
("hp41-cli cannot be opened because the developer cannot be verified").

| Secret | Purpose | Value source |
|--------|---------|--------------|
| `APPLE_CERTIFICATE` | base64-encoded Developer ID Application .p12 | Export from Keychain Access → File → Export Items → .p12 → `base64 -i cert.p12 \| pbcopy` |
| `APPLE_CERTIFICATE_PASSWORD` | Password used during the .p12 export | Set during the export above |
| `APPLE_SIGNING_IDENTITY` | The `Developer ID Application: ...` string | `security find-identity -v -p codesigning` on a Mac with the cert installed |
| `APPLE_ID` | Apple ID email for notarization | The email you log into developer.apple.com with |
| `APPLE_PASSWORD` | App-specific password (NOT your Apple ID password) | https://appleid.apple.com → Sign-In and Security → App-Specific Passwords |
| `APPLE_TEAM_ID` | 10-character team identifier | https://developer.apple.com/account → top-right team name |

### Setting the secrets

```sh
# One-time setup — replace <…> with real values.
gh secret set APPLE_CERTIFICATE          -b "$(base64 -i ~/Downloads/cert.p12)"
gh secret set APPLE_CERTIFICATE_PASSWORD -b '<p12 export password>'
gh secret set APPLE_SIGNING_IDENTITY     -b 'Developer ID Application: Daniel Senften (XXXXXXXXXX)'
gh secret set APPLE_ID                   -b 'daniel.senften@talent-factory.ch'
gh secret set APPLE_PASSWORD             -b '<app-specific password>'
gh secret set APPLE_TEAM_ID              -b 'XXXXXXXXXX'
```

After setting, verify with `gh secret list` — all six should be present.

## Platform-specific notes

### macOS

- The CLI binary is signed with `codesign --options runtime --timestamp` and
  notarized via `xcrun notarytool submit ... --wait`. Standalone executables
  cannot be `stapler stapled`, but the notarization ticket is checked online
  by Gatekeeper on first launch.
- The GUI is built as a **universal binary** (`--target universal-apple-darwin`),
  signed by tauri-action's built-in signing flow, and notarized as a .app
  bundle (stapled-stapleable).
- macOS 12 (Monterey) or later is required for the universal binary to run.

### Windows

- **No Authenticode signing** — the project does not own a Windows code-signing
  certificate. Users will see a SmartScreen warning on first launch:
  > "Windows protected your PC" → "More info" → "Run anyway".
  This is the standard hobbyist-distribution experience.
- The CLI ships as a `.zip` containing `hp41-cli.exe`; the GUI ships as
  both `.msi` (installer) and `.exe` (portable).

### Linux

- No signing required. `.AppImage` is universal across distros; `.deb` works on
  Debian/Ubuntu derivatives.
- The CLI builds against `x86_64-unknown-linux-gnu` (glibc); systems with glibc
  too old (< 2.34) need to build from source.

## Release procedure

1. Land all v3.0.1 (or later) changes on `develop`, open PR to `main`, merge.
2. Check out the merge commit on `main`, create annotated tag:
   ```sh
   git checkout main && git pull
   git tag -a v3.0.1 -m "v3.0.1 — <one-line summary>

   <multi-paragraph release notes here — becomes the GitHub Release body>"
   git push origin v3.0.1
   ```
3. The three release workflows fire automatically:
   - `release.yml` creates the GitHub Release from the tag annotation (~30s)
   - `release-cli-binaries.yml` builds 4 CLI binaries in parallel (~5–8 min)
   - `release-gui-binaries.yml` builds 3 GUI packages in parallel (~10–15 min,
     dominated by Tauri's webview build on macOS)
4. After all three finish: visit `https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/<tag>` and confirm all binaries are attached.

## Troubleshooting

### Notarization fails with "Could not find the App-Specific Password"

`APPLE_PASSWORD` must be an **app-specific** password, not your regular Apple ID
password. Create one at <https://appleid.apple.com> → Sign-In and Security →
App-Specific Passwords → Generate. Use the password verbatim (with the dashes).

### tauri-action fails on macOS with `code signing identity not found`

`APPLE_SIGNING_IDENTITY` must exactly match the identity string shown by
`security find-identity -v -p codesigning` — typically
`Developer ID Application: <Your Name> (XXXXXXXXXX)`. The trailing 10-character
ID in parentheses is your team ID; double-check it matches `APPLE_TEAM_ID`.

### CLI build fails on `x86_64-apple-darwin` runner

GitHub deprecated the Intel macOS runner; we currently use `macos-13` which is
Apple Silicon with Rosetta. Build times on Rosetta for the x86_64 target are
2–3× slower than the native arm64 build. If this becomes a release-blocker we
can drop x86_64 macOS (Apple's own Rosetta 2 will run the arm64 binary on
older Intel Macs anyway).

### "Release already exists" warning from release.yml

Normal. The release was created by an earlier workflow run (most commonly when
the tag was pushed manually first); the second invocation is a no-op. The
binary workflows don't depend on release.yml — they attach to the release by
tag name independently.
