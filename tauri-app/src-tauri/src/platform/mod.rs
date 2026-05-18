use serde::Serialize;

#[cfg(windows)]
mod windows;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RectInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl RectInfo {
    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub id: usize,
    pub name: String,
    pub bounds: RectInfo,
    pub work_area: RectInfo,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: String,
    pub monitors: Vec<MonitorInfo>,
    pub primary_work_area: RectInfo,
    pub taskbar_edge: Option<String>,
    pub floor_y: i32,
    pub fullscreen_window_active: bool,
    pub window_collision_available: bool,
    pub click_through_available: bool,
}

#[tauri::command]
pub fn dump_platform_info() -> Result<PlatformInfo, String> {
    platform_info()
}

pub fn platform_info() -> Result<PlatformInfo, String> {
    imp::platform_info()
}

pub fn primary_work_area() -> Result<RectInfo, String> {
    Ok(platform_info()?.primary_work_area)
}

pub fn is_fullscreen_window_active() -> bool {
    imp::is_fullscreen_window_active()
}

pub fn find_window_under_pet(pet_bounds: RectInfo) -> Option<RectInfo> {
    imp::find_window_under_pet(pet_bounds)
}

pub fn set_click_through(enabled: bool) -> Result<(), String> {
    imp::set_click_through(enabled)
}

pub fn infer_taskbar_edge(bounds: &RectInfo, work_area: &RectInfo) -> Option<String> {
    if work_area.bottom() < bounds.bottom() {
        Some("bottom".to_owned())
    } else if work_area.y > bounds.y {
        Some("top".to_owned())
    } else if work_area.x > bounds.x {
        Some("left".to_owned())
    } else if work_area.right() < bounds.right() {
        Some("right".to_owned())
    } else {
        None
    }
}

#[cfg(windows)]
mod imp {
    pub use super::windows::{
        find_window_under_pet, is_fullscreen_window_active, platform_info, set_click_through,
    };
}

#[cfg(not(windows))]
mod imp {
    use super::{PlatformInfo, RectInfo};

    pub fn platform_info() -> Result<PlatformInfo, String> {
        let fallback = RectInfo {
            x: 0,
            y: 0,
            width: 160,
            height: 160,
        };

        Ok(PlatformInfo {
            os: std::env::consts::OS.to_owned(),
            monitors: Vec::new(),
            primary_work_area: fallback.clone(),
            taskbar_edge: None,
            floor_y: fallback.bottom(),
            fullscreen_window_active: false,
            window_collision_available: false,
            click_through_available: false,
        })
    }

    pub fn is_fullscreen_window_active() -> bool {
        false
    }

    pub fn find_window_under_pet(_pet_bounds: RectInfo) -> Option<RectInfo> {
        None
    }

    pub fn set_click_through(_enabled: bool) -> Result<(), String> {
        Err("click-through is only available on Windows".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{infer_taskbar_edge, RectInfo};

    #[test]
    fn infers_bottom_taskbar_from_work_area() {
        let bounds = RectInfo {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let work_area = RectInfo {
            x: 0,
            y: 0,
            width: 1920,
            height: 1040,
        };

        assert_eq!(
            Some("bottom".to_owned()),
            infer_taskbar_edge(&bounds, &work_area)
        );
    }
}
