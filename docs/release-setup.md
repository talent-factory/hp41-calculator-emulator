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

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | base64-encoded Developer ID Application .p12 (cert + private key) |
| `APPLE_CERTIFICATE_PASSWORD` | Password set during the .p12 export |
| `APPLE_SIGNING_IDENTITY` | The `Developer ID Application: <Name> (<TeamID>)` string |
| `APPLE_ID` | Apple ID email used for the Developer Program |
| `APPLE_PASSWORD` | App-specific password (NOT the Apple ID login password) |
| `APPLE_TEAM_ID` | 10-character team identifier from the Developer Account |

### First-time Apple Developer setup

If you have never signed a binary for distribution before, follow these steps
in order. Skip the steps you've already done.

#### Step 1 — Verify which signing identities are already installed

```sh
security find-identity -v -p codesigning
```

You need a row that begins with `"Developer ID Application: <Your Name> (XXXXXXXXXX)"`.
If you see only `"Apple Development: ..."`, that identity is for local
development/testing and **cannot** be used for distribution outside the App
Store — you need to create a Developer ID Application cert (next step).

#### Step 2 — Create a Certificate Signing Request (CSR)

On your Mac:

1. Open **Keychain Access**.
2. Menu: **Keychain Access → Certificate Assistant → Request a Certificate
   From a Certificate Authority…**.
3. Fill in:
   - **User Email Address**: your Apple Developer Program email.
   - **Common Name**: your name (e.g. `Daniel Senften`).
   - **CA Email Address**: leave blank.
   - Select **"Saved to disk"**.
4. Save as `CertificateSigningRequest.certSigningRequest` (default name is fine).

#### Step 3 — Create the Developer ID Application certificate

1. Open <https://developer.apple.com/account/resources/certificates/list>.
2. Click the **"+"** to add a new certificate.
3. Under **Software**, select **"Developer ID Application"** (NOT "Apple
   Development" and NOT "Apple Distribution").
4. Click **Continue**.
5. Upload the `.certSigningRequest` file from Step 2.
6. Click **Continue**, then **Download** to get the `.cer` file.
7. Double-click the downloaded `.cer` — Keychain Access imports it
   automatically.

Verify the installation:

```sh
security find-identity -v -p codesigning
# Should now show two identities:
#   1) <hash> "Apple Development: ..."
#   2) <hash> "Developer ID Application: Daniel Senften (XXXXXXXXXX)"
```

The `(XXXXXXXXXX)` 10-character string in parentheses is your **`APPLE_TEAM_ID`**.

#### Step 4 — Export the certificate + private key as .p12

1. In **Keychain Access**, switch to **Login** keychain + **Certificates**
   category in the sidebar.
2. Find the `Developer ID Application: ...` row and click the disclosure
   triangle to expand it. You should see **two** items: the certificate
   itself AND a private key with the same Common Name.
3. **Select BOTH** items (Cmd-click or Shift-click). Exporting only the
   certificate without the private key produces a .p12 that cannot sign
   anything.
4. **File → Export Items…** (or right-click → "Export 2 items…").
5. **File Format**: `Personal Information Exchange (.p12)`.
6. Save as e.g. `~/Downloads/developer-id-application.p12`.
7. Set an **export password** — remember this; it becomes
   `APPLE_CERTIFICATE_PASSWORD`. (You may also be prompted for your login
   keychain password — that is a different password, used to authorize the
   export.)

#### Step 5 — Generate an app-specific password for notarization

The Apple notarization service requires an **app-specific password**, not
your Apple ID login password.

1. Open <https://appleid.apple.com>.
2. Sign in with your Apple ID.
3. Under **Sign-In and Security**, click **App-Specific Passwords**.
4. Click **Generate Password…** (or the **"+"** button).
5. Label it descriptively, e.g. `hp41-emulator-notarization`.
6. Apple shows a password in the format `xxxx-xxxx-xxxx-xxxx` (with dashes).
   Copy it verbatim — you will not be able to view it again. This is your
   `APPLE_PASSWORD` secret.

### Setting the six secrets

After all the prerequisites above are in place:

```sh
# (1) Certificate + private key as base64
gh secret set APPLE_CERTIFICATE \
  -b "$(base64 -i ~/Downloads/developer-id-application.p12)"

# (2) The .p12 export password from Step 4
gh secret set APPLE_CERTIFICATE_PASSWORD -b '<p12 export password>'

# (3) The exact signing identity string from `security find-identity`
#     (everything between the quotes — include the team ID parentheses)
gh secret set APPLE_SIGNING_IDENTITY \
  -b 'Developer ID Application: Daniel Senften (XXXXXXXXXX)'

# (4) The Apple ID email registered with the Developer Program
gh secret set APPLE_ID -b 'daniel.senften@talent-factory.ch'

# (5) The app-specific password from Step 5 (with dashes)
gh secret set APPLE_PASSWORD -b 'xxxx-xxxx-xxxx-xxxx'

# (6) The 10-character team ID
gh secret set APPLE_TEAM_ID -b 'XXXXXXXXXX'
```

### Verify the setup

```sh
gh secret list | grep APPLE
# Expected output (6 lines, names only — values are hidden):
#   APPLE_CERTIFICATE          Updated YYYY-MM-DD
#   APPLE_CERTIFICATE_PASSWORD Updated YYYY-MM-DD
#   APPLE_ID                   Updated YYYY-MM-DD
#   APPLE_PASSWORD             Updated YYYY-MM-DD
#   APPLE_SIGNING_IDENTITY     Updated YYYY-MM-DD
#   APPLE_TEAM_ID              Updated YYYY-MM-DD
```

Then run a dry-run of the binary workflow against an existing tag to confirm
the signing flow works before cutting the next real release:

```sh
gh workflow run release-cli-binaries.yml -f tag=v3.0
gh run watch   # follow the latest run
```

A successful run shows green `Sign + notarize hp41-cli (macOS)` steps with no
`::warning::` annotations about missing secrets.

### Rotating the Apple cert

Developer ID certificates expire after 5 years. To rotate:

1. Repeat Steps 2–4 with a new CSR (the old private key is **not** reused —
   each Developer ID cert pins to a new keypair).
2. Re-run the `gh secret set APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD`
   commands with the new .p12.
3. `APPLE_SIGNING_IDENTITY` may need updating if the cert's Common Name
   changes (rare).
4. `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` stay the same unless you
   change Apple ID or team membership.

The OLD certificate continues to work until it expires; binaries already
notarized with it remain valid. The new cert is only required for builds
**after** the old cert expires.

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
