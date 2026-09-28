// SPDX-License-Identifier: MPL-2.0

//! Deterministic particle simulation for the overlay orb.

use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Listening,
    Transcribing,
}

/// One projected particle, ready to draw. Coordinates are in logical px relative to the orb centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grain {
    pub x: f32,
    pub y: f32,
    /// 0 = back, 1 = front.
    pub depth: f32,
    /// Diameter in px.
    pub size: f32,
    /// Index into the 4-colour theme palette (0..4).
    pub color: u8,
    pub alpha: f32,
}

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    fn scale(self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

struct Particle {
    v: Vec3,
    w: Vec3,
    angle: f32,
    speed: f32,
    radius_jitter: f32,
    phase: f32,
    breath_frequency: f32,
    alpha_base: f32,
    color: u8,
    size_base: f32,
}

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        let [high, low, ..] = self.next_u64().to_be_bytes();
        f32::from(u16::from_be_bytes([high, low])) / 65_536.0
    }
}

pub struct Swarm {
    particles: Vec<Particle>,
    rotation: f32,
}

impl Swarm {
    #[must_use]
    pub fn new(count: usize, seed: u64) -> Self {
        let mut rng = Rng(seed);
        let mut particles = Vec::with_capacity(count);
        for _ in 0..count {
            let z = rng.next_f32().mul_add(2.0, -1.0);
            let longitude = rng.next_f32() * TAU;
            let equator = (1.0 - z * z).sqrt();
            let (sin_longitude, cos_longitude) = longitude.sin_cos();
            let v = Vec3 {
                x: equator * cos_longitude,
                y: equator * sin_longitude,
                z,
            };
            let reference = if z.abs() < 0.9 {
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }
            } else {
                Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }
            };
            let tangent = reference.cross(v);
            let tangent_length =
                (tangent.x * tangent.x + tangent.y * tangent.y + tangent.z * tangent.z).sqrt();
            let tangent = tangent.scale(1.0 / tangent_length);
            let other_tangent = v.cross(tangent);
            let axis_angle = rng.next_f32() * TAU;
            let (sin_axis, cos_axis) = axis_angle.sin_cos();
            let axis = tangent.scale(cos_axis).add(other_tangent.scale(sin_axis));
            let w = axis.cross(v);

            let orbital_speed =
                (0.15 + rng.next_f32() * 0.5) * if rng.next_f32() < 0.5 { -1.0 } else { 1.0 };
            let radius_jitter = 0.85 + rng.next_f32() * 0.3;
            let phase = rng.next_f32() * TAU;
            let breath_frequency = 1.0 + rng.next_f32() * 3.0;
            let alpha_base = 0.75 + rng.next_f32() * 0.25;
            let color_draw = rng.next_f32();
            let color = if color_draw < 0.35 {
                0
            } else if color_draw < 0.6 {
                1
            } else if color_draw < 0.85 {
                2
            } else {
                3
            };
            let size_draw = rng.next_f32();
            let size_base = if size_draw < 0.7 {
                0.35 + rng.next_f32() * 0.5
            } else if size_draw < 0.95 {
                0.8 + rng.next_f32() * 0.7
            } else {
                1.5 + rng.next_f32() * 0.9
            };
            particles.push(Particle {
                v,
                w,
                angle: 0.0,
                speed: orbital_speed,
                radius_jitter,
                phase,
                breath_frequency,
                alpha_base,
                color,
                size_base,
            });
        }
        Self {
            particles,
            rotation: 0.0,
        }
    }

    /// `level` is the smoothed voice level in 0..=1.
    pub fn step(&mut self, dt: f32, level: f32, mode: Mode) {
        let (speed_multiplier, rotation_speed) = match mode {
            Mode::Listening => (1.8 + level * 2.2, 0.45 + level * 0.9),
            Mode::Transcribing => (4.0, 2.6),
        };
        for particle in &mut self.particles {
            particle.angle += particle.speed * speed_multiplier * dt;
        }
        self.rotation += rotation_speed * dt;
    }

    /// `radius` is the orb radius in px (15 for a 96 px surface). Clears then fills `out`.
    pub fn grains(&self, radius: f32, time: f32, level: f32, mode: Mode, out: &mut Vec<Grain>) {
        out.clear();
        out.reserve(self.particles.len());
        let orb_radius = match mode {
            Mode::Listening => radius * (1.0 + level * 0.45),
            Mode::Transcribing => radius * 13.0 / 15.0,
        };
        let (sin_rotation, cos_rotation) = self.rotation.sin_cos();
        let (sin_tilt, cos_tilt) = 0.4_f32.sin_cos();
        for particle in &self.particles {
            let (sin_angle, cos_angle) = particle.angle.sin_cos();
            let position = particle.v.scale(cos_angle).add(particle.w.scale(sin_angle));
            let breath_phase = time.mul_add(particle.breath_frequency, particle.phase);
            let pulse = (time * particle.breath_frequency * 2.7 + particle.phase)
                .sin()
                .max(0.0);
            let breath = 1.0 + 0.09 * breath_phase.sin() + level * 0.5 * pulse;
            let position = position.scale(orb_radius * particle.radius_jitter * breath);
            let x = position.x * cos_rotation + position.z * sin_rotation;
            let z = -position.x * sin_rotation + position.z * cos_rotation;
            let y = position.y * cos_tilt - z * sin_tilt;
            let z = position.y * sin_tilt + z * cos_tilt;
            let depth = if orb_radius == 0.0 {
                0.5
            } else {
                f32::midpoint(z / orb_radius, 1.0).clamp(0.0, 1.0)
            };
            out.push(Grain {
                x,
                y,
                depth,
                size: particle.size_base * (0.55 + 0.75 * depth) * 2.6,
                color: particle.color,
                alpha: ((0.35 + 0.65 * depth) * particle.alpha_base).min(1.0),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Grain, Mode, Swarm};

    fn grains(swarm: &Swarm, level: f32, mode: Mode) -> Vec<Grain> {
        let mut out = Vec::new();
        swarm.grains(15.0, 1.0, level, mode, &mut out);
        out
    }

    fn mean_radius(grains: &[Grain]) -> f32 {
        let sum: f32 = grains.iter().map(|grain| grain.x.hypot(grain.y)).sum();
        sum / f32::from(u16::try_from(grains.len()).expect("test swarm fits in u16"))
    }

    #[test]
    fn same_seed_produces_same_grains_after_step() {
        let mut first = Swarm::new(600, 42);
        let mut second = Swarm::new(600, 42);
        first.step(0.016, 0.5, Mode::Listening);
        second.step(0.016, 0.5, Mode::Listening);
        assert_eq!(
            grains(&first, 0.5, Mode::Listening),
            grains(&second, 0.5, Mode::Listening)
        );
    }

    #[test]
    fn produces_bounded_grains_and_reuses_output() {
        let swarm = Swarm::new(600, 42);
        let mut out = Vec::with_capacity(600);
        let buffer = out.as_ptr();
        for mode in [Mode::Listening, Mode::Transcribing] {
            for level in [0.0, 1.0] {
                swarm.grains(15.0, 1.0, level, mode, &mut out);
                assert_eq!(out.len(), 600);
                assert_eq!(out.as_ptr(), buffer);
                for grain in &out {
                    assert!(grain.x.is_finite() && grain.y.is_finite());
                    assert!(grain.x.abs() <= 15.0 * 2.7);
                    assert!(grain.y.abs() <= 15.0 * 2.7);
                    assert!(grain.color < 4);
                    assert!((0.0..=1.0).contains(&grain.depth));
                    assert!((0.0..=1.0).contains(&grain.alpha));
                }
            }
        }
    }

    #[test]
    fn transcribing_contracts_and_voice_expands_the_orb() {
        let swarm = Swarm::new(600, 42);
        assert!(
            mean_radius(&grains(&swarm, 0.5, Mode::Transcribing))
                < mean_radius(&grains(&swarm, 0.5, Mode::Listening))
        );
        assert!(
            mean_radius(&grains(&swarm, 1.0, Mode::Listening))
                > mean_radius(&grains(&swarm, 0.0, Mode::Listening))
        );
    }
}
