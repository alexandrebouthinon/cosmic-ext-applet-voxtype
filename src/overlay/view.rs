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
    fn orb_fits_the_surface_at_full_swell() {
        // Grain centres stay within 2.7 radius (`Swarm::grains`); the largest grain
        // adds about 6 px at the reference side.
        for side in [180.0, 240.0, 300.0] {
            let reach = radius(side) * 2.7 + 6.0 * side / REFERENCE_SIDE;
            assert!(reach < side / 2.0, "side {side}: reach {reach}");
        }
    }
}
