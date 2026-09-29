// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, OverlayPosition, OverlaySize};
use crate::fl;
use crate::voxtype;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{Limits, Subscription, window::Id};
use cosmic::prelude::*;
use cosmic::widget::{self, segmented_button};

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Last known Voxtype state; `Stopped` until the first status line arrives.
    state: voxtype::State,
    info: voxtype::Info,
    /// The `voxtype` binary is not installed: shown like a stopped daemon.
    binary_missing: bool,
    /// The `--overlay` child process, running while the overlay is enabled.
    overlay: Option<OverlayChild>,
    /// Popup tabs; each entry carries its `Tab` as data.
    tabs: segmented_button::SingleSelectModel,
    /// Orb position dropdown labels; built once in `init` so they outlive the view.
    position_labels: Vec<String>,
    /// Orb size segmented control; each entry carries its `OverlaySize` as data.
    orb_size: segmented_button::SingleSelectModel,
    /// `None` until the first `systemctl` query answers.
    daemon: Option<voxtype::Daemon>,
    /// Last command failure, shown under the popup header.
    error: Option<String>,
    /// Confirmation shown after a successful stop until the popup is reopened.
    stop_notice: bool,
    daemon_pending: bool,
    status_generation: u64,
}

/// Orb positions in the order shown by the position dropdown.
const POSITIONS: [OverlayPosition; 5] = [
    OverlayPosition::TopLeft,
    OverlayPosition::TopRight,
    OverlayPosition::BottomLeft,
    OverlayPosition::BottomRight,
    OverlayPosition::BottomCenter,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Status,
    Micro,
    Orb,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Voxtype(voxtype::Event),
    UpdateConfig(Config),
    ToggleOverlay(bool),
    SetPosition(OverlayPosition),
    SetSize(OverlaySize),
    SelectTab(segmented_button::Entity),
    Daemon(Result<voxtype::Daemon, String>),
    ToggleDaemon(bool),
    DaemonSet(bool, Result<(), String>),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "io.github.alexandrebouthinon.CosmicExtAppletVoxtype";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => {
                    // for why in errors {
                    //     tracing::error!(%why, "error loading app config");
                    // }

                    config
                }
            })
            .unwrap_or_default();

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            tabs: segmented_button::Model::builder()
                .insert(|b| b.text(fl!("tab-status")).data(Tab::Status).activate())
                .insert(|b| b.text(fl!("tab-micro")).data(Tab::Micro))
                .insert(|b| b.text(fl!("tab-orb")).data(Tab::Orb))
                .build(),
            position_labels: vec![
                fl!("position-top-left"),
                fl!("position-top-right"),
                fl!("position-bottom-left"),
                fl!("position-bottom-right"),
                fl!("position-bottom-center"),
            ],
            orb_size: segmented_button::Model::builder()
                .insert(|b| {
                    let b = b.text(fl!("size-small")).data(OverlaySize::Small);
                    if config.size == OverlaySize::Small {
                        b.activate()
                    } else {
                        b
                    }
                })
                .insert(|b| {
                    let b = b.text(fl!("size-medium")).data(OverlaySize::Medium);
                    if config.size == OverlaySize::Medium {
                        b.activate()
                    } else {
                        b
                    }
                })
                .insert(|b| {
                    let b = b.text(fl!("size-large")).data(OverlaySize::Large);
                    if config.size == OverlaySize::Large {
                        b.activate()
                    } else {
                        b
                    }
                })
                .build(),
            config,
            ..Default::default()
        };
        app.sync_overlay();

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// The applet's button in the panel will be drawn using the main view method.
    /// This view should emit messages to toggle the applet's popup window, which will
    /// be drawn using the `view_window` method.
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            // Still shown without the binary, so the user can tell the applet itself runs.
            .icon_button(icon_name(if self.binary_missing {
                voxtype::State::Stopped
            } else {
                self.state
            }))
            .on_press(Message::TogglePopup)
            .into()
    }

    /// The applet's popup window will be drawn using this view method. If there are
    /// multiple poups, you may match the id parameter to determine which popup to
    /// create a view for.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let spacing = cosmic::theme::spacing();
        let tab = match self.tabs.active_data::<Tab>() {
            Some(Tab::Micro) => self.micro_tab(),
            Some(Tab::Orb) => self.orb_tab(),
            _ => self.status_tab(),
        };
        let content = widget::Column::new()
            .push(self.header())
            .push_maybe(
                matches!(self.daemon, None | Some(voxtype::Daemon::Missing))
                    .then(|| widget::text::caption(fl!("daemon-missing"))),
            )
            .push_maybe(
                (self.daemon == Some(voxtype::Daemon::Failed))
                    .then(|| widget::text::caption(fl!("daemon-failed-hint"))),
            )
            .push_maybe(
                self.stop_notice
                    .then(|| widget::text::caption(fl!("daemon-stopped-notice"))),
            )
            .push_maybe(self.error.as_deref().map(widget::text::caption))
            .push(widget::segmented_control::horizontal(&self.tabs).on_activate(Message::SelectTab))
            .push(tab)
            .spacing(spacing.space_s)
            .padding(spacing.space_s);

        self.core.applet.popup_container(content).into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-lived async tasks running in the background which
    /// emit messages to the application through a channel. They may be conditionally
    /// activated by selectively appending to the subscription batch, and will
    /// continue to execute for the duration that they remain in the batch.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            voxtype::subscription_with_generation(self.status_generation).map(Message::Voxtype),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime. The application will not exit until all
    /// tasks are finished.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Voxtype(voxtype::Event::Status { state, info }) => {
                let stopped_changed =
                    (self.state == voxtype::State::Stopped) != (state == voxtype::State::Stopped);
                self.binary_missing = false;
                self.state = state;
                // A stopped daemon reports no info; keep the last known values for the popup.
                if state != voxtype::State::Stopped {
                    self.info = info;
                }
                if stopped_changed {
                    return Self::daemon_query();
                }
            }
            Message::Voxtype(voxtype::Event::BinaryMissing) => self.binary_missing = true,
            Message::UpdateConfig(config) => {
                self.config = config;
                self.sync_overlay();
                self.sync_orb_size();
            }
            Message::ToggleOverlay(enabled) => {
                if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    _ = self.config.set_show_overlay(&ctx, enabled);
                }
            }
            Message::SetPosition(position) => {
                if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    _ = self.config.set_position(&ctx, position);
                }
            }
            Message::SetSize(size) => {
                if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    _ = self.config.set_size(&ctx, size);
                }
                self.sync_orb_size();
            }
            Message::SelectTab(id) => self.tabs.activate(id),
            Message::Daemon(Ok(daemon)) => self.daemon = Some(daemon),
            Message::Daemon(Err(e)) => self.error = Some(e),
            message @ (Message::ToggleDaemon(_) | Message::DaemonSet(..)) => {
                return self.update_daemon(message);
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    self.error = None;
                    self.stop_notice = false;
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    Task::batch([get_popup(popup_settings), Self::daemon_query()])
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl AppModel {
    fn update_daemon(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::ToggleDaemon(running) => {
                if self.daemon_pending
                    || !matches!(
                        self.daemon,
                        Some(
                            voxtype::Daemon::Active
                                | voxtype::Daemon::Inactive
                                | voxtype::Daemon::Failed
                        )
                    )
                {
                    return Task::none();
                }
                self.daemon_pending = true;
                self.error = None;
                self.stop_notice = false;
                return cosmic::task::future(async move {
                    cosmic::Action::App(Message::DaemonSet(
                        running,
                        voxtype::set_daemon(running).await,
                    ))
                });
            }
            Message::DaemonSet(running, result) => {
                self.daemon_pending = false;
                match result {
                    Ok(()) => {
                        if running {
                            self.status_generation = self.status_generation.wrapping_add(1);
                        } else {
                            self.stop_notice = true;
                        }
                    }
                    Err(e) => self.error = Some(e),
                }
                return Self::daemon_query();
            }
            _ => {}
        }
        Task::none()
    }

    fn daemon_query() -> Task<cosmic::Action<Message>> {
        cosmic::task::future(async {
            cosmic::Action::App(Message::Daemon(voxtype::daemon().await))
        })
    }

    /// Fixed popup header: state glyph, name, state text and daemon toggler.
    fn header(&self) -> Element<'_, Message> {
        let state = if self.binary_missing {
            voxtype::State::Stopped
        } else {
            self.state
        };
        let status = match (state, self.daemon) {
            (_, Some(voxtype::Daemon::Failed)) => fl!("state-failed"),
            (_, Some(voxtype::Daemon::Inactive | voxtype::Daemon::Missing))
            | (voxtype::State::Stopped, _) => fl!("state-stopped"),
            (voxtype::State::Idle, _) => fl!("state-idle"),
            (voxtype::State::Recording, _) => fl!("state-recording"),
            (voxtype::State::Transcribing, _) => fl!("state-transcribing"),
        };
        let running = self.daemon == Some(voxtype::Daemon::Active);
        let toggle = widget::toggler(running).on_toggle_maybe(
            (!self.daemon_pending
                && matches!(
                    self.daemon,
                    Some(
                        voxtype::Daemon::Active
                            | voxtype::Daemon::Inactive
                            | voxtype::Daemon::Failed
                    )
                ))
            .then_some(Message::ToggleDaemon),
        );
        widget::Row::new()
            .push(widget::icon::from_name(icon_name(state)).size(24))
            .push(
                widget::Column::new()
                    .push(widget::text::heading("Voxtype"))
                    .push(widget::text::caption(status))
                    .width(cosmic::iced::Length::Fill),
            )
            .push(toggle)
            .align_y(cosmic::iced::Alignment::Center)
            .spacing(cosmic::theme::spacing().space_s)
            .into()
    }

    fn status_tab(&self) -> Element<'_, Message> {
        // The daemon reports no info while stopped; a missing field while running is also shown as unavailable.
        let unavailable = self.binary_missing || self.state == voxtype::State::Stopped;
        let value = |field: &Option<String>| {
            if unavailable {
                fl!("unavailable")
            } else {
                field.clone().unwrap_or_else(|| fl!("unavailable"))
            }
        };
        widget::list_column()
            .add(widget::settings::item(
                fl!("info-model"),
                widget::text::body(value(&self.info.model)),
            ))
            .add(widget::settings::item(
                fl!("info-backend"),
                widget::text::body(value(&self.info.backend)),
            ))
            .into()
    }

    #[expect(clippy::unused_self, reason = "filled by the microphone task")]
    fn micro_tab(&self) -> Element<'_, Message> {
        widget::Column::new().into()
    }

    fn orb_tab(&self) -> Element<'_, Message> {
        let position_selected = POSITIONS.iter().position(|p| *p == self.config.position);
        // `on_activate` requires a `'static` closure, so the entity -> size mapping is
        // collected up front instead of borrowing `self.orb_size` from inside it.
        let size_by_entity: Vec<(segmented_button::Entity, OverlaySize)> = self
            .orb_size
            .iter()
            .filter_map(|id| {
                self.orb_size
                    .data::<OverlaySize>(id)
                    .map(|size| (id, *size))
            })
            .collect();
        widget::list_column()
            .add(widget::settings::item(
                fl!("show-overlay"),
                widget::toggler(self.config.show_overlay).on_toggle(Message::ToggleOverlay),
            ))
            .add(widget::settings::item(
                fl!("orb-position"),
                widget::dropdown(self.position_labels.as_slice(), position_selected, |i| {
                    Message::SetPosition(POSITIONS.get(i).copied().unwrap_or_default())
                }),
            ))
            // Full width under its label: squeezed beside it, the active item's check mark
            // hides the neighbouring labels.
            .add(
                widget::Column::new()
                    .push(widget::text::body(fl!("orb-size")))
                    .push(
                        widget::segmented_control::horizontal(&self.orb_size)
                            .on_activate(move |entity| {
                                let size = size_by_entity
                                    .iter()
                                    .find(|(id, _)| *id == entity)
                                    .map_or_else(OverlaySize::default, |(_, size)| *size);
                                Message::SetSize(size)
                            })
                            .width(cosmic::iced::Length::Fill),
                    )
                    .spacing(cosmic::theme::spacing().space_xxs),
            )
            .into()
    }

    /// Runs the overlay child exactly while it is enabled, respawning it if it died.
    fn sync_overlay(&mut self) {
        if !self.config.show_overlay {
            self.overlay = None;
            return;
        }
        let running = self
            .overlay
            .as_mut()
            .is_some_and(|OverlayChild(child)| matches!(child.try_wait(), Ok(None)));
        if !running {
            self.overlay = std::env::current_exe()
                .and_then(|exe| std::process::Command::new(exe).arg("--overlay").spawn())
                .ok()
                .map(OverlayChild);
        }
    }

    /// Keeps the size segmented control's selection in sync with the config,
    /// including changes made externally (e.g. another applet instance).
    fn sync_orb_size(&mut self) {
        let entity = self
            .orb_size
            .iter()
            .find(|&id| self.orb_size.data::<OverlaySize>(id) == Some(&self.config.size));
        if let Some(id) = entity {
            self.orb_size.activate(id);
        }
    }
}

fn icon_name(state: voxtype::State) -> &'static str {
    match state {
        voxtype::State::Idle => "audio-input-microphone-symbolic",
        voxtype::State::Recording => "microphone-sensitivity-high-symbolic",
        voxtype::State::Transcribing => "document-edit-symbolic",
        voxtype::State::Stopped => "microphone-sensitivity-muted-symbolic",
    }
}

/// Kills the overlay process when dropped, so it never outlives the applet's intent.
struct OverlayChild(std::process::Child);

impl Drop for OverlayChild {
    fn drop(&mut self) {
        _ = self.0.kill();
        // Reap it so no zombie is left behind.
        _ = self.0.wait();
    }
}
