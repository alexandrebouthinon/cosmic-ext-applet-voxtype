// SPDX-License-Identifier: MPL-2.0

//! Voxtype daemon state and control, only through its CLI and `systemctl --user`.

use cosmic::iced::{Subscription, futures, stream};
use futures::SinkExt;
use serde::Deserialize;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

const BIN: &str = "voxtype";
const MAX_BACKOFF: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum State {
    Idle,
    Recording,
    Transcribing,
    /// The daemon is not running (`voxtype.service` stopped or crashed).
    #[default]
    Stopped,
}

impl State {
    /// Whether the overlay should be on screen.
    #[must_use]
    pub fn is_active(self) -> bool {
        matches!(self, Self::Recording | Self::Transcribing)
    }
}

/// Extended info shown in the popup; `None` when Voxtype does not report it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Info {
    pub model: Option<String>,
    pub device: Option<String>,
    pub backend: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Status {
        state: State,
        info: Info,
    },
    /// The `voxtype` binary is not in `PATH`: the applet hides itself.
    BinaryMissing,
}

#[derive(Deserialize)]
struct RawStatus {
    alt: String,
    model: Option<String>,
    device: Option<String>,
    backend: Option<String>,
}

/// Parses one JSON line of `voxtype status --format json --extended`.
/// Unknown states are ignored so a newer Voxtype cannot break the applet.
#[must_use]
pub fn parse_status_line(line: &str) -> Option<(State, Info)> {
    let raw: RawStatus = serde_json::from_str(line).ok()?;
    let state = match raw.alt.as_str() {
        "idle" => State::Idle,
        // Streaming is live dictation: same feedback as recording.
        "recording" | "streaming" => State::Recording,
        "transcribing" => State::Transcribing,
        "stopped" => State::Stopped,
        _ => return None,
    };
    let info = Info {
        model: raw.model,
        device: raw.device,
        backend: raw.backend,
    };
    Some((state, info))
}

/// Health of the `voxtype.service` user unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Daemon {
    Active,
    /// Stopped on purpose (applet, terminal, or never started).
    Inactive,
    /// Crashed or failed to start: the user needs `journalctl`.
    Failed,
    /// No such unit: `voxtype setup systemd` was never run.
    Missing,
}

/// Parses `systemctl --user show voxtype -p LoadState -p ActiveState`.
#[must_use]
pub fn parse_daemon(show: &str) -> Daemon {
    let prop = |key: &str| {
        show.lines()
            .find_map(|l| l.strip_prefix(key)?.strip_prefix('='))
            .unwrap_or_default()
    };
    if prop("LoadState") == "not-found" {
        return Daemon::Missing;
    }
    match prop("ActiveState") {
        "active" | "activating" | "reloading" => Daemon::Active,
        "failed" => Daemon::Failed,
        _ => Daemon::Inactive,
    }
}

/// An audio device as listed by `voxtype info devices --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Device {
    pub name: String,
    pub default: bool,
}

/// Runs a command and returns its stdout, or its stderr as the error to show the user.
async fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| format!("{program}: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

pub async fn daemon() -> Result<Daemon, String> {
    let show = run(
        "systemctl",
        &[
            "--user",
            "show",
            "voxtype",
            "-p",
            "LoadState",
            "-p",
            "ActiveState",
        ],
    )
    .await?;
    Ok(parse_daemon(&show))
}

pub async fn set_daemon(running: bool) -> Result<(), String> {
    let verb = if running { "start" } else { "stop" };
    run("systemctl", &["--user", verb, "voxtype"])
        .await
        .map(drop)
}

#[expect(dead_code, reason = "wired by the microphone task")]
pub async fn restart_daemon() -> Result<(), String> {
    run("systemctl", &["--user", "restart", "voxtype"])
        .await
        .map(drop)
}

#[expect(dead_code, reason = "wired by the microphone task")]
pub async fn config_get(key: &str) -> Result<String, String> {
    run(BIN, &["config", "get", key]).await
}

#[expect(dead_code, reason = "wired by the microphone task")]
pub async fn config_set(key: &str, value: &str) -> Result<(), String> {
    run(BIN, &["config", "set", key, value]).await.map(drop)
}

/// All ALSA devices Voxtype sees, outputs included: callers filter them.
#[expect(dead_code, reason = "wired by the microphone task")]
pub async fn devices() -> Result<Vec<Device>, String> {
    let json = run(BIN, &["info", "devices", "--json"]).await?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

/// Streams Voxtype state changes, restarting the follower with backoff when it exits.
pub fn subscription() -> Subscription<Event> {
    subscription_with_generation(0)
}

/// Restarts the follower when the applet changes generation after starting the service.
pub fn subscription_with_generation(generation: u64) -> Subscription<Event> {
    Subscription::run_with(generation, |_| {
        stream::channel(8, async |mut out: futures::channel::mpsc::Sender<Event>| {
            let mut backoff = Duration::from_secs(1);
            loop {
                let child = Command::new(BIN)
                    .args(["status", "--follow", "--format", "json", "--extended"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .kill_on_drop(true)
                    .spawn();
                match child {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        _ = out.send(Event::BinaryMissing).await;
                    }
                    Err(_) => {}
                    Ok(mut child) => {
                        if let Some(stdout) = child.stdout.take() {
                            let mut lines = BufReader::new(stdout).lines();
                            while let Ok(Some(line)) = lines.next_line().await {
                                if let Some((state, info)) = parse_status_line(&line) {
                                    backoff = Duration::from_secs(1);
                                    _ = out.send(Event::Status { state, info }).await;
                                }
                            }
                        }
                        _ = child.wait().await;
                        _ = out
                            .send(Event::Status {
                                state: State::Stopped,
                                info: Info::default(),
                            })
                            .await;
                    }
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

    #[test]
    fn parses_extended_idle() {
        let line = r#"{"text": "x", "alt": "idle", "class": "idle", "tooltip": "t", "model": "parakeet", "device": "default", "backend": "ONNX GPU"}"#;
        let (state, info) = parse_status_line(line).unwrap();
        assert_eq!(state, State::Idle);
        assert_eq!(info.model.as_deref(), Some("parakeet"));
        assert_eq!(info.device.as_deref(), Some("default"));
        assert_eq!(info.backend.as_deref(), Some("ONNX GPU"));
    }

    #[test]
    fn maps_all_known_states() {
        for (alt, want) in [
            ("recording", State::Recording),
            ("streaming", State::Recording),
            ("transcribing", State::Transcribing),
            ("stopped", State::Stopped),
        ] {
            let line =
                format!(r#"{{"text": "", "alt": "{alt}", "class": "{alt}", "tooltip": ""}}"#);
            assert_eq!(parse_status_line(&line).unwrap().0, want, "{alt}");
        }
    }

    #[test]
    fn ignores_unknown_state_and_garbage() {
        assert!(
            parse_status_line(r#"{"text": "", "alt": "bogus", "class": "", "tooltip": ""}"#)
                .is_none()
        );
        assert!(parse_status_line("not json").is_none());
    }

    #[test]
    fn only_recording_and_transcribing_are_active() {
        assert!(State::Recording.is_active());
        assert!(State::Transcribing.is_active());
        assert!(!State::Idle.is_active());
        assert!(!State::Stopped.is_active());
    }

    #[test]
    fn maps_unit_states() {
        for (show, want) in [
            ("LoadState=loaded\nActiveState=active", Daemon::Active),
            ("LoadState=loaded\nActiveState=activating", Daemon::Active),
            ("LoadState=loaded\nActiveState=inactive", Daemon::Inactive),
            ("LoadState=loaded\nActiveState=failed", Daemon::Failed),
            ("LoadState=not-found\nActiveState=inactive", Daemon::Missing),
        ] {
            assert_eq!(parse_daemon(show), want, "{show}");
        }
    }

    #[test]
    fn parses_device_list() {
        let json =
            r#"[{"name": "default", "default": true}, {"name": "pipewire", "default": false}]"#;
        let devices: Vec<Device> = serde_json::from_str(json).unwrap();
        assert_eq!(devices.len(), 2);
        assert!(devices[0].default);
        assert_eq!(devices[1].name, "pipewire");
    }
}
