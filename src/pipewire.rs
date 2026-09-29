// SPDX-License-Identifier: MPL-2.0

//! Microphones as `PipeWire` sees them: Voxtype only lists raw ALSA PCMs, so the applet
//! picks the `PipeWire` default source and leaves Voxtype on `default`.

use serde_json::Value;

/// A capture node, virtual ones (Easy Effects) included, monitors excluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sources {
    pub list: Vec<Source>,
    /// `node.name` of the current default source.
    pub default: Option<String>,
}

/// Parses the output of `pw-dump`.
pub fn parse_dump(json: &str) -> Result<Sources, String> {
    let objects: Vec<Value> = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let mut sources = Sources::default();
    for object in &objects {
        let props = &object["info"]["props"];
        if props["media.class"]
            .as_str()
            .is_some_and(|class| class.starts_with("Audio/Source"))
        {
            let Some(name) = props["node.name"].as_str() else {
                continue;
            };
            let label = ["node.description", "node.nick"]
                .iter()
                .find_map(|key| props[*key].as_str())
                .unwrap_or(name);
            sources.list.push(Source {
                name: name.to_owned(),
                label: label.to_owned(),
            });
        }
        if object["props"]["metadata.name"] == "default" {
            sources.default = object["metadata"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|entry| entry["key"] == "default.audio.source")
                .and_then(|entry| entry["value"]["name"].as_str())
                .map(str::to_owned);
        }
    }
    Ok(sources)
}

pub async fn sources() -> Result<Sources, String> {
    parse_dump(&crate::voxtype::run("pw-dump", &[]).await?)
}

/// Changes the microphone of every app following the default, not only Voxtype's.
pub async fn set_default_source(name: &str) -> Result<(), String> {
    crate::voxtype::run("pactl", &["set-default-source", name])
        .await
        .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_real_and_virtual_sources_with_their_default() {
        let json = r#"[
            {"id": 1, "type": "PipeWire:Interface:Node", "info": {"props": {
                "media.class": "Audio/Source",
                "node.name": "alsa_input.usb-Blue",
                "node.description": "Blue Microphones Stéréo numérique (IEC958)"}}},
            {"id": 2, "type": "PipeWire:Interface:Node", "info": {"props": {
                "media.class": "Audio/Source/Virtual",
                "node.name": "easyeffects_source",
                "node.description": "Easy Effects Source"}}},
            {"id": 3, "type": "PipeWire:Interface:Node", "info": {"props": {
                "media.class": "Audio/Sink",
                "node.name": "alsa_output.hdmi",
                "node.description": "HDMI"}}},
            {"id": 4, "type": "PipeWire:Interface:Node", "info": {"props": {
                "media.class": "Stream/Input/Audio",
                "node.name": "voxtype"}}},
            {"id": 5, "type": "PipeWire:Interface:Metadata", "props": {"metadata.name": "default"},
             "metadata": [
                {"subject": 0, "key": "default.audio.sink", "value": {"name": "alsa_output.hdmi"}},
                {"subject": 0, "key": "default.audio.source", "value": {"name": "easyeffects_source"}}]}
        ]"#;
        let sources = parse_dump(json).unwrap();
        assert_eq!(
            sources.list,
            [
                Source {
                    name: "alsa_input.usb-Blue".into(),
                    label: "Blue Microphones Stéréo numérique (IEC958)".into(),
                },
                Source {
                    name: "easyeffects_source".into(),
                    label: "Easy Effects Source".into(),
                },
            ]
        );
        assert_eq!(sources.default.as_deref(), Some("easyeffects_source"));
    }

    #[test]
    fn falls_back_to_nick_then_name() {
        let json = r#"[
            {"info": {"props": {"media.class": "Audio/Source", "node.name": "a", "node.nick": "Nick"}}},
            {"info": {"props": {"media.class": "Audio/Source", "node.name": "b"}}}
        ]"#;
        let labels: Vec<_> = parse_dump(json)
            .unwrap()
            .list
            .into_iter()
            .map(|s| s.label)
            .collect();
        assert_eq!(labels, ["Nick", "b"]);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_dump("not json").is_err());
    }
}
