// SPDX-License-Identifier: MPL-2.0

//! Overlay process: a click-through layer surface drawing the particle orb.
//! Runs as `cosmic-ext-applet-voxtype --overlay`, spawned by the applet.

use cosmic::iced::platform_specific::runtime::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced::platform_specific::shell::wayland::commands::layer_surface::{
    Anchor, KeyboardInteractivity, Layer, get_layer_surface,
};
use cosmic::iced::widget::canvas;
use cosmic::iced::{Color, Length, Point, Rectangle, Renderer, Subscription, mouse, window::Id};
use cosmic::prelude::*;
use std::time::{Duration, Instant};

const SIZE: u32 = 96;
const MARGIN: i32 = 16;

pub struct Overlay {
    core: cosmic::Core,
    started: Instant,
    t: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
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

    fn init(core: cosmic::Core, _flags: ()) -> (Self, Task<cosmic::Action<Message>>) {
        let settings = SctkLayerSurfaceSettings {
            id: Id::unique(),
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            // An empty input region lets clicks reach the window below.
            input_zone: Some(Vec::new()),
            anchor: Anchor::BOTTOM | Anchor::RIGHT,
            output: IcedOutput::Active,
            namespace: "voxtype-overlay".into(),
            margin: IcedMargin {
                top: 0,
                right: MARGIN,
                bottom: MARGIN,
                left: 0,
            },
            size: Some((Some(SIZE), Some(SIZE))),
            exclusive_zone: -1,
            ..Default::default()
        };
        let app = Self {
            core,
            started: Instant::now(),
            t: 0.0,
        };
        (app, get_layer_surface(settings))
    }

    fn view(&self) -> Element<'_, Message> {
        cosmic::widget::Space::new().into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        canvas(Orb { t: self.t })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        cosmic::iced::time::every(Duration::from_millis(16)).map(Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::Tick(now) => self.t = (now - self.started).as_secs_f32(),
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

/// Spike-only drawing: dots on a rotating ring, to check transparency and frame pacing.
struct Orb {
    t: f32,
}

impl<M> canvas::Program<M, cosmic::Theme> for Orb {
    type State = ();

    #[expect(
        clippy::many_single_char_names,
        reason = "spike drawing, replaced by view.rs"
    )]
    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &cosmic::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let c = frame.center();
        for i in 0..600u16 {
            let k = f32::from(i);
            let a = self.t * (0.3 + (k * 0.618).fract()) + k * 2.399;
            let r = 14.0 + 6.0 * (k * 0.37).fract();
            let depth = f32::midpoint(a.sin(), 1.0);
            let p = Point::new(c.x + a.cos() * r, c.y + (k * 0.11).sin() * r * 0.9);
            let color = Color::from_rgba(0.75, 0.43, 0.93, 0.2 + 0.8 * depth);
            let d = 0.8 + 2.0 * depth;
            frame.fill_rectangle(
                Point::new(p.x - d / 2.0, p.y - d / 2.0),
                cosmic::iced::Size::new(d, d),
                color,
            );
        }
        vec![frame.into_geometry()]
    }
}
