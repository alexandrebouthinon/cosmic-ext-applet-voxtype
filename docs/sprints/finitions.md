# Finitions sprint

Branch: `sprint/finitions`, from `develop`
Velocity: 20 (small sprint, outside the backlog)

## Features
- Orb halo, dark veil (docs/specs/halo.md): 5
- French translation (docs/specs/i18n-fr.md): 4

No batch 0: no shared contract between the two features.

Testable deliverable: the orb shows with a smooth veil that follows the voice, with no banding; with `LANG=fr_FR.UTF-8`, the applet is in French.

Mockup: https://claude.ai/artifact/CYw8gJCW6DM68gC6fsGHab (variant B).

## To capitalise

## Acceptance testing
| # | Feedback | Type | Branch | Points | Status |
|---|---|---|---|---|---|
| 1 | Microphone tab too long: "Micro" / "Mic" | adjustment | feat/i18n-fr | 1 | validated |
| 2 | Orb switch label too long, pushes the button off screen: "Activer" / "Enable" | adjustment | feat/i18n-fr | 1 | validated |

## Outcome (end of sprint)
- Planned / delivered points: 9 / 9 (+2 in acceptance testing)
- Worked: mockup in 3 variants, veil chosen on the first try; veil in nested discs, smooth and seamless.
- Failed → rule:
  - FR labels too long (tab, orb switch) and "Backend" mistakenly translated → short labels, technical terms not translated (`archi` pitfall)
- Capitalised: `tech/cosmic/libcosmic-layer-surface-rendering` (radial veil drawn under iced), `tech/cosmic/cosmic-panel-applet-overlay` (`.desktop` translation via xdgen)
