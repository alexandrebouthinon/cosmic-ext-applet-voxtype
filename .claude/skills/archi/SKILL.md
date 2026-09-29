---
name: archi
description: "Use when writing a brief for a subagent, Codex or Copilot, or before coding in an unfamiliar module of cosmic-ext-applet-voxtype: modules, where new code goes, reference files, pitfalls."
---

# Architecture of cosmic-ext-applet-voxtype

A single binary, two processes: the applet (launched by cosmic-panel) and the overlay (`--overlay`, a child of the applet). `src/main.rs` picks one based on the argument.

## Modules
- `src/app.rs`: panel applet (icon, popup, overlay toggle). Launches and kills the `--overlay` child (`OverlayChild`, kill + wait on Drop). Depends on `voxtype`, `config`.
- `src/overlay/mod.rs`: libcosmic app with no main window; creates/destroys the click-through layer surface based on Voxtype state, fade, disc blur. Exits if `parent_id()` changes.
- `src/overlay/particles.rs`: pure, deterministic simulation (seed), no iced. Testable on its own.
- `src/overlay/audio.rs`: reads `$XDG_RUNTIME_DIR/voxtype/audio.sock` (16-byte frames) + smoothing.
- `src/overlay/view.rs`: canvas drawing and palette from the COSMIC theme.
- `src/voxtype.rs`: CLI client (`voxtype status --follow --format json`), `State`, `Event`. Shared by both processes.
- `src/config.rs`: cosmic-config (`show_overlay`, position, size), shared; the overlay reads the applet's APP_ID.

## Where to put new code
| Type | Location | Reference file |
|---|---|---|
| Panel / popup UI | `src/app.rs` | cosmic-applets `cosmic-applet-audio` |
| Persistent setting | `src/config.rs` (+ bump `Config::VERSION` on breaking change) | `src/config.rs` |
| Call to Voxtype | `src/voxtype.rs`, CLI only | `parse_status_line` + its tests |
| Animation logic | `src/overlay/particles.rs` (pure, tested) | `src/overlay/particles.rs` |
| Rendering | `src/overlay/view.rs` | `src/overlay/view.rs` |
| Displayed text | `i18n/en/*.ftl` + `fl!` | `src/app.rs` |

## Conventions
- Pure logic outside `update`/`view`, with unit tests in the same file.
- Expensive subscriptions (60 fps tick, audio socket) active only when the surface exists: 0% CPU at rest.
- Colors always drawn from the active COSMIC theme, never hardcoded.
- Clippy pedantic with `-D warnings`: exceptions via `#[expect(..., reason = ...)]`.

## Pitfalls
- Never a layer surface in the applet: cosmic-panel proxies them. The overlay is a separate process, with `X-HostWaylandDisplay=true` in the `.desktop` file.
- libcosmic blurs the whole surface by default: keep `core.set_auto_blur(Default::default())`. No compositor blur (visible banding): the orb draws its own radial veil (`view.rs`).
- Panel tests: `just install-user` then `pkill -x cosmic-panel` (relaunched by the session). Absolute `Exec=` is required.
- `pkill -f`: anchor the pattern (`'^[^ ]*cosmic-ext-applet-voxtype --overlay$'`), otherwise it kills its own shell.
- `just run` is not the panel rendering; overlay alone: `cargo run -- --overlay` (exits if its parent dies).
- Voxtype: CLI only, never modified or forked.
- System lists (mics, outputs…): validate during scoping against the machine's real data, compared to what COSMIC shows. Voxtype only sees raw ALSA PCMs; mics go through PipeWire (`src/pipewire.rs`), with Voxtype staying on `default`.
- `AppModel`: each tab's state in its own struct and one `update_<feature>` per feature, from the brief onward. Otherwise, each feature passes clippy on its own but not once combined (`struct_excessive_bools`, `too_many_lines`).
- Translations: short labels on tabs and next to controls (≈ 12 characters max), otherwise the control overflows the popup; technical terms (Backend…) stay untranslated.
