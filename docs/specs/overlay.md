# "Particle orb" overlay

## Need
The user dictates with Voxtype via a COSMIC shortcut and gets no visual feedback. An on-screen particle orb shows them that the mic is listening and reacting to their voice.

## Flow
1. The user presses the Voxtype shortcut: the state switches to `recording`.
2. The orb fades in (bottom-right of the main screen, by default): about 600 orbiting particles forming a sphere (mockup 10b).
3. During recording, the radius, speed and jitter follow the mic level (`audio.sock`).
4. During transcription (`transcribing`), the orb contracts, spins faster and switches to the accent color.
5. Back to `idle`: fade-out, the surface is hidden.

## Edge cases and limits
- The orb only exists if the applet is added to a panel or the dock (COSMIC only launches the applet in that case) and the popup's "overlay" switch is on.
- `audio.sock` missing or closed: the orb still shows, with a resting animation (level 0), and reconnects in the background.
- Daemon stopped while displayed: the orb hides.
- The overlay never takes clicks or focus (a "click-through" surface).
- COSMIC theme change: the colors follow without a restart.

## Out of scope
- "Reduced animations" mode (backlog).
- Multi-screen support (backlog): a single orb when the applet runs on multiple panels (instance lock, to scope by studying cosmic-applets), screen choice. Test planned with a user who has two screens.
- Starting or stopping recording by clicking the orb (backlog).

## Acceptance criteria
- [ ] Applet removed from the panel: no orb, even during a recording.
- [ ] The orb appears within 150 ms of switching to `recording`, and disappears when returning to `idle`.
- [ ] The radius and speed visibly vary when speaking, and stay at rest in silence.
- [ ] In `transcribing`, the colors switch to the theme's accent.
- [ ] A click on the orb reaches the window underneath.
- [ ] Position (4 corners + bottom-center), size (S/M/L) and activation are readable from the applet's config.
- [ ] The overlay process uses less than 5% of one CPU core during the animation, and 0% at rest (surface hidden, no render loop).

## Technical choices
- Architecture: child process, the same binary in `--overlay` mode, launched and killed by the applet (decided after the spike). The `.desktop` file carries `X-HostWaylandDisplay=true` so cosmic-panel passes through the real `WAYLAND_DISPLAY`.
- Rendering: libcosmic's `wgpu` feature; particles as `fill_rectangle` (`Path::circle` costs 14% CPU vs 4% for 600 grains at 60 fps), circles reserved for the rare large grains.
- Rendering: iced `canvas`; a wgpu shader only if the canvas struggles (decided by the user).
- Colors derived from the COSMIC theme: accent → sparkles and transcription; purples and cyan from the palette → body (decided by the user).
- Audio level: `$XDG_RUNTIME_DIR/voxtype/audio.sock`, 16-byte frames at 100 Hz (seq u32, min f32, max f32, peak_dbfs f32).
- Screen: the main screen (decided by the user).

## Assumptions
- `IcedOutput::Active` places the surface on the active screen; with a single screen, that's the main screen (multi-screen in the backlog).
- Color mapping onto the theme palette: 35% purple (`accent_purple`), 25% light purple, 25% cyan (`accent_blue`), 15% accent (fine-tuning possible later).

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| 0 | Voxtype client: `status --follow` subscription, idle/recording/transcribing/daemon missing/binary missing states | 3 | src/voxtype.rs | cosmic-applets/cosmic-applet-audio |
| 0 | Config: position, size, overlay enabled | 1 | src/config.rs | template |
| 1 | Spike: layer-shell surface from the applet, empty input (click-through), default output | 3 | src/overlay/ | cosmic-osd |
| 1 | `audio.sock` reader: frame parsing, reconnection, smoothing | 2 | src/overlay/audio.rs | voxtype src/osd/ipc.rs |
| 1 | Particle simulation (port of 9a) + unit tests | 3 | src/overlay/particles.rs | mockup 10b |
| 1 | Canvas rendering + theme colors | 3 | src/overlay/view.rs | libcosmic theme |
| 1 | Surface lifecycle: show/hide based on state, position, size | 3 | src/overlay/mod.rs, src/app.rs | cosmic-osd |
