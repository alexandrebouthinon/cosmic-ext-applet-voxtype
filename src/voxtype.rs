// SPDX-License-Identifier: MPL-2.0

//! Voxtype daemon state, read only through its CLI (`voxtype status --follow`).

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

/// Streams Voxtype state changes, restarting the follower with backoff when it exits.
pub fn subscription() -> Subscription<Event> {
    Subscription::run(|| {
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
}
