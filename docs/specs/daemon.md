# Stopping and restarting the daemon

## Need
Free up the GPU (for a game, for example) by stopping Voxtype from the panel, then restart it when you want to dictate.

## Flow
1. The popup header toggle reflects `systemctl --user is-active voxtype`.
2. Turning it off: `systemctl --user stop voxtype`, immediate effect, even during dictation. Message "GPU freed, Voxtype will restart at next login".
3. Turning it on: `systemctl --user start voxtype`; the applet reconnects to the state stream.

## Edge cases and limits
- Stopped by the user (`inactive`) and failed (`failed`): same crossed-out icon; the header says "Stopped" or "Failed" with the hint `journalctl --user -u voxtype`.
- Unit missing: toggle disabled, message "Service not found: run `voxtype setup systemd`".
- Stop or start failure: inline error under the header, the toggle reverts to the real state.
- Stopped outside the applet (terminal): the state follows, with no action needed.

## Out of scope
- Disabling startup on login (`systemctl disable`).
- Automatic restart when closing a game.

## Acceptance criteria
- [ ] Turning off the toggle stops the service in under a second; `nvidia-smi` (or equivalent) no longer shows Voxtype.
- [ ] Turning it on restarts the service and the icon returns to normal without further action.
- [ ] `inactive` and `failed` display differently in the popup.
- [ ] Without a `voxtype.service` unit, the toggle is disabled and the message gives the command.
- [ ] A command failure is displayed and the toggle reflects the real state.

## Technical choices
- `systemctl --user` via the CLI, no D-Bus (decided by the user).
- Simple stop, even mid-dictation (decided by the user).

## Assumptions
- None.

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| ✅ | Daemon toggle, inactive/failed/missing states, errors | 3 | src/app.rs, src/voxtype.rs | cosmic-applets/cosmic-applet-audio |
