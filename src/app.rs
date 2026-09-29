// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
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
    /// `None` until the first `systemctl` query answers.
    daemon: Option<voxtype::Daemon>,
    /// Last command failure, shown under the popup header.
    error: Option<String>,
}

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
    SelectTab(segmented_button::Entity),
    Daemon(Result<voxtype::Daemon, String>),
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
        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            tabs: segmented_button::Model::builder()
                .insert(|b| b.text(fl!("tab-status")).data(Tab::Status).activate())
                .insert(|b| b.text(fl!("tab-micro")).data(Tab::Micro))
                .insert(|b| b.text(fl!("tab-orb")).data(Tab::Orb))
                .build(),
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
            voxtype::subscription().map(Message::Voxtype),
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
                self.binary_missing = false;
                self.state = state;
                // A stopped daemon reports no info; keep the last known values for the popup.
                if state != voxtype::State::Stopped {
                    self.info = info;
                }
            }
            Message::Voxtype(voxtype::Event::BinaryMissing) => self.binary_missing = true,
            Message::UpdateConfig(config) => {
                self.config = config;
                self.sync_overlay();
            }
            Message::ToggleOverlay(enabled) => {
                if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    _ = self.config.set_show_overlay(&ctx, enabled);
                }
            }
            Message::SelectTab(id) => self.tabs.activate(id),
            Message::Daemon(Ok(daemon)) => self.daemon = Some(daemon),
            Message::Daemon(Err(e)) => self.error = Some(e),
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    self.error = None;
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
                    Task::batch([
                        get_popup(popup_settings),
                        cosmic::task::future(async {
                            cosmic::Action::App(Message::Daemon(voxtype::daemon().await))
                        }),
                    ])
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
    /// Fixed popup header: state glyph, name, state text and daemon toggler.
    fn header(&self) -> Element<'_, Message> {
        let state = if self.binary_missing {
            voxtype::State::Stopped
        } else {
            self.state
        };
        let status = match (state, self.daemon) {
            (voxtype::State::Idle, _) => fl!("state-idle"),
            (voxtype::State::Recording, _) => fl!("state-recording"),
            (voxtype::State::Transcribing, _) => fl!("state-transcribing"),
            (voxtype::State::Stopped, Some(voxtype::Daemon::Failed)) => fl!("state-failed"),
            (voxtype::State::Stopped, _) => fl!("state-stopped"),
        };
        // Read-only until the daemon toggler task wires start/stop.
        let running = self.daemon == Some(voxtype::Daemon::Active);
        widget::Row::new()
            .push(widget::icon::from_name(icon_name(state)).size(24))
            .push(
                widget::Column::new()
                    .push(widget::text::heading("Voxtype"))
                    .push(widget::text::caption(status))
                    .width(cosmic::iced::Length::Fill),
            )
            .push(widget::toggler(running))
            .align_y(cosmic::iced::Alignment::Center)
            .spacing(cosmic::theme::spacing().space_s)
            .into()
    }

    #[expect(clippy::unused_self, reason = "filled by the status tab task")]
    fn status_tab(&self) -> Element<'_, Message> {
        widget::Column::new().into()
    }

    #[expect(clippy::unused_self, reason = "filled by the microphone task")]
    fn micro_tab(&self) -> Element<'_, Message> {
        widget::Column::new().into()
    }

    fn orb_tab(&self) -> Element<'_, Message> {
        widget::list_column()
            .add(widget::settings::item(
                fl!("show-overlay"),
                widget::toggler(self.config.show_overlay).on_toggle(Message::ToggleOverlay),
            ))
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
