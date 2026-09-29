# Mic selection

## Need
Choose from the panel which mic Voxtype will use for the next dictations.

## Flow
1. In the popup's Mic tab, a list shows PipeWire sources (`pw-dump`, `media.class` `Audio/Source*`, including virtual ones like Easy Effects, no monitors), under their readable name (`node.description`), with the default source selected.
2. The user picks a source: `pactl set-default-source <name>`. A note indicates the mic changes for all apps.
3. If Voxtype isn't set to `audio.device = default`: `voxtype config set audio.device default`, then `systemctl --user restart voxtype`; the icon briefly switches to "daemon stopped".

## Edge cases and limits
- Recording or transcription in progress: the list is disabled.
- Daemon stopped by the user: `config set` only, no restart; the switch to `default` applies on next startup (message in the tab).
- No source: "no mic" message in the tab.
- `pactl` failure: error message, the selection doesn't change. `config set` or restart failure: error message, the default source stays changed.

## Out of scope
- Testing the mic (level meter) in the popup.
- Capturing an audio output (monitor) for transcription (backlog, if the need arises).

## Acceptance criteria
- [ ] The list shows readable names (e.g. Blue Microphones, Easy Effects Source), with no outputs or monitors.
- [ ] Choosing a mic changes the default PipeWire source; the next dictation uses that mic.
- [ ] Voxtype off `default`: it's set back and restarted; if the daemon is stopped, it isn't relaunched.
- [ ] Impossible to change mic during a recording.
- [ ] An error is displayed and nothing changes if the command fails.

## Technical choices
- Automatic restart of the user service (decided by the user).

## Functional choices
- Inputs only: no outputs or monitors in the list (decided by the user).
- Mic source: PipeWire, not `voxtype info devices` (raw ALSA PCMs, unreadable). Voxtype rejects PipeWire source names (`Audio device not found`, tested): it stays on `default` and the applet changes the system's default source instead (decided by the user during acceptance testing).
- `pw-dump` rather than `pactl --format=json`: the latter produces invalid JSON with accented descriptions (pactl 17).

## Assumptions
- None.

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| ✅ | Device list, selection, config set + restart, errors | 3 | src/voxtype.rs, src/app.rs | cosmic-applets/cosmic-applet-audio |
