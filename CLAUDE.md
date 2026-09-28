# cosmic-ext-applet-voxtype

COSMIC panel applet (Rust, libcosmic) showing Voxtype's recording state. Scaffolded from `pop-os/cosmic-applet-template`.

## Commands
- Verify (fmt → clippy --fix → clippy -D warnings → tests): `just verify`
- One test: `cargo test <name>`
- Window run (quick check, not the real panel rendering): `just run`
- Panel test: `just install-user`, then add the applet in COSMIC Settings → Panel
- Voxtype state source: `voxtype status --follow --format json`; control: `voxtype record …`

## Architecture
- `src/app.rs`: applet model, view, update, subscriptions
- `src/config.rs`: cosmic-config settings
- `i18n/`: Fluent translations (`fl!` macro)
- `resources/`: desktop file, metainfo, icon (processed by `build.rs` via xdgen)

## Conventions
- Branches: `main` + `feat/*`, FF merge after validation. A `develop` branch is planned later with semantic-release.
- Versions and tags come from semantic-release: never use `just tag`.

## References
- Official applets: https://github.com/pop-os/cosmic-applets
- Template: https://github.com/pop-os/cosmic-applet-template

## Never
- Modify or fork Voxtype: talk to it only through its CLI.
- Require root: no `sudo`, no `just install` to `/usr`; use `just install-user`.
