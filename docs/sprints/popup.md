# Popup sprint

Branch: `sprint/popup`, from `develop` (created from `main` at kickoff)
Velocity: 20

## Features
- Popup: header + Status and Orb tabs (docs/specs/applet.md): 7 (batch 0: 3, batch 1: 4)
- Mic selection, Mic tab (docs/specs/micro.md): 3
- Stopping and restarting the daemon (docs/specs/daemon.md): 3
- Develop branch and releases (docs/specs/release.md): 7

Testable deliverable: tabbed popup in the panel; turning off the toggle frees the GPU; `just release` publishes a prerelease; the binary installs with `--install`.

Mockup: https://claude.ai/artifact/Twxfwdprz56zxNrgHM3Rqp (layout B).

## To capitalise
- Voxtype `audio.device` only accepts ALSA names from `voxtype info devices`; a PipeWire source name (`alsa_input.…`) gives `Audio device not found` when recording starts, not when the daemon starts (real-world test)
- Codex's `workspace-write` sandbox can't run cargo: verifying Codex tasks falls to Claude (popup sprint)
- libcosmic: `Subscription::run_with(data, f)` recreates the subscription when `data` changes; a generation counter forces a follower (`voxtype status --follow`) to reconnect after a start, `kill_on_drop` kills the old child (daemon-1)
- semantic-release-cargo: `prepare(pluginConfig, context)` reads `context.nextRelease.version`; passing it the version alone throws a TypeError (release-2)
- Several features that add fields/messages to `AppModel` each pass clippy on their own but not together (`struct_excessive_bools`, `too_many_lines`): group state by tab (struct) and one `update_<feature>` per feature from the brief onward (batch 1 integration)

## Acceptance testing
| # | Feedback | Type | Branch | Points | Status |
|---|---|---|---|---|---|
| 1 | Status tab: correct info | — | — | — | validated |
| 2 | Daemon toggle: correctly stops Parakeet | — | — | — | validated |
| 3 | Mic: raw, unreadable ALSA list; list PipeWire sources (readable name, no monitors) and select via `pactl set-default-source`, Voxtype on `default` | adjustment | feat/micro | 3 | validated |
| 4 | Orb: the selection indicator hides the S/M/L labels | adjustment | feat/applet | 1 | validated |

## Outcome (end of sprint)
- Planned / delivered points: 20 / 20 (+4 in acceptance testing)
- Worked: structured acceptance testing (4 tracked feedback items, 2 validated right away); integration in `tmp/integ` before the demo; first prerelease 1.0.0-develop.1 via `just release`
- Failed → rule:
  - Mic specified on Voxtype's unreadable ALSA names, redone during the demo → validate system lists against the machine's real data (project's archi skill)
  - Worktrees' `target/` filled up `/tmp` (tmpfs), verify crashed → worktrees in the repo's `.worktrees/`, on disk (sprint and recette skills)
  - Features passing clippy individually but not together on `AppModel` → state per tab as a struct and `update_<feature>` from the brief onward (project's archi skill)
- Capitalised: `tech/linux/pipewire-audio-sources`, `tech/cosmic/libcosmic-applet-widgets`, `tech/rust/cargo-worktrees-tmpfs`, `tech/release/semantic-release-cargo` (created); `voxtype-integration`, `codex-sandbox-limits` (updated)
