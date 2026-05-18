#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PetManifest {
    pub header: Header,
    pub image: SpriteSheet,
    pub spawns: Vec<Spawn>,
    pub animations: Vec<Animation>,
    pub sounds: Vec<Sound>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub author: String,
    pub title: String,
    pub petname: String,
    pub version: String,
    pub info: String,
    pub application: String,
    pub icon_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteSheet {
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub png_base64: String,
    pub transparency: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spawn {
    pub id: i32,
    pub probability: i32,
    pub x: String,
    pub y: String,
    pub next: Option<NextTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Animation {
    pub id: i32,
    pub name: String,
    pub start: Movement,
    pub end: Movement,
    pub sequence: Sequence,
    pub border_next: Vec<NextTransition>,
    pub gravity_next: Vec<NextTransition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Movement {
    pub x: String,
    pub y: String,
    pub interval: String,
    pub offset_y: i32,
    pub opacity: f64,
}

impl Eq for Movement {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    pub repeat: String,
    pub repeat_from: i32,
    pub frames: Vec<i32>,
    pub next: Vec<NextTransition>,
    pub action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextTransition {
    pub animation_id: i32,
    pub probability: i32,
    pub only: NextOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sound {
    pub animation_id: i32,
    pub probability: i32,
    pub loop_count: i32,
    pub base64: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextOnly {
    None,
    Taskbar,
    Window,
    Horizontal,
    HorizontalPlus,
    Vertical,
}

impl NextOnly {
    pub fn from_legacy(value: Option<&str>) -> Self {
        match value.unwrap_or("none").trim() {
            "taskbar" => Self::Taskbar,
            "window" => Self::Window,
            "horizontal" => Self::Horizontal,
            "horizontal+" => Self::HorizontalPlus,
            "vertical" => Self::Vertical,
            _ => Self::None,
        }
    }
}
