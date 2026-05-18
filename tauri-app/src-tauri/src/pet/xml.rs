use std::{fmt, fs, path::Path};

use quick_xml::de::from_str;
use serde::Deserialize;

use super::model::{
    Animation, Header, Movement, NextOnly, NextTransition, PetManifest, Sequence, Spawn,
    Sound, SpriteSheet,
};

#[derive(Debug)]
pub enum PetXmlError {
    Io(std::io::Error),
    Parse(quick_xml::DeError),
}

impl fmt::Display for PetXmlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "failed to read pet XML: {error}"),
            Self::Parse(error) => write!(formatter, "failed to parse pet XML: {error}"),
        }
    }
}

impl std::error::Error for PetXmlError {}

impl From<std::io::Error> for PetXmlError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<quick_xml::DeError> for PetXmlError {
    fn from(error: quick_xml::DeError) -> Self {
        Self::Parse(error)
    }
}

pub fn parse_pet_manifest_file(path: impl AsRef<Path>) -> Result<PetManifest, PetXmlError> {
    let xml = fs::read_to_string(path)?;
    parse_pet_manifest(&xml)
}

pub fn parse_pet_manifest(xml: &str) -> Result<PetManifest, PetXmlError> {
    let raw: RawRoot = from_str(xml)?;
    Ok(raw.into())
}

#[derive(Debug, Deserialize)]
#[serde(rename = "animations")]
struct RawRoot {
    header: RawHeader,
    image: RawImage,
    spawns: RawSpawns,
    #[serde(rename = "animations")]
    animations: RawAnimations,
    sounds: Option<RawSounds>,
}

#[derive(Debug, Deserialize)]
struct RawHeader {
    author: String,
    title: String,
    petname: String,
    version: String,
    info: String,
    application: String,
    icon: String,
}

#[derive(Debug, Deserialize)]
struct RawImage {
    tilesx: u32,
    tilesy: u32,
    png: String,
    transparency: String,
}

#[derive(Debug, Deserialize)]
struct RawSpawns {
    #[serde(rename = "spawn", default)]
    spawn: Vec<RawSpawn>,
}

#[derive(Debug, Deserialize)]
struct RawSpawn {
    #[serde(rename = "@id")]
    id: i32,
    #[serde(rename = "@probability", default)]
    probability: i32,
    x: String,
    y: String,
    next: Option<RawNext>,
}

#[derive(Debug, Deserialize)]
struct RawAnimations {
    #[serde(rename = "animation", default)]
    animation: Vec<RawAnimation>,
}

#[derive(Debug, Deserialize)]
struct RawAnimation {
    #[serde(rename = "@id")]
    id: i32,
    name: String,
    start: RawMovement,
    end: Option<RawMovement>,
    sequence: RawSequence,
    border: Option<RawTransitionList>,
    gravity: Option<RawTransitionList>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawMovement {
    x: String,
    y: String,
    interval: String,
    #[serde(default)]
    offsety: i32,
    #[serde(default = "default_opacity")]
    opacity: f64,
}

#[derive(Debug, Deserialize)]
struct RawSequence {
    #[serde(rename = "@repeat", default)]
    repeat: String,
    #[serde(rename = "@repeatfrom", default)]
    repeatfrom: i32,
    #[serde(rename = "frame", default)]
    frame: Vec<i32>,
    #[serde(rename = "next", default)]
    next: Vec<RawNext>,
    action: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTransitionList {
    #[serde(rename = "next", default)]
    next: Vec<RawNext>,
}

#[derive(Debug, Deserialize)]
struct RawSounds {
    #[serde(rename = "sound", default)]
    sound: Vec<RawSound>,
}

#[derive(Debug, Deserialize)]
struct RawSound {
    #[serde(rename = "@animationid")]
    animationid: i32,
    #[serde(default = "default_probability")]
    probability: i32,
    #[serde(rename = "loop", default)]
    loop_count: i32,
    base64: String,
}

#[derive(Debug, Deserialize)]
struct RawNext {
    #[serde(rename = "$text")]
    value: i32,
    #[serde(rename = "@probability", default)]
    probability: i32,
    #[serde(rename = "@only")]
    only: Option<String>,
}

fn default_opacity() -> f64 {
    1.0
}

fn default_probability() -> i32 {
    100
}

impl From<RawRoot> for PetManifest {
    fn from(raw: RawRoot) -> Self {
        Self {
            header: raw.header.into(),
            image: raw.image.into(),
            spawns: raw.spawns.spawn.into_iter().map(Spawn::from).collect(),
            animations: raw
                .animations
                .animation
                .into_iter()
                .map(Animation::from)
                .collect(),
            sounds: raw
                .sounds
                .map(|sounds| sounds.sound.into_iter().map(Sound::from).collect())
                .unwrap_or_default(),
        }
    }
}

impl From<RawHeader> for Header {
    fn from(raw: RawHeader) -> Self {
        Self {
            author: trim_text(raw.author),
            title: trim_text(raw.title),
            petname: trim_text(raw.petname),
            version: trim_text(raw.version),
            info: trim_text(raw.info),
            application: trim_text(raw.application),
            icon_base64: trim_text(raw.icon),
        }
    }
}

impl From<RawImage> for SpriteSheet {
    fn from(raw: RawImage) -> Self {
        Self {
            tiles_x: raw.tilesx,
            tiles_y: raw.tilesy,
            png_base64: trim_text(raw.png),
            transparency: trim_text(raw.transparency),
        }
    }
}

impl From<RawSpawn> for Spawn {
    fn from(raw: RawSpawn) -> Self {
        Self {
            id: raw.id,
            probability: raw.probability,
            x: trim_text(raw.x),
            y: trim_text(raw.y),
            next: raw.next.map(NextTransition::from),
        }
    }
}

impl From<RawAnimation> for Animation {
    fn from(raw: RawAnimation) -> Self {
        let start = Movement::from(raw.start);
        let end = raw.end.map(Movement::from).unwrap_or_else(|| start.clone());

        Self {
            id: raw.id,
            name: trim_text(raw.name),
            start,
            end,
            sequence: raw.sequence.into(),
            border_next: raw
                .border
                .map(|border| border.next.into_iter().map(NextTransition::from).collect())
                .unwrap_or_default(),
            gravity_next: raw
                .gravity
                .map(|gravity| gravity.next.into_iter().map(NextTransition::from).collect())
                .unwrap_or_default(),
        }
    }
}

impl From<RawMovement> for Movement {
    fn from(raw: RawMovement) -> Self {
        Self {
            x: trim_text(raw.x),
            y: trim_text(raw.y),
            interval: trim_text(raw.interval),
            offset_y: raw.offsety,
            opacity: raw.opacity,
        }
    }
}

impl From<RawSequence> for Sequence {
    fn from(raw: RawSequence) -> Self {
        Self {
            repeat: trim_text(raw.repeat),
            repeat_from: raw.repeatfrom,
            frames: raw.frame,
            next: raw.next.into_iter().map(NextTransition::from).collect(),
            action: raw.action.map(trim_text).filter(|value| !value.is_empty()),
        }
    }
}

impl From<RawNext> for NextTransition {
    fn from(raw: RawNext) -> Self {
        Self {
            animation_id: raw.value,
            probability: raw.probability,
            only: NextOnly::from_legacy(raw.only.as_deref()),
        }
    }
}

impl From<RawSound> for Sound {
    fn from(raw: RawSound) -> Self {
        Self {
            animation_id: raw.animationid,
            probability: raw.probability,
            loop_count: raw.loop_count,
            base64: trim_text(raw.base64),
        }
    }
}

fn trim_text(value: String) -> String {
    value.trim().to_owned()
}
