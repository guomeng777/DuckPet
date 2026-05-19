use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

#[cfg(windows)]
use ::windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
};

pub const DEFAULT_PET_INSTANCE_ID: &str = "pet_1";
pub const SETTINGS_WINDOW_LABEL: &str = "settings";

const PET_WINDOW_WIDTH: f64 = 160.0;
const PET_WINDOW_HEIGHT: f64 = 160.0;

#[tauri::command]
pub fn open_pet_window(app: AppHandle) -> Result<(), String> {
    open_pet_window_for_instance(app, DEFAULT_PET_INSTANCE_ID)
}

#[tauri::command]
pub fn close_pet_window(app: AppHandle) -> Result<(), String> {
    close_pet_window_for_instance(app, DEFAULT_PET_INSTANCE_ID)
}

pub fn open_pet_window_for_instance(
    app: AppHandle,
    pet_instance_id: &str,
) -> Result<(), String> {
    let label = pet_window_label(pet_instance_id);

    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|error| error.to_string())?;
        raise_pet_window(&window)?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::App(format!("index.html?window=pet&petInstanceId={pet_instance_id}").into()),
    )
    .title(format!("DuckPet {pet_instance_id}"))
    .inner_size(PET_WINDOW_WIDTH, PET_WINDOW_HEIGHT)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .focused(false)
    .skip_taskbar(true)
    .shadow(false)
    .build()
    .map_err(|error| error.to_string())?;

    raise_pet_window(&window)?;

    Ok(())
}

pub fn close_pet_window_for_instance(
    app: AppHandle,
    pet_instance_id: &str,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&pet_window_label(pet_instance_id)) {
        window.close().map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        raise_pet_windows(&app)?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        &app,
        SETTINGS_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("DuckPet Settings")
    .inner_size(900.0, 600.0)
    .resizable(true)
    .build()
    .map_err(|error| error.to_string())?;

    window.set_focus().map_err(|error| error.to_string())?;
    raise_pet_windows(&app)?;

    Ok(())
}

pub fn raise_pet_windows(app: &AppHandle) -> Result<(), String> {
    for (label, window) in app.webview_windows() {
        if is_pet_window_label(&label) {
            raise_pet_window(&window)?;
        }
    }

    Ok(())
}

pub fn raise_pet_window(window: &WebviewWindow) -> Result<(), String> {
    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    apply_native_topmost(window)
}

pub fn set_pet_click_through(
    app: AppHandle,
    pet_instance_id: &str,
    enabled: bool,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&pet_window_label(pet_instance_id)) {
        window
            .set_ignore_cursor_events(enabled)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn pet_window_label(pet_instance_id: &str) -> String {
    pet_instance_id.to_owned()
}

fn is_pet_window_label(label: &str) -> bool {
    label.starts_with("pet_")
}

#[cfg(windows)]
fn apply_native_topmost(window: &WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
    }
    .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn apply_native_topmost(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}
