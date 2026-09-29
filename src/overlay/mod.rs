// SPDX-License-Identifier: MPL-2.0

//! Overlay process: a click-through layer surface drawing the particle orb.
//! Runs as `cosmic-ext-applet-voxtype --overlay`, spawned by the applet.

pub mod audio;
pub mod particles;
pub mod view;

use crate::config::{Config, OverlayPosition};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced::platform_specific::shell::wayland::commands::layer_surface::{
    Anchor, KeyboardInteractivity, Layer, destroy_layer_surface, get_layer_surface,
};
use cosmic::iced::widget::canvas;
use cosmic::iced::{Color, Length, Subscription, window::Id};
use cosmic::prelude::*;
use particles::{Grain, Mode, Swarm};
use std::time::{Duration, Instant};

const PARTICLES: usize = 600;
const SEED: u64 = 0x5eed_0b1e;
// Slow enough that syllables blend into one swell instead of jolts.
const ATTACK_PER_S: f32 = 8.0;
const RELEASE_PER_S: f32 = 2.0;
const FADE_PER_S: f32 = 1.0 / 0.2;
const CONFIG_ID: &str = <crate::app::AppModel as cosmic::Application>::APP_ID;

pub struct Overlay {
    core: cosmic::Core,
    config: Config,
    /// Present from the start of the fade-in until the end of the fade-out.
    surface: Option<Id>,
    active: bool,
    fade: f32,
    /// The applet that spawned us; we exit once it is gone.
    parent: u32,
    started: Instant,
    last_tick: Instant,
    swarm: Swarm,
    grains: Vec<Grain>,
    mode: Mode,
    target_level: f32,
    level: audio::Smoother,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    Level(f32),
    Voxtype(crate::voxtype::Event),
    Config(Config),
    CheckParent,
}

impl cosmic::Application for Overlay {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "io.github.alexandrebouthinon.CosmicExtAppletVoxtype.Overlay";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(mut core: cosmic::Core, _flags: ()) -> (Self, Task<cosmic::Action<Message>>) {
        // Auto blur would frost the whole square surface; the orb draws its own veil instead.
        #[expect(
            clippy::default_trait_access,
            reason = "libcosmic does not re-export enumflags2"
        )]
        core.set_auto_blur(Default::default());
        let config = cosmic_config::Config::new(CONFIG_ID, Config::VERSION)
            .map(|context| Config::get_entry(&context).unwrap_or_else(|(_, config)| config))
            .unwrap_or_default();
        let now = Instant::now();
        let app = Self {
            core,
            config,
            surface: None,
            active: false,
            fade: 0.0,
            parent: std::os::unix::process::parent_id(),
            started: now,
            last_tick: now,
            swarm: Swarm::new(PARTICLES, SEED),
            grains: Vec::with_capacity(PARTICLES),
            mode: Mode::Listening,
            target_level: 0.0,
            level: audio::Smoother::new(ATTACK_PER_S, RELEASE_PER_S),
        };
        (app, Task::none())
    }

    fn view(&self) -> Element<'_, Message> {
        cosmic::widget::Space::new().into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        canvas(view::Orb {
            grains: &self.grains,
            mode: self.mode,
            fade: self.fade,
            level: self.level.value(),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            crate::voxtype::subscription().map(Message::Voxtype),
            self.core
                .watch_config::<Config>(CONFIG_ID)
                .map(|update| Message::Config(update.config)),
            cosmic::iced::time::every(Duration::from_secs(1)).map(|_| Message::CheckParent),
        ];
        // No render loop and no audio socket while hidden: the overlay idles at 0 % CPU.
        if self.surface.is_some() {
            subscriptions
                .push(cosmic::iced::time::every(Duration::from_millis(16)).map(Message::Tick));
            subscriptions.push(audio::subscription().map(Message::Level));
        }
        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::Tick(now) => {
                let dt = (now - self.last_tick).as_secs_f32();
                self.last_tick = now;
                return self.animate(now, dt);
            }
            Message::Level(level) => self.target_level = level,
            Message::Voxtype(crate::voxtype::Event::Status { state, .. }) => {
                if state == crate::voxtype::State::Transcribing {
                    self.mode = Mode::Transcribing;
                } else if state.is_active() {
                    self.mode = Mode::Listening;
                }
                self.active = state.is_active();
                if self.active && self.surface.is_none() {
                    return self.show();
                }
            }
            Message::Voxtype(crate::voxtype::Event::BinaryMissing) => self.active = false,
            Message::Config(config) => {
                let moved =
                    config.position != self.config.position || config.size != self.config.size;
                self.config = config;
                if moved && let Some(id) = self.surface.take() {
                    return destroy_layer_surface(id).chain(self.show());
                }
            }
            Message::CheckParent => {
                if std::os::unix::process::parent_id() != self.parent {
                    std::process::exit(0);
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::iced::theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
            icon_color: Color::WHITE,
        })
    }
}

impl Overlay {
    fn show(&mut self) -> Task<cosmic::Action<Message>> {
        let id = Id::unique();
        self.surface = Some(id);
        // The tick subscription restarts with the surface: avoid a huge first step.
        self.last_tick = Instant::now();
        let side = self.config.size.px();
        get_layer_surface(SctkLayerSurfaceSettings {
            id,
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            // An empty input region lets clicks reach the window below.
            input_zone: Some(Vec::new()),
            anchor: anchor(self.config.position),
            output: IcedOutput::Active,
            namespace: "voxtype-overlay".into(),
            // The orb already sits inside transparent padding.
            margin: IcedMargin::default(),
            size: Some((Some(side), Some(side))),
            exclusive_zone: -1,
            ..Default::default()
        })
    }

    fn animate(&mut self, now: Instant, dt: f32) -> Task<cosmic::Action<Message>> {
        let target = if self.active { 1.0 } else { 0.0 };
        self.fade = step_toward(self.fade, target, FADE_PER_S * dt);
        if !self.active && self.fade == 0.0 {
            self.target_level = 0.0;
            return self
                .surface
                .take()
                .map_or_else(Task::none, destroy_layer_surface);
        }
        let level = self.level.update(self.target_level, dt);
        self.swarm.step(dt, level, self.mode);
        let time = (now - self.started).as_secs_f32();
        #[expect(clippy::cast_precision_loss, reason = "surface side is at most 300")]
        let radius = view::radius(self.config.size.px() as f32);
        self.swarm
            .grains(radius, time, level, self.mode, &mut self.grains);
        Task::none()
    }
}

fn step_toward(value: f32, target: f32, max_step: f32) -> f32 {
    if value < target {
        (value + max_step).min(target)
    } else {
        (value - max_step).max(target)
    }
}

fn anchor(position: OverlayPosition) -> Anchor {
    match position {
        OverlayPosition::TopLeft => Anchor::TOP | Anchor::LEFT,
        OverlayPosition::TopRight => Anchor::TOP | Anchor::RIGHT,
        OverlayPosition::BottomLeft => Anchor::BOTTOM | Anchor::LEFT,
        OverlayPosition::BottomRight => Anchor::BOTTOM | Anchor::RIGHT,
        // Anchoring a single edge centres the surface along it.
        OverlayPosition::BottomCenter => Anchor::BOTTOM,
    }
}

#[cfg(test)]
mod tests {
    use super::step_toward;

    #[test]
    fn fade_steps_clamp_at_the_target() {
        assert!((step_toward(0.9, 1.0, 0.5) - 1.0).abs() < f32::EPSILON);
        assert!(step_toward(0.1, 0.0, 0.5).abs() < f32::EPSILON);
        assert!((step_toward(0.5, 1.0, 0.1) - 0.6).abs() < 1e-6);
    }
}
