# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

Note: the product is a native COSMIC desktop applet (Rust, libcosmic), not a web app. HTML is used only for mockups; the shipped UI is built from libcosmic widgets.

## Users
The author first: a COSMIC user who dictates with Voxtype through a keyboard shortcut and wants to see, at a glance, whether dictation is listening. Later, the wider community of COSMIC users who use Voxtype.

## Product Purpose
Give Voxtype a visible presence on COSMIC. The panel icon shows the daemon's state. A click-through particle orb reacts to the voice while dictating. A popup shows the daemon's details and settings, and lets the user stop the daemon to free the GPU (for a game, say) and start it again. Success: the user never opens a terminal to know or change Voxtype's state.

## Positioning
The only COSMIC-native companion for Voxtype. It talks to Voxtype only through its public CLI and audio socket, never by patching or forking it.

## Operating Context
- The applet lives in the COSMIC panel or dock. It exists only while it is added there.
- The popup is a standard applet popup, about 360 px wide, opened from the panel icon.
- Dictation is started by the user's Voxtype shortcut, not by the applet.
- The daemon runs as the systemd user service `voxtype.service`.

## Capabilities and Constraints
- Voxtype states: idle, recording, transcribing, and stopped (the daemon is down or the binary is missing).
- Daemon control goes through `systemctl --user stop|start voxtype`. A stop takes effect at once, even while dictating. Voxtype starts again at the next login.
- A stopped daemon and a failed one share the panel icon. The popup text tells them apart.
- Microphone choice: `voxtype config set audio.device`, then a service restart.
- Colours come from the active COSMIC theme; nothing is hard-coded.
- Never requires root.

## Brand Commitments
- Name: Voxtype Applet (`cosmic-ext-applet-voxtype`).
- The particle orb is the visual signature (`docs/assets/banner.svg`, `resources/icon.svg`).

## Evidence on Hand
Specs in `docs/specs/`. There are no users, testimonials or benchmarks yet; never invent them.

## Product Principles
- Native COSMIC first: the popup blends in with the official applets (audio, network). The personality stays in the orb.
- Never in the way: zero CPU and zero clicks stolen when idle.
- Every failure stays visible and distinguishable from a bug in the applet itself.
- Talk to Voxtype only through its public interfaces.

## Accessibility & Inclusion
Follow COSMIC's defaults: theme contrast, keyboard navigation and screen reader support through libcosmic widgets. No extra requirement.
