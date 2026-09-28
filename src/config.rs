// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
    BottomCenter,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlaySize {
    Small,
    #[default]
    Medium,
    Large,
}

impl OverlaySize {
    /// Side of the square overlay surface, in logical pixels.
    #[must_use]
    #[expect(
        dead_code,
        reason = "contract for the overlay surface (sprint orbe, lot 1)"
    )]
    pub fn px(self) -> u32 {
        match self {
            Self::Small => 72,
            Self::Medium => 96,
            Self::Large => 128,
        }
    }
}

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    pub show_overlay: bool,
    pub position: OverlayPosition,
    pub size: OverlaySize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_overlay: true,
            position: OverlayPosition::default(),
            size: OverlaySize::default(),
        }
    }
}
