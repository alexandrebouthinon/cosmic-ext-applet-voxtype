# Panel applet

## Need
See Voxtype's state at a glance, check its info and configure the overlay without going through the terminal.

## Flow
1. The panel icon reflects the state: idle, recording, transcribing, daemon stopped (crossed-out icon).
2. Click on the icon: a popup opens (layout B of the mockup: https://claude.ai/artifact/Twxfwdprz56zxNrgHM3Rqp).
3. Fixed header: state glyph, "Voxtype", status text, daemon toggle (daemon.md).
4. Tabs (`segmented_button`):
   - Status: model and backend (`voxtype status --format json --extended`);
   - Mic: list of inputs (micro.md);
   - Orb: shown/hidden toggle, position in a dropdown (4 corners + bottom-center, bottom-right by default), S/M/L size in a segmented control.

## Edge cases and limits
- Daemon stopped: crossed-out icon, Status tab "unavailable", automatic reconnection (backoff).
- `voxtype` binary missing: crossed-out icon, same as daemon stopped (the applet stays visible to prove it's running).

## Out of scope
- Start or stop recording on click (backlog).
- `device` field of `status --extended`: that's the mic, shown in the Mic tab only.

## Acceptance criteria
- [ ] The icon changes state in under 200 ms after a Voxtype change.
- [ ] Stopping `voxtype.service` gives the crossed-out icon; restarting it restores the normal state with no user action.
- [ ] Without the `voxtype` binary in the PATH, the applet shows the crossed-out icon.
- [ ] The popup shows the real model and backend; "unavailable" when the daemon is stopped.
- [ ] Orb position and size are set in the Orb tab.
- [ ] An overlay setting change applies immediately and persists after a session restart.

## Technical choices
- Layout: header + tabs (B); position in a dropdown (decided by the user).
- Settings in cosmic-config (`src/config.rs`); the overlay reads them live.

## Assumptions
- None.

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| ✅ | Icon based on state (including crossed-out), crossed-out icon if binary is missing (orbe sprint) | 2 | src/app.rs, resources/ | cosmic-applets/cosmic-applet-audio |
| ✅ | Popup skeleton: header, tabs, messages; CLI wrappers (`status --extended`, `systemctl --user`, devices) in src/voxtype.rs | 3 | src/app.rs, src/voxtype.rs | cosmic-applets/cosmic-applet-audio |
| ✅ | Status tab (model, backend, "unavailable") | 2 | src/app.rs | cosmic-applets/cosmic-applet-audio |
| ✅ | Orb tab (toggle, position dropdown, segmented size) | 2 | src/app.rs, src/config.rs | cosmic-applets |
