# Release in CI with an AppImage

Supersedes the local `just release` flow of [release.md](release.md).

## Need
Publish v1 and later versions from GitHub Actions, so the attached binary runs on the reference COSMIC distro (Pop!_OS 24.04) and not only on the maintainer's rolling distro (newer glibc), and offer an AppImage next to the bare binary.

## Flow
1. Every push to `main` or `develop` runs a `release` workflow on `ubuntu-24.04`, after the verify job passes.
2. semantic-release decides: no `feat`/`fix` since the last tag → no release. Otherwise on `develop`: prerelease (`vX.Y.Z-develop.N`, tag, notes, CHANGELOG + `Cargo.toml`/`Cargo.lock` committed), no assets. On `main`: stable release, same commit, plus two assets built after the version bump:
   - `cosmic-ext-applet-voxtype-x86_64-linux` (bare binary);
   - `cosmic-ext-applet-voxtype-x86_64.AppImage`.
3. The user downloads either one and runs it with `--install`: it is copied to `~/.local/bin/cosmic-ext-applet-voxtype`, and the `.desktop` file (`X-CosmicApplet=true`, absolute `Exec=`) and icon are written to `~/.local/share`, so the applet appears in COSMIC Settings → Panel. `--uninstall` removes all three.
4. From the AppImage, `--install` copies the AppImage file itself (`$APPIMAGE`), not the binary inside its temporary mount.
5. `just release` is removed; v1 ships by fast-forwarding `develop` into `main` and pushing.

## Edge cases and limits
- Build or AppImage packaging fails: the workflow fails before any commit, tag or GitHub release is created.
- The `chore(release)` commit is pushed with `GITHUB_TOKEN`, so it triggers no new run; it carries no `[skip ci]`, which would skip the stable run when `main` is fast-forwarded onto it.
- AppImage on a host with no FUSE: the AppImage runtime prints its own error; the README documents `--appimage-extract-and-run`. The AppImage uses the static type-2 runtime (no `libfuse2` needed, `fusermount3` is enough).
- The overlay child is spawned from `current_exe()`: inside the AppImage it is the mounted binary, valid while the applet runs.
- Private repository: release runs consume the account's Actions minutes.
- The metainfo icon URL points to a missing path (`resources/icons/hicolor/…`): fixed to `resources/icon.svg`.

## Out of scope
- `.deb`, Flatpak, AUR, multi-arch.
- AppImage desktop integration tools (Gear Lever, AppImageLauncher): `--install` is the only supported way to register the applet.
- Translating raw error output of external commands (`pactl`, `voxtype`).

## Acceptance criteria
- [ ] A push to `develop` with a `feat`/`fix` creates a prerelease with no assets; a push with only `docs`/`chore` creates nothing.
- [ ] A push to `main` creates the stable release with both assets attached, built on `ubuntu-24.04`.
- [ ] The release binary needs no glibc newer than Ubuntu 24.04's (2.39).
- [ ] `./cosmic-ext-applet-voxtype-x86_64.AppImage --install` puts the AppImage in `~/.local/bin` and the applet shows in COSMIC Settings → Panel; the overlay works; `--uninstall` removes it.
- [ ] `just release` no longer exists; `CLAUDE.md` and the README describe the new flow and both downloads.
- [ ] The metainfo icon URL resolves.

## Technical choices
- semantic-release run in CI on every push to `main`/`develop` (decided by the user).
- AppImage built with `appimagetool` (static runtime) from an AppDir holding the binary, desktop file, icon and metainfo; `libxkbcommon` stays a host library, present on every COSMIC system (decided by the user: AppImage via `--install`).

## Assumptions
- None.

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| 1 | Release workflow (semantic-release in CI, build + AppImage on `main`), remove `just release`, docs | 3 | .github/workflows/release.yml, release.config.mjs, justfile, CLAUDE.md, README.md | .github/workflows/verify.yml |
| 1 | AppImage packaging recipe + `--install` from `$APPIMAGE` | 3 | justfile, src/install.rs | src/install.rs |
| 1 | Metainfo icon URL | 1 | resources/app.metainfo.xml | — |
