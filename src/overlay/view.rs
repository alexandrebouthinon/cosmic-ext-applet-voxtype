// SPDX-License-Identifier: MPL-2.0

//! Canvas drawing of the orb, coloured from the live COSMIC theme.

use super::particles::{Grain, Mode};
use cosmic::iced::widget::canvas;
use cosmic::iced::{Color, Point, Rectangle, Renderer, mouse};

/// Surface side the particle sizes were tuned for.
const REFERENCE_SIDE: f32 = 160.0;
/// Orb radius as a fraction of the surface side; the rest is room for voice swelling.
const RADIUS_RATIO: f32 = 0.17;

#[must_use]
pub fn radius(side: f32) -> f32 {
    side * RADIUS_RATIO
}

/// Veil reach relative to the orb radius, at rest and extra at full voice level.
const VEIL_REACH: f32 = 1.9;
const VEIL_REACH_GROWTH: f32 = 0.55;
/// Veil opacity at its centre, at rest and extra at full voice level.
const VEIL_CORE: f32 = 0.42;
const VEIL_CORE_GROWTH: f32 = 0.12;
/// Opacity falloff from the centre (0) to the rim (1), as a share of the core opacity.
const VEIL_STOPS: [(f32, f32); 5] = [
    (0.0, 1.0),
    (0.35, 0.8),
    (0.6, 0.45),
    (0.82, 0.15),
    (1.0, 0.0),
];
/// Width of one veil ring, in logical pixels: thin enough to read as a smooth gradient.
const VEIL_RING: f32 = 1.0;

/// Overall dose of the core light and grain lightening ("Blend +" mockup).
const LIGHT_STRENGTH: f32 = 0.45;
/// Core light reach relative to the orb radius, at rest and extra at full voice level.
const LIGHT_REACH: f32 = 0.9;
const LIGHT_REACH_GROWTH: f32 = 0.5;
/// Core light opacity at its centre, at rest and extra at full voice level, before the dose.
const LIGHT_CORE: f32 = 0.35;
const LIGHT_CORE_GROWTH: f32 = 0.4;
const LIGHT_STOPS: [(f32, f32); 4] = [(0.0, 1.0), (0.4, 0.55), (0.75, 0.15), (1.0, 0.0)];
/// Grains never turn fully white, so their hue still reads at the core.
const MAX_LIGHTEN: f32 = 0.7;

/// Depth above which a grain faces the viewer enough to carry a light halo.
const HALO_DEPTH: f32 = 0.5;
/// Overall dose of the halo alphas ("Blend +" mockup).
const HALO_STRENGTH: f32 = 1.8;
/// Halo alpha factor at rest and extra at full voice level, before the dose.
const HALO_BASE: f32 = 0.07;
const HALO_GROWTH: f32 = 0.12;
/// Outer halo radius relative to the grain radius, at rest and extra at full voice level.
const HALO_OUTER_REACH: f32 = 2.8;
const HALO_OUTER_REACH_GROWTH: f32 = 1.4;
/// Inner halo radius relative to the grain radius, at rest and extra at full voice level.
const HALO_INNER_REACH: f32 = 1.7;
const HALO_INNER_REACH_GROWTH: f32 = 0.6;

/// Reach (in pixels) and centre opacity of the dark veil around an orb of `radius`.
#[must_use]
pub fn veil(radius: f32, level: f32) -> (f32, f32) {
    let level = level.clamp(0.0, 1.0);
    (
        radius * VEIL_REACH_GROWTH.mul_add(level, VEIL_REACH),
        VEIL_CORE_GROWTH.mul_add(level, VEIL_CORE),
    )
}

/// Share of the centre opacity at `t` = distance / reach, interpolated between `stops`.
fn falloff(stops: &[(f32, f32)], t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    stops
        .windows(2)
        .find(|pair| t <= pair[1].0)
        .map_or(0.0, |pair| {
            let ((t0, a0), (t1, a1)) = (pair[0], pair[1]);
            a0 + (a1 - a0) * (t - t0) / (t1 - t0)
        })
}

/// Reach (in pixels) and centre opacity of the light at the core of an orb of `radius`.
#[must_use]
pub fn core_light(radius: f32, level: f32, fade: f32) -> (f32, f32) {
    let level = level.clamp(0.0, 1.0);
    (
        radius * LIGHT_REACH_GROWTH.mul_add(level, LIGHT_REACH),
        LIGHT_CORE_GROWTH.mul_add(level, LIGHT_CORE) * LIGHT_STRENGTH * fade,
    )
}

/// How far a grain fades toward white: most at the centre (`dist` 0) and in front (`depth` 1).
#[must_use]
pub fn grain_lightening(dist: f32, depth: f32, level: f32) -> f32 {
    let level = level.clamp(0.0, 1.0);
    let lit = (1.0 - dist).max(0.0)
        * 0.45f32.mul_add(level, 0.35)
        * 0.6f32.mul_add(depth, 0.4)
        * LIGHT_STRENGTH;
    lit.min(MAX_LIGHTEN)
}

/// Outer and inner halo (radius, alpha) for a front grain of `size` (radius, px), or `None`
/// behind [`HALO_DEPTH`]. `fade` is the overlay's global opacity.
#[must_use]
pub fn halos(size: f32, depth: f32, level: f32, fade: f32) -> Option<[(f32, f32); 2]> {
    if depth <= HALO_DEPTH {
        return None;
    }
    let level = level.clamp(0.0, 1.0);
    let k = HALO_GROWTH.mul_add(level, HALO_BASE) * HALO_STRENGTH;
    let outer = (
        size * HALO_OUTER_REACH_GROWTH.mul_add(level, HALO_OUTER_REACH),
        k * 0.5 * depth * fade,
    );
    let inner = (
        size * HALO_INNER_REACH_GROWTH.mul_add(level, HALO_INNER_REACH),
        k * depth * fade,
    );
    Some([outer, inner])
}

/// Colour of the core light: pink-white while dictating, a light shade of the accent while transcribing.
#[must_use]
pub fn light_color(theme: &cosmic::cosmic_theme::Theme, mode: Mode) -> Color {
    match mode {
        Mode::Listening => mix(palette(theme, mode)[1], Color::WHITE, 0.25),
        Mode::Transcribing => mix(Color::from(theme.accent_color()), Color::WHITE, 0.3),
    }
}

/// Very dark violet body lit by fuchsia and deep indigo grains, with accent sparks.
/// Transcribing switches every grain to shades of the accent.
#[must_use]
pub fn palette(theme: &cosmic::cosmic_theme::Theme, mode: Mode) -> [Color; 4] {
    let accent = Color::from(theme.accent_color());
    match mode {
        Mode::Listening => {
            let purple = Color::from(theme.palette.accent_purple);
            let pink = Color::from(theme.palette.accent_pink);
            let indigo = Color::from(theme.palette.accent_indigo);
            // The stock palette is pastel: pushed away from grey, it reads as neon on the dark orb.
            [
                saturate(mix(purple, Color::BLACK, 0.45), 1.5),
                saturate(mix(pink, purple, 0.35), 3.0),
                saturate(mix(indigo, Color::BLACK, 0.35), 2.0),
                accent,
            ]
        }
        Mode::Transcribing => [
            mix(accent, Color::BLACK, 0.2),
            accent,
            mix(accent, Color::WHITE, 0.3),
            accent,
        ],
    }
}

fn saturate(c: Color, amount: f32) -> Color {
    let grey = (c.r + c.g + c.b) / 3.0;
    let push = |v: f32| (v + (v - grey) * amount).clamp(0.0, 1.0);
    Color::from_rgba(push(c.r), push(c.g), push(c.b), c.a)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a,
    )
}

pub struct Orb<'a> {
    pub grains: &'a [Grain],
    pub mode: Mode,
    /// Global opacity for the fade-in and fade-out, in 0..=1.
    pub fade: f32,
    /// Smoothed voice level, in 0..=1.
    pub level: f32,
}

/// Radial gradient faked with nested discs drawn from the rim inwards rather than
/// disjoint rings: disjoint rings leave anti-aliasing seams where the background shows through.
fn draw_radial(
    frame: &mut canvas::Frame,
    reach: f32,
    peak: f32,
    stops: &[(f32, f32)],
    color: Color,
) {
    let center = frame.center();
    let mut covered = 0.0;
    let mut r = reach;
    while r > 0.0 {
        let target = peak * falloff(stops, (r - VEIL_RING / 2.0).max(0.0) / reach);
        // Each disc adds only what the outer ones lack, so stacked alphas meet the target.
        let alpha = 1.0 - (1.0 - target) / (1.0 - covered);
        if alpha > 0.0 {
            frame.fill(
                &canvas::Path::circle(center, r),
                Color { a: alpha, ..color },
            );
            covered = target;
        }
        r -= VEIL_RING;
    }
}

impl<M> canvas::Program<M, cosmic::Theme> for Orb<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &cosmic::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = frame.center();
        let scale = bounds.width.min(bounds.height) / REFERENCE_SIDE;
        let colors = palette(theme.cosmic(), self.mode);
        let radius = radius(bounds.width.min(bounds.height));
        // Same veil while transcribing: only the grains change colour.
        let body = palette(theme.cosmic(), Mode::Listening)[0];
        let (reach, core) = veil(radius, self.level);
        draw_radial(
            &mut frame,
            reach,
            core * self.fade,
            &VEIL_STOPS,
            mix(body, Color::BLACK, 0.8),
        );
        let (reach, core) = core_light(radius, self.level, self.fade);
        draw_radial(
            &mut frame,
            reach,
            core,
            &LIGHT_STOPS,
            light_color(theme.cosmic(), self.mode),
        );
        // Grains sit on a sphere swollen by the voice (`Swarm::grains`).
        let sphere = radius * 0.45f32.mul_add(self.level, 1.0);
        for grain in self.grains {
            let dist = grain.x.hypot(grain.y) / sphere;
            let lit = grain_lightening(dist, grain.depth, self.level);
            let color = mix(colors[usize::from(grain.color % 4)], Color::WHITE, lit);
            let point = Point::new(center.x + grain.x, center.y + grain.y);
            let size = grain.size * scale / 2.0;
            // Halos are drawn under the grain disc, outer first so the inner one layers on top.
            if let Some([outer, inner]) = halos(size, grain.depth, self.level, self.fade) {
                for (radius, alpha) in [outer, inner] {
                    frame.fill(
                        &canvas::Path::circle(point, radius),
                        Color { a: alpha, ..color },
                    );
                }
            }
            frame.fill(
                &canvas::Path::circle(point, size),
                Color {
                    a: lit.mul_add(0.4, grain.alpha).min(1.0) * self.fade,
                    ..color
                },
            );
        }
        vec![frame.into_geometry()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcribing_uses_only_accent_shades() {
        let theme = cosmic::cosmic_theme::Theme::dark_default();
        let body = palette(&theme, Mode::Listening)[0];
        let accent = Color::from(theme.accent_color());
        let transcribing = palette(&theme, Mode::Transcribing);
        assert!(!transcribing.contains(&body));
        assert!(transcribing.contains(&accent));
    }

    #[test]
    fn core_light_grows_with_the_voice_and_vanishes_when_faded_out() {
        let (quiet_reach, quiet) = core_light(10.0, 0.0, 1.0);
        let (loud_reach, loud) = core_light(10.0, 1.0, 1.0);
        assert!(quiet > 0.0 && loud > quiet && loud_reach > quiet_reach);
        assert!(core_light(10.0, 1.0, 0.0).1.abs() < f32::EPSILON);
    }

    #[test]
    fn grain_lightening_grows_with_the_voice_toward_the_centre_and_front() {
        assert!(grain_lightening(0.2, 0.8, 1.0) > grain_lightening(0.2, 0.8, 0.0));
        assert!(grain_lightening(0.1, 0.8, 0.5) > grain_lightening(0.6, 0.8, 0.5));
        assert!(grain_lightening(0.2, 1.0, 0.5) > grain_lightening(0.2, 0.0, 0.5));
        assert!(grain_lightening(1.5, 1.0, 1.0).abs() < f32::EPSILON);
        assert!(grain_lightening(0.0, 1.0, 1.0) <= MAX_LIGHTEN);
    }

    #[test]
    fn transcribing_light_is_an_accent_shade() {
        let theme = cosmic::cosmic_theme::Theme::dark_default();
        let accent = Color::from(theme.accent_color());
        let light = light_color(&theme, Mode::Transcribing);
        assert_eq!(light, mix(accent, Color::WHITE, 0.3));
        assert_ne!(light, light_color(&theme, Mode::Listening));
    }

    #[test]
    fn veil_matches_the_mockup_at_rest_and_full_voice() {
        let (reach, core) = veil(10.0, 0.0);
        assert!((reach - 19.0).abs() < 1e-4 && (core - 0.42).abs() < 1e-4);
        let (reach, core) = veil(10.0, 1.0);
        assert!((reach - 24.5).abs() < 1e-4 && (core - 0.54).abs() < 1e-4);
    }

    #[test]
    fn veil_falloff_is_continuous_and_fades_out_at_the_rim() {
        assert!((falloff(&VEIL_STOPS, 0.0) - 1.0).abs() < f32::EPSILON);
        assert!(falloff(&VEIL_STOPS, 1.0).abs() < f32::EPSILON);
        let mut previous = falloff(&VEIL_STOPS, 0.0);
        for i in 1..=1000u16 {
            let current = falloff(&VEIL_STOPS, f32::from(i) / 1000.0);
            assert!(current <= previous, "rises at {i}");
            assert!(previous - current < 0.01, "jumps at {i}");
            previous = current;
        }
    }

    #[test]
    fn veil_fits_the_surface_at_full_voice() {
        for side in [180.0, 240.0, 300.0] {
            let (reach, _) = veil(radius(side), 1.0);
            assert!(reach < side / 2.0, "side {side}: reach {reach}");
        }
    }

    #[test]
    fn halos_grow_with_the_voice_and_vanish_when_faded_out_or_facing_away() {
        let quiet = halos(4.0, 1.0, 0.0, 1.0).expect("front grain has halos");
        let loud = halos(4.0, 1.0, 1.0, 1.0).expect("front grain has halos");
        for i in 0..2 {
            assert!(
                loud[i].0 > quiet[i].0,
                "radius {i} does not grow with level"
            );
            assert!(loud[i].1 > quiet[i].1, "alpha {i} does not grow with level");
            assert!(
                quiet[i].1 > 0.0,
                "alpha {i} is zero at rest (silence floor)"
            );
        }
        assert!(halos(4.0, 1.0, 1.0, 0.0).is_none_or(|h| h.iter().all(|(_, a)| *a == 0.0)));
        assert!(halos(4.0, 0.5, 0.5, 1.0).is_none());
        assert!(halos(4.0, 0.0, 0.5, 1.0).is_none());
    }

    #[test]
    fn front_grain_halos_average_at_most_two_extra_discs_per_grain_over_a_rotation() {
        use super::super::particles::{Mode, Swarm};

        let level = 0.5;
        let mode = Mode::Listening;
        // Same formula as `Swarm::step`'s `Mode::Listening` branch.
        let rotation_speed = 0.9f32.mul_add(level, 0.45);
        let frames: u16 = 64;
        let dt = std::f32::consts::TAU / rotation_speed / f32::from(frames);

        let mut swarm = Swarm::new(600, 42);
        let mut grains = Vec::new();
        let mut total_discs = 0.0f32;

        for _ in 0..frames {
            swarm.step(dt, level, mode);
            swarm.grains(15.0, 1.0, level, mode, &mut grains);
            let halo_count = grains
                .iter()
                .filter(|grain| halos(1.0, grain.depth, level, 1.0).is_some())
                .count();
            let halo_count = f32::from(u16::try_from(halo_count).expect("halo count fits in u16"));
            let grain_count =
                f32::from(u16::try_from(grains.len()).expect("grain count fits in u16"));
            total_discs += grain_count + 2.0 * halo_count;
        }

        let grain_count = 600.0;
        let average = total_discs / f32::from(frames);
        assert!(
            average <= 2.0 * grain_count,
            "average discs {average} exceeds budget of {}",
            2.0 * grain_count
        );
    }

    #[test]
    fn orb_fits_the_surface_at_full_swell() {
        // Grain centres stay within 2.7 radius (`Swarm::grains`); the largest grain
        // adds about 6 px at the reference side.
        for side in [180.0, 240.0, 300.0] {
            let reach = radius(side) * 2.7 + 6.0 * side / REFERENCE_SIDE;
            assert!(reach < side / 2.0, "side {side}: reach {reach}");
        }
    }
}
