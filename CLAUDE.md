# cosmic-ext-applet-voxtype

COSMIC panel applet (Rust, libcosmic) showing Voxtype's recording state. Scaffolded from `pop-os/cosmic-applet-template`.

## Commands
- Verify (fmt → clippy --fix → clippy -D warnings → tests): `just verify`
- One test: `cargo test <name>`
- Window run (quick check, not the real panel rendering): `just run`
- Panel test: `just install-user`, then add the applet in COSMIC Settings → Panel
- Voxtype state source: `voxtype status --follow --format json`; control: `voxtype record …`
- Release: automatic in CI (`.github/workflows/release.yml`) on every push to `main` or `develop`
- AppImage (needs `appimagetool` on PATH): `just appimage`

## Architecture
- `src/app.rs`: applet model, view, update, subscriptions
- `src/config.rs`: cosmic-config settings
- `i18n/`: Fluent translations (`fl!` macro)
- `resources/`: desktop file, metainfo, icon (processed by `build.rs` via xdgen)

## Conventions
- Branches: `main` + `develop` + `feat/*`. Work lands on `develop`; `develop` is FF-merged to `main` for stable releases.
- Pushing to `develop` publishes a prerelease, pushing to `main` a stable release with the binary and the AppImage. Versions and tags come from semantic-release: never use `just tag`.

## References
- Official applets: https://github.com/pop-os/cosmic-applets
- Template: https://github.com/pop-os/cosmic-applet-template

## Never
- Modify or fork Voxtype: talk to it only through its CLI.
- Require root: no `sudo`, no `just install` to `/usr`; use `just install-user`.
