# Develop branch and releases

## Need
Publish clean versions of the applet (tag, notes, binary) with no manual work, and install the applet without cloning the repo.

## Flow
1. Work lands on `develop`; `just release` publishes a prerelease there (`vX.Y.Z-develop.N`): tag, notes, `Cargo.toml`, `Cargo.lock` and CHANGELOG committed, no binary.
2. `develop` fast-forward merged into `main`; `just release` publishes the stable version: tag, GitHub Release, CHANGELOG, `Cargo.toml` bump, x86_64 binary built locally and attached.
3. The user downloads the binary and runs `cosmic-ext-applet-voxtype --install`: copies it to `~/.local/bin`, writes the `.desktop` file (with `X-CosmicApplet=true` and an absolute `Exec=`) and the icon to `~/.local/share`. `--uninstall` removes all three.
4. GitHub Actions runs `just verify` on every push and PR.

## Edge cases and limits
- `just release` outside `main`/`develop`, or a dirty tree: refused with a message.
- No `feat`/`fix` commit since the last version: no release, semantic-release message.
- `--install` when `~/.local/bin` doesn't exist: it's created. File already present: replaced.

## Out of scope
- AppImage, Flatpak (the sandbox blocks the voxtype CLI, systemctl and `audio.sock`), deb, rpm.
- Multi-arch builds in CI (once the repo is public).

## Acceptance criteria
- [ ] The `develop` branch exists on the remote and `CLAUDE.md` describes the flow.
- [ ] `just release` on `develop` creates a prerelease with no binary.
- [ ] `just release` on `main` creates a release with the binary attached.
- [ ] The downloaded binary, with `--install`, appears in COSMIC Settings → Panel; `--uninstall` removes it.
- [ ] CI fails if `just verify` fails.

## Technical choices
- semantic-release + semantic-release-cargo, run locally via `just release` (decided by the user).
- Binary alone with `--install`/`--uninstall`, embedded `.desktop` and icon (decided by the user).
- Prereleases with no binary, x86_64 only (decided by the user).

## Assumptions
- None.

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| ✅ | Develop branch, release.config.mjs, package.json, `just release` recipe, CHANGELOG | 3 | release.config.mjs, package.json, justfile, CLAUDE.md | — |
| ✅ | `--install` / `--uninstall` (embedded desktop file + icon) | 3 | src/main.rs, src/install.rs | justfile (install-user recipe) |
| ✅ | CI workflow `just verify` | 1 | .github/workflows/verify.yml | — |
