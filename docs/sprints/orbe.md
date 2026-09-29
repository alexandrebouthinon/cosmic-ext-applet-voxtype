# Orbe sprint

Branch: `sprint/orbe`
Velocity: 20

## Features
- "Particle orb" overlay (docs/specs/overlay.md): 18 (batch 0: 4, batch 1: 14)
- Panel icon based on state (docs/specs/applet.md, batch 1 task): 2

Testable deliverable: applet in the panel, icon up to date, orb 10b that appears during dictation and reacts to the voice.

## To capitalise
- Voxtype's gtk4 OSD crashes on cosmic-comp 1.9: on first hide, it destroys the `zwlr_layer_surface`, then does a nil attach + commit, and the compositor disconnects it (scoping).
- Voxtype mic level: `$XDG_RUNTIME_DIR/voxtype/audio.sock`, 16-byte frames at 100 Hz, native order (seq u32, min f32, max f32, peak_dbfs f32, clamped to -120) (voxtype src/osd/ipc.rs).
- `voxtype config set audio.device` only applies after `systemctl --user restart voxtype` (scoping).

- cosmic-panel embeds a compositor that accepts applets' layer-shell and **proxies** their surfaces (EGL re-copy, input zone not passed through). A clean overlay = separate process; `X-HostWaylandDisplay` in the applet's `.desktop` passes through the real `WAYLAND_DISPLAY` (cosmic-panel `wrapper_space.rs`).
- libcosmic: click-through layer surface = `SctkLayerSurfaceSettings { input_zone: Some(vec![]) }`; app with no main window = `Settings::default().no_main_window(true)` (spike, verified).
- iced canvas: 600 `Path::circle` at 60 fps cost ~14% of one core (CPU tessellation), vs ~4% with `fill_rectangle` (spike, measured).
- libcosmic blurs the whole layer surface (square) if the theme has `frosted_system_interface`: turn it off with `core.set_auto_blur(Default::default())`, then `blur(id, Some(Vec<Rectangle>))` for a precise area; rectangles only, so a disc = horizontal bands (surface lifecycle, verified).
- cosmic-panel doesn't have `~/.local/bin` in its PATH: a user-installed applet must have an absolute `Exec=` (install-user, verified).
- Child process watching `parent_id()`: `sh -c 'cmd'` does a direct exec and the parent disappears; add `; true` for manual tests. `pkill -f` also matches the shell that launches it: anchor the pattern (`'^[^ ]*bin --flag$'`) (verified).
- Measuring a process's CPU: fields 14+15 of `/proc/PID/stat` before/after `timeout 5 tail -f /dev/null` (verified).
- COSMIC palette: the stock colors (`theme.palette.accent_*`) are pastel; saturate them for a neon look on a dark background (verified).
- Codex in `workspace-write` sandbox can't run `cargo clippy --fix` (verified).

## Outcome (end of sprint)
- Planned / delivered points: 20 / 20
- Worked: batch 0 spike that revealed the cosmic-panel proxy early; real measurements (CPU /proc, screenshots); overlay at 0% CPU at rest.
- Failed → rule:
  - LSP rule repeated by the user → discover enables the stack's LSPs in the project settings and offers to install missing servers (discover skill)
  - Applet hidden with no voxtype, fixed during the demo → an error case stays visible and distinguishable from a tool failure (cadrage skill)
  - Stale `feat/applet-icon` branch found at the end of the sprint → clean up branches and worktrees at the end of each batch (sprint skill)
  - Codex can't run `clippy --fix` → verify is rerun only if the report fails, is missing, or the diff changed (orchestrate, reviewer)
- Capitalised: tech/cosmic/cosmic-panel-applet-overlay, tech/cosmic/libcosmic-layer-surface-rendering, tech/voxtype/voxtype-integration, tech/linux/process-watchdog-pitfalls, tech/codex/codex-sandbox-limits
