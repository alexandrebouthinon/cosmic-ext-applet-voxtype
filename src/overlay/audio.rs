// SPDX-License-Identifier: MPL-2.0

//! Voxtype audio-level reader: parses the `audio.sock` frame stream and smooths it
//! for the overlay orb. See `src/voxtype.rs` for the sibling status reader.

use cosmic::iced::{Subscription, futures, stream};
use futures::SinkExt;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

/// Size of one frame on `audio.sock`: `seq: u32`, `min: f32`, `max: f32`, `peak_dbfs: f32`.
pub const FRAME_BYTES: usize = 16;

const MIN_BACKOFF: Duration = Duration::from_millis(200);
const MAX_BACKOFF: Duration = Duration::from_secs(5);

/// One audio-level sample published by Voxtype, at roughly 100 Hz.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub seq: u32,
    pub min: f32,
    pub max: f32,
    /// Peak level in dBFS, clamped at -120 dB by Voxtype.
    pub peak_dbfs: f32,
}

impl Frame {
    /// Parses a fixed-size frame in native byte order (Voxtype and the applet run
    /// as the same-architecture local processes, so no endianness conversion is needed).
    #[must_use]
    pub fn from_bytes(bytes: &[u8; FRAME_BYTES]) -> Self {
        Self {
            seq: u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            min: f32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            max: f32::from_ne_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            peak_dbfs: f32::from_ne_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        }
    }
}

/// Maps dBFS to 0..=1: -60 dB and below -> 0, 0 dB -> 1, linear in dB between. NaN -> 0.
#[must_use]
pub fn level_from_dbfs(dbfs: f32) -> f32 {
    if dbfs.is_nan() {
        return 0.0;
    }
    ((dbfs + 60.0) / 60.0).clamp(0.0, 1.0)
}

/// Attack/release smoothing, driven per render frame.
pub struct Smoother {
    value: f32,
    attack_per_s: f32,
    release_per_s: f32,
}

impl Smoother {
    #[must_use]
    pub fn new(attack_per_s: f32, release_per_s: f32) -> Self {
        Self {
            value: 0.0,
            attack_per_s,
            release_per_s,
        }
    }

    /// Moves toward `target` by an exponential step of rate attack (rising) or release
    /// (falling); returns the new value, always in 0..=1.
    pub fn update(&mut self, target: f32, dt: f32) -> f32 {
        // A non-finite step would poison `value` with NaN for good.
        if !dt.is_finite() || dt <= 0.0 {
            return self.value;
        }
        let rate = if target >= self.value {
            self.attack_per_s
        } else {
            self.release_per_s
        };
        let k = 1.0 - (-rate * dt).exp();
        self.value = (self.value + (target - self.value) * k).clamp(0.0, 1.0);
        self.value
    }

    #[must_use]
    pub fn value(&self) -> f32 {
        self.value
    }
}

/// Emits `level_from_dbfs(peak_dbfs)` for each frame read; emits 0.0 once whenever the
/// connection is lost.
pub fn subscription() -> Subscription<f32> {
    Subscription::run(|| {
        stream::channel(8, async |mut out: futures::channel::mpsc::Sender<f32>| {
            let mut backoff = MIN_BACKOFF;
            loop {
                // A relative value would resolve against the current directory.
                let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR")
                    .filter(|dir| std::path::Path::new(dir).is_absolute())
                else {
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                    continue;
                };
                let path = std::path::Path::new(&runtime_dir).join("voxtype/audio.sock");
                if let Ok(mut socket) = UnixStream::connect(&path).await {
                    let mut buf = [0u8; FRAME_BYTES];
                    while socket.read_exact(&mut buf).await.is_ok() {
                        backoff = MIN_BACKOFF;
                        let frame = Frame::from_bytes(&buf);
                        _ = out.send(level_from_dbfs(frame.peak_dbfs)).await;
                    }
                    _ = out.send(0.0).await;
                }
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_bytes(seq: u32, min: f32, max: f32, peak_dbfs: f32) -> [u8; FRAME_BYTES] {
        let mut bytes = [0u8; FRAME_BYTES];
        bytes[0..4].copy_from_slice(&seq.to_ne_bytes());
        bytes[4..8].copy_from_slice(&min.to_ne_bytes());
        bytes[8..12].copy_from_slice(&max.to_ne_bytes());
        bytes[12..16].copy_from_slice(&peak_dbfs.to_ne_bytes());
        bytes
    }

    #[test]
    fn frame_round_trips_bytes() {
        let bytes = frame_bytes(42, -0.5, 0.5, -12.3);
        let frame = Frame::from_bytes(&bytes);
        assert_eq!(
            frame,
            Frame {
                seq: 42,
                min: -0.5,
                max: 0.5,
                peak_dbfs: -12.3,
            }
        );
    }

    #[test]
    fn level_from_dbfs_maps_range_to_unit_interval() {
        assert!(level_from_dbfs(-120.0).abs() < f32::EPSILON);
        assert!(level_from_dbfs(-60.0).abs() < f32::EPSILON);
        assert!(level_from_dbfs(f32::NAN).abs() < f32::EPSILON);
        assert!((level_from_dbfs(0.0) - 1.0).abs() < f32::EPSILON);
        assert!((level_from_dbfs(6.0) - 1.0).abs() < f32::EPSILON);
        assert!((level_from_dbfs(-30.0) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn smoother_rises_faster_than_it_falls() {
        let mut rising = Smoother::new(25.0, 6.0);
        let risen = rising.update(1.0, 0.05);

        let mut falling = Smoother::new(25.0, 6.0);
        falling.update(1.0, 10.0); // saturate near the target first
        let after_fall = falling.update(0.0, 0.05);
        let fallen = 1.0 - after_fall;

        assert!(
            risen > fallen,
            "rise {risen} should outpace fall {fallen} over the same dt"
        );
    }

    #[test]
    fn smoother_stays_in_bounds_and_converges() {
        let mut smoother = Smoother::new(25.0, 6.0);
        for _ in 0..1000 {
            let value = smoother.update(1.0, 0.016);
            assert!((0.0..=1.0).contains(&value));
        }
        assert!((smoother.value() - 1.0).abs() < 1e-3);

        for _ in 0..1000 {
            let value = smoother.update(0.0, 0.016);
            assert!((0.0..=1.0).contains(&value));
        }
        assert!(smoother.value() < 1e-3);
    }
}
