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

/// Reach (in pixels) and centre opacity of the dark veil around an orb of `radius`.
#[must_use]
pub fn veil(radius: f32, level: f32) -> (f32, f32) {
    let level = level.clamp(0.0, 1.0);
    (
        radius * VEIL_REACH_GROWTH.mul_add(level, VEIL_REACH),
        VEIL_CORE_GROWTH.mul_add(level, VEIL_CORE),
    )
}

/// Share of the core opacity at `t` = distance / reach: continuous, decreasing, 0 at the rim.
#[must_use]
pub fn veil_falloff(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    VEIL_STOPS
        .windows(2)
        .find(|pair| t <= pair[1].0)
        .map_or(0.0, |pair| {
            let ((t0, a0), (t1, a1)) = (pair[0], pair[1]);
            a0 + (a1 - a0) * (t - t0) / (t1 - t0)
        })
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

impl Orb<'_> {
    /// Nested discs drawn from the rim inwards rather than disjoint rings: disjoint
    /// rings leave anti-aliasing seams where the background shows through.
    fn draw_veil(&self, frame: &mut canvas::Frame, radius: f32, color: Color) {
        let center = frame.center();
        let (reach, core) = veil(radius, self.level);
        let peak = core * self.fade;
        let mut covered = 0.0;
        let mut r = reach;
        while r > 0.0 {
            let target = peak * veil_falloff((r - VEIL_RING / 2.0).max(0.0) / reach);
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
        // Same veil while transcribing: only the grains change colour.
        let body = palette(theme.cosmic(), Mode::Listening)[0];
        self.draw_veil(
            &mut frame,
            radius(bounds.width.min(bounds.height)),
            mix(body, Color::BLACK, 0.8),
        );
        for grain in self.grains {
            let color = colors[usize::from(grain.color % 4)];
            frame.fill(
                &canvas::Path::circle(
                    Point::new(center.x + grain.x, center.y + grain.y),
                    grain.size * scale / 2.0,
                ),
                Color {
                    a: grain.alpha * self.fade,
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
    fn veil_matches_the_mockup_at_rest_and_full_voice() {
        let (reach, core) = veil(10.0, 0.0);
        assert!((reach - 19.0).abs() < 1e-4 && (core - 0.42).abs() < 1e-4);
        let (reach, core) = veil(10.0, 1.0);
        assert!((reach - 24.5).abs() < 1e-4 && (core - 0.54).abs() < 1e-4);
    }

    #[test]
    fn veil_falloff_is_continuous_and_fades_out_at_the_rim() {
        assert!((veil_falloff(0.0) - 1.0).abs() < f32::EPSILON);
        assert!(veil_falloff(1.0).abs() < f32::EPSILON);
        let mut previous = veil_falloff(0.0);
        for i in 1..=1000u16 {
            let current = veil_falloff(f32::from(i) / 1000.0);
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
    fn orb_fits_the_surface_at_full_swell() {
        // Grain centres stay within 2.7 radius (`Swarm::grains`); the largest grain
        // adds about 6 px at the reference side.
        for side in [180.0, 240.0, 300.0] {
            let reach = radius(side) * 2.7 + 6.0 * side / REFERENCE_SIDE;
            assert!(reach < side / 2.0, "side {side}: reach {reach}");
        }
    }
}
