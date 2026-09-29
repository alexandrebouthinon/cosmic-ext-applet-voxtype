# Grain light

## Need
Today, the orb's grains are simple flat circles that move. The user wants them to look like they glow: a soft light at the orb's core and a halo around the front grains, both following the voice (mockup "Blend +").

## Flow
1. The orb appears: the core light and the halos rise with the fade-in.
2. While dictating:
   - the core glows pink and white;
   - grains near the center and foreground fade toward white;
   - front grains (depth > 0.5) carry two translucent halos in their own color, slightly lightened.
3. As the voice rises: the core widens and intensifies, the halos widen and thicken. Both follow the smoothed level (attack 8/s, release 2/s).
4. During transcription: same effect, the core and grains shift to the accent amber.
5. The orb disappears: everything fades out with it.

## Edge cases and limits
- Only flat translucent discs: iced's canvas has no radial gradient, additive blending, or blur.
- Cost: at most 2 discs per grain, averaged over one rotation (×2 limit set by the user, about 8% of one core).
- Core light: nested discs, like the veil; its cost adds to the veil's, not to the grains'.
- Settings taken from the "Blend +" mockup:
  - core at 45%: alpha `(0.35 + 0.4·level) × 0.45` and reach `radius × (0.9 + 0.5·level)`;
  - grain lightening: `(1 − dist)·(0.35 + 0.45·level)·(0.4 + 0.6·depth)·0.45`, capped at 0.7;
  - halos: `k = (0.07 + 0.12·level) × 1.8`. Outer halo of radius `size × (2.8 + 1.4·level)` and alpha `k·0.5·depth`; inner halo of radius `size × (1.7 + 0.6·level)` and alpha `k·depth`.
- Silence: the halos stay visible but subtle (k ≈ 0.13), never disappearing.

## Out of scope
- Sparkles, shimmer, reflections (variant B).
- Intensity setting in the popup.
- Changes to the veil, motion, or base grain colors.

## Acceptance criteria
- [ ] Cost test: over one rotation, the average number of grain discs (halos included) is at most 2 × the number of grains.
- [ ] Test: core intensity and halo intensity increase with the level and equal 0 when the fade equals 0.
- [ ] Test: during transcription, the core light is a shade of the accent color.
- [ ] On the panel, the orb looks like the "Blend +" mockup while dictating and transcribing, on both bright and dark backgrounds.
- [ ] CPU measured during dictation: about 8% of one core at most.

## Technical choices
- Variant: blend A (halos) + C (bright core), "Blend +" dosage (decided by the user).
- Light during transcription: same effect, in amber (decided by the user).
- CPU budget: up to about ×2 compared to the current level (decided by the user).
- Core drawn like the veil (nested discs every 1 px, `alpha = 1 − (1 − target)/(1 − covered)`), reusing `draw_veil`'s logic.

## Assumptions

## Open questions

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| 1 | Bright core: nested discs, grain lightening, intensity and color tests | 3 | `src/overlay/view.rs` | `src/overlay/view.rs` (`draw_veil`) |
| 1 | Front grain halos, cost test | 2 | `src/overlay/view.rs` | `src/overlay/view.rs` (`Orb::draw`) |
| 1 | Panel tuning and CPU measurement | 1 | `src/overlay/view.rs` | — |
