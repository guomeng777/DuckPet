use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW},
        UI::WindowsAndMessaging::{
            EnumWindows, FindWindowW, GetForegroundWindow, GetLayeredWindowAttributes,
            GetTitleBarInfo, GetWindowLongW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
            IsIconic, IsWindowVisible, GWL_EXSTYLE, LAYERED_WINDOW_ATTRIBUTES_FLAGS, LWA_ALPHA,
            TITLEBARINFO, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
        },
    },
    core::{w, BOOL},
};

use super::{infer_taskbar_edge, MonitorInfo, PlatformInfo, RectInfo};

const PRIMARY_MONITOR_FLAG: u32 = 1;
const STATE_SYSTEM_INVISIBLE: u32 = 0x0000_8000;
const MIN_SUPPORTING_WINDOW_SIZE: i32 = 40;

pub fn platform_info() -> Result<PlatformInfo, String> {
    let monitors = monitors()?;
    let primary_monitor = monitors
        .iter()
        .find(|monitor| monitor.is_primary)
        .or_else(|| monitors.first())
        .ok_or_else(|| "no display monitors were found".to_owned())?;
    let primary_work_area = refine_work_area_with_taskbar(
        &primary_monitor.bounds,
        &primary_monitor.work_area,
        primary_taskbar_rect().as_ref(),
    );
    let taskbar_edge = infer_taskbar_edge(&primary_monitor.bounds, &primary_work_area);

    Ok(PlatformInfo {
        os: "windows".to_owned(),
        monitors,
        floor_y: primary_work_area.bottom(),
        primary_work_area,
        taskbar_edge,
        fullscreen_window_active: is_fullscreen_window_active(),
        window_collision_available: true,
        click_through_available: true,
    })
}

pub fn monitors() -> Result<Vec<MonitorInfo>, String> {
    let mut monitors = Vec::<MonitorInfo>::new();
    let monitors_ptr = &mut monitors as *mut Vec<MonitorInfo>;

    let ok = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitor_proc),
            LPARAM(monitors_ptr as isize),
        )
    };

    if !ok.as_bool() {
        return Err("EnumDisplayMonitors failed".to_owned());
    }

    Ok(monitors)
}

pub fn is_fullscreen_window_active() -> bool {
    let hwnd = unsafe { GetForegroundWindow() };

    if hwnd.0.is_null() {
        return false;
    }

    let mut rect = RECT::default();

    if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
        return false;
    }

    monitors()
        .map(|monitors| {
            monitors.iter().any(|monitor| {
                rect.left <= monitor.bounds.x
                    && rect.top <= monitor.bounds.y
                    && rect.right >= monitor.bounds.right()
                    && rect.bottom >= monitor.bounds.bottom()
            })
        })
        .unwrap_or(false)
}

pub fn find_window_under_pet(pet_bounds: RectInfo) -> Option<RectInfo> {
    let mut search = WindowSearch {
        pet_bounds,
        best: None,
    };
    let search_ptr = &mut search as *mut WindowSearch;

    unsafe { EnumWindows(Some(enum_window_proc), LPARAM(search_ptr as isize)) }.ok()?;

    search.best
}

pub fn set_click_through(_enabled: bool) -> Result<(), String> {
    Err("click-through is reserved for a later task".to_owned())
}

unsafe extern "system" fn enum_monitor_proc(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let monitors = unsafe { &mut *(data.0 as *mut Vec<MonitorInfo>) };
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    let ok = unsafe {
        GetMonitorInfoW(
            monitor,
            &mut info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
    };

    if ok.as_bool() {
        let id = monitors.len();
        monitors.push(MonitorInfo {
            id,
            name: device_name(&info.szDevice),
            bounds: rect_info(info.monitorInfo.rcMonitor),
            work_area: rect_info(info.monitorInfo.rcWork),
            is_primary: info.monitorInfo.dwFlags & PRIMARY_MONITOR_FLAG == PRIMARY_MONITOR_FLAG,
        });
    }

    BOOL(1)
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, data: LPARAM) -> BOOL {
    let search = unsafe { &mut *(data.0 as *mut WindowSearch) };

    if let Some(candidate) = candidate_window_rect(hwnd, &search.pet_bounds) {
        let should_replace = search
            .best
            .as_ref()
            .map(|best| candidate.y < best.y)
            .unwrap_or(true);

        if should_replace {
            search.best = Some(candidate);
        }
    }

    BOOL(1)
}

fn candidate_window_rect(hwnd: HWND, pet_bounds: &RectInfo) -> Option<RectInfo> {
    if hwnd.0.is_null() {
        return None;
    }

    if unsafe { !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() } {
        return None;
    }

    let title = window_title(hwnd);
    if title.trim().is_empty() || title.starts_with("DuckPet") {
        return None;
    }

    if has_ignored_window_style(hwnd) || has_invisible_titlebar(hwnd) {
        return None;
    }

    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
        return None;
    }

    let candidate = rect_info(rect);
    if candidate.width < MIN_SUPPORTING_WINDOW_SIZE || candidate.height < MIN_SUPPORTING_WINDOW_SIZE
    {
        return None;
    }

    if !is_supporting_candidate(&candidate, pet_bounds) {
        return None;
    }

    Some(candidate)
}

fn has_ignored_window_style(hwnd: HWND) -> bool {
    let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32;

    if ex_style & WS_EX_TOOLWINDOW.0 != 0 || ex_style & WS_EX_TRANSPARENT.0 != 0 {
        return true;
    }

    ex_style & WS_EX_LAYERED.0 != 0 && is_transparent_layered_window(hwnd)
}

fn is_transparent_layered_window(hwnd: HWND) -> bool {
    let mut color_key = COLORREF(0);
    let mut alpha = 255_u8;
    let mut flags = LAYERED_WINDOW_ATTRIBUTES_FLAGS(0);

    if unsafe {
        GetLayeredWindowAttributes(
            hwnd,
            Some(&mut color_key),
            Some(&mut alpha),
            Some(&mut flags),
        )
    }
    .is_err()
    {
        return true;
    }

    flags.contains(LWA_ALPHA) && alpha < 255
}

fn has_invisible_titlebar(hwnd: HWND) -> bool {
    let mut title_bar_info = TITLEBARINFO::default();
    title_bar_info.cbSize = std::mem::size_of::<TITLEBARINFO>() as u32;

    if unsafe { GetTitleBarInfo(hwnd, &mut title_bar_info) }.is_err() {
        return false;
    }

    title_bar_info.rgstate[0] & STATE_SYSTEM_INVISIBLE != 0
}

fn window_title(hwnd: HWND) -> String {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len <= 0 {
        return String::new();
    }

    let mut buffer = vec![0_u16; len as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buffer) };

    if copied <= 0 {
        return String::new();
    }

    String::from_utf16_lossy(&buffer[..copied as usize])
}

fn is_supporting_candidate(candidate: &RectInfo, pet_bounds: &RectInfo) -> bool {
    let pet_bottom = pet_bounds.bottom();
    let pet_center_x = pet_bounds.x + pet_bounds.width / 2;

    candidate.y >= pet_bottom - 3
        && pet_center_x >= candidate.x
        && pet_center_x <= candidate.right()
}

fn rect_info(rect: RECT) -> RectInfo {
    RectInfo {
        x: rect.left,
        y: rect.top,
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    }
}

fn primary_taskbar_rect() -> Option<RectInfo> {
    let hwnd = unsafe { FindWindowW(w!("Shell_TrayWnd"), None).ok()? };
    let mut rect = RECT::default();

    unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;

    let taskbar = rect_info(rect);
    if taskbar.width <= 0 || taskbar.height <= 0 {
        None
    } else {
        Some(taskbar)
    }
}

fn refine_work_area_with_taskbar(
    bounds: &RectInfo,
    work_area: &RectInfo,
    taskbar: Option<&RectInfo>,
) -> RectInfo {
    let Some(taskbar) = taskbar else {
        return work_area.clone();
    };

    if !rects_intersect(bounds, taskbar) {
        return work_area.clone();
    }

    let mut refined = work_area.clone();
    let bounds_center_x = bounds.x + bounds.width / 2;
    let bounds_center_y = bounds.y + bounds.height / 2;

    if taskbar.y >= bounds_center_y {
        let bottom = refined.bottom().min(taskbar.y);
        refined.height = (bottom - refined.y).max(0);
    } else if taskbar.bottom() <= bounds_center_y {
        let top = refined.y.max(taskbar.bottom());
        let bottom = refined.bottom();
        refined.y = top;
        refined.height = (bottom - top).max(0);
    } else if taskbar.x <= bounds_center_x {
        let left = refined.x.max(taskbar.right());
        let right = refined.right();
        refined.x = left;
        refined.width = (right - left).max(0);
    } else {
        let right = refined.right().min(taskbar.x);
        refined.width = (right - refined.x).max(0);
    }

    refined
}

fn rects_intersect(left: &RectInfo, right: &RectInfo) -> bool {
    left.x < right.right()
        && left.right() > right.x
        && left.y < right.bottom()
        && left.bottom() > right.y
}

fn device_name(buffer: &[u16]) -> String {
    let len = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());

    String::from_utf16_lossy(&buffer[..len])
}

struct WindowSearch {
    pet_bounds: RectInfo,
    best: Option<RectInfo>,
}

#[cfg(test)]
mod tests {
    use super::{refine_work_area_with_taskbar, RectInfo};

    #[test]
    fn refines_full_monitor_work_area_with_bottom_taskbar() {
        let bounds = RectInfo {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let work_area = bounds.clone();
        let taskbar = RectInfo {
            x: 0,
            y: 1032,
            width: 1920,
            height: 48,
        };

        let refined = refine_work_area_with_taskbar(&bounds, &work_area, Some(&taskbar));

        assert_eq!(1032, refined.height);
    }

    #[test]
    fn keeps_smaller_existing_work_area() {
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
            height: 1000,
        };
        let taskbar = RectInfo {
            x: 0,
            y: 1032,
            width: 1920,
            height: 48,
        };

        let refined = refine_work_area_with_taskbar(&bounds, &work_area, Some(&taskbar));

        assert_eq!(1000, refined.height);
    }
}
