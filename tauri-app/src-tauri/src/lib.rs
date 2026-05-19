mod audio;
mod commands;
pub mod pet;
pub mod platform;
mod windows;

use tauri::menu::MenuBuilder;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

const TRAY_SHOW_SETTINGS_ID: &str = "show_settings";
const TRAY_QUIT_ID: &str = "quit";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(audio::AudioState::default())
        .manage(commands::pet::PetRuntimeState::default())
        .setup(|app| {
            let menu = MenuBuilder::new(app)
                .text(TRAY_SHOW_SETTINGS_ID, "显示主窗口")
                .separator()
                .text(TRAY_QUIT_ID, "退出")
                .build()?;

            let mut tray = TrayIconBuilder::with_id("duckpet")
                .menu(&menu)
                .tooltip("DuckPet")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    TRAY_SHOW_SETTINGS_ID => {
                        if let Err(error) = windows::open_settings_window(app.clone()) {
                            eprintln!("failed to show settings window from tray: {error}");
                        }
                    }
                    TRAY_QUIT_ID => {
                        if let Err(error) = commands::pet::exit_app_from_handle(app) {
                            eprintln!("failed to exit app from tray: {error}");
                            app.exit(1);
                        }
                    }
                    _ => {}
                });

            if let Some(icon) = app.default_window_icon().cloned() {
                tray = tray.icon(icon);
            }

            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == windows::SETTINGS_WINDOW_LABEL {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        if let Err(error) = window.hide() {
                            eprintln!("failed to hide settings window: {error}");
                        }
                    }
                    tauri::WindowEvent::Focused(true) => {
                        if let Err(error) = windows::raise_pet_windows(&window.app_handle()) {
                            eprintln!("failed to raise pet windows after settings focus: {error}");
                        }
                    }
                    _ => {}
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            audio::get_audio_status,
            audio::set_audio_muted,
            commands::pet::load_pet_manifest,
            commands::pet::list_available_pets,
            commands::pet_assets::list_manageable_pets,
            commands::pet_assets::validate_pet_archive,
            commands::pet_assets::save_uploaded_pet,
            commands::pet_assets::delete_uploaded_pet,
            commands::pet_assets::export_pet_archive,
            commands::pet::start_pet_runtime,
            commands::pet::pause_pet_runtime,
            commands::pet::resume_pet_runtime,
            commands::pet::close_pet_runtime,
            commands::pet::close_all_pet_runtimes,
            commands::pet::spawn_pet_runtime,
            commands::pet::get_pet_runtime_status,
            commands::pet::set_window_collision_enabled,
            commands::pet::set_pet_click_through_enabled,
            commands::pet::begin_pet_drag,
            commands::pet::end_pet_drag,
            commands::pet::trigger_pet_kill_or_close,
            commands::pet::exit_app,
            commands::pet::next_pet_frame_for_instance,
            commands::pet::next_active_pet_frame,
            commands::pet::next_pet_frame,
            platform::dump_platform_info,
            windows::open_settings_window,
            windows::open_pet_window,
            windows::close_pet_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
