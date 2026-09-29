# Orb halo

## Need
The user dictates with the orb on screen. The compositor blur, approximated by 24 bands, shows stepping, a hard edge and slits where the background shows through (very visible on a bright background). It is replaced by a drawn, smooth dark veil that follows the voice.

## Flow
1. Dictation starts: the orb fades in with its veil.
2. While speaking: the veil expands and thickens with the smoothed mic level (same smoothing as the orb).
3. Silence: it returns to its resting size.
4. End: it fades out with the orb.

## Edge cases and limits
- No audio stream (socket missing): level 0, veil at rest, same as the orb today.
- Transcription: same veil; only the grains change color.
- S/M/L sizes: the veil is proportional to the orb's radius, never clipped by the surface edge.

## Out of scope
- Real background blur (removed, no more `blur` request to the compositor).
- Colored halo, veil setting in the popup.

## Acceptance criteria
- [ ] No more calls to the blur protocol or band-splitting in `src/overlay/`.
- [ ] Dark radial veil (mockup variant B): core ≈ 0.42 opacity, reach 1.9 × radius at rest, up to 2.45 × radius and 0.54 opacity at max level.
- [ ] Continuous opacity profile decreasing to 0 at the edge (unit test), with no visible stepping on a bright background at 2× screen zoom.
- [ ] The veil fits within the surface for all 3 sizes (unit test).
- [ ] The veil follows the fade-in and fade-out.

## Technical choices
- Rendering: veil drawn in the orb's canvas (decided by the user: "halo drawn alone", variant B).
- Gradient: iced has no radial gradient; nested discs every 1 px, from the edge to the center, each adding the missing opacity (disjoint rings would leave anti-aliasing seams), profile drawn from a pure, tested function.
- Veil drawn even when the theme doesn't have "frosted interface" (it no longer depends on the compositor).

## Assumptions

## Open questions

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| 1 | Veil profile function (reach, opacity based on distance and level) + tests | 2 | `src/overlay/view.rs` | `view.rs` (`radius`, `mix` and their tests) |
| 1 | Veil ring drawing in `Orb::draw` (level passed to `Orb`), removal of compositor blur (`disc`, `BLUR_*`, `blur`) | 3 | `src/overlay/view.rs`, `src/overlay/mod.rs` | `Orb::draw` |
