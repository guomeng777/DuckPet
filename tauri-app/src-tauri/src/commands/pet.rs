use std::{
    collections::BTreeMap,
    env, fs,
    path::PathBuf,
    sync::Mutex,
};

use base64::{engine::general_purpose::STANDARD, Engine};
use quick_xml::{events::Event, Reader};
use serde::Serialize;
use tauri::{AppHandle, Manager, PhysicalPosition, State};

use crate::audio::{self, AudioState};
use crate::pet::{
    model::PetManifest,
    runtime::{PetFrame, PetRuntime, SpriteDimensions},
    xml::parse_pet_manifest_file,
};
use crate::{commands::pet_assets, platform, windows};

const MAX_PET_INSTANCES: usize = 4;
const PET_WINDOW_WIDTH: i32 = 160;
const PET_WINDOW_HEIGHT: i32 = 160;

pub struct PetRuntimeState {
    store: Mutex<PetRuntimeStore>,
}

impl Default for PetRuntimeState {
    fn default() -> Self {
        Self {
            store: Mutex::new(PetRuntimeStore::default()),
        }
    }
}

struct PetRuntimeStore {
    instances: BTreeMap<String, PetRuntimeInstance>,
    active_instance_id: Option<String>,
    next_instance_number: u64,
    max_instances: usize,
    window_collision_enabled: bool,
    click_through_enabled: bool,
}

impl Default for PetRuntimeStore {
    fn default() -> Self {
        Self {
            instances: BTreeMap::new(),
            active_instance_id: None,
            next_instance_number: 0,
            max_instances: MAX_PET_INSTANCES,
            window_collision_enabled: true,
            click_through_enabled: false,
        }
    }
}

struct PetRuntimeInstance {
    xml_path: PathBuf,
    runtime: Option<PetRuntime>,
    paused: bool,
    latest_frame: Option<PetFrame>,
    dragging: bool,
    pending_animation_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailablePet {
    pub id: String,
    pub xml_path: String,
    pub header: PetHeaderSummary,
    pub animation_count: usize,
    pub spawn_count: usize,
    pub sound_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetRuntimeStatus {
    pub selected_xml_path: Option<String>,
    pub active_instance_id: Option<String>,
    pub is_running: bool,
    pub is_paused: bool,
    pub window_open: bool,
    pub latest_frame: Option<PetFrame>,
    pub max_instances: usize,
    pub window_collision_enabled: bool,
    pub click_through_enabled: bool,
    pub instances: Vec<PetInstanceStatus>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetInstanceStatus {
    pub pet_instance_id: String,
    pub xml_path: String,
    pub is_paused: bool,
    pub window_open: bool,
    pub latest_frame: Option<PetFrame>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetManifestSummary {
    pub source_path: String,
    pub header: PetHeaderSummary,
    pub sprite_sheet: SpriteSheetSummary,
    pub animation_count: usize,
    pub spawn_count: usize,
    pub sound_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetHeaderSummary {
    pub author: String,
    pub title: String,
    pub petname: String,
    pub version: String,
    pub info: String,
    pub application: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSheetSummary {
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub tile_width: i32,
    pub tile_height: i32,
    pub transparency: String,
    pub data_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetInteractionResult {
    pub status: PetRuntimeStatus,
    pub used_animation: bool,
}

#[tauri::command]
pub fn load_pet_manifest(xml_path: String) -> Result<PetManifestSummary, String> {
    let resolved_path = resolve_xml_path(&xml_path)?;
    let manifest = parse_pet_manifest_file(&resolved_path).map_err(|error| error.to_string())?;

    build_manifest_summary(resolved_path, manifest)
}

#[tauri::command]
pub fn list_available_pets(app: AppHandle) -> Result<Vec<AvailablePet>, String> {
    list_available_pets_from_dirs(pet_assets::available_pet_directories(&app)?)
}

pub(crate) fn list_available_pets_from_dirs(
    pet_dirs: Vec<pet_assets::PetDirectory>,
) -> Result<Vec<AvailablePet>, String> {
    let mut pets = Vec::new();

    for pet_dir in pet_dirs {
        let xml_path = pet_dir.path.join("animations.xml");
        if !xml_path.is_file() {
            continue;
        }

        let index = load_pet_manifest_index(&xml_path).map_err(|error| {
            format!(
                "failed to index pet manifest {}: {error}",
                xml_path.display()
            )
        })?;
        let resolved_path = xml_path
            .canonicalize()
            .map_err(|error| format!("failed to resolve pet XML path: {error}"))?;
        let id = pet_dir.id;

        pets.push(AvailablePet {
            id,
            xml_path: resolved_path.display().to_string(),
            header: index.header,
            animation_count: index.animation_count,
            spawn_count: index.spawn_count,
            sound_count: index.sound_count,
        });
    }

    pets.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(pets)
}

#[tauri::command]
pub fn start_pet_runtime(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    xml_path: String,
) -> Result<PetRuntimeStatus, String> {
    let pet_instance_id = upsert_active_instance(&state, xml_path)?;
    apply_click_through_to_instance(&app, &state, &pet_instance_id)?;
    runtime_status(app, state)
}

#[tauri::command]
pub fn spawn_pet_runtime(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    xml_path: String,
) -> Result<PetRuntimeStatus, String> {
    let pet_instance_id = create_instance(&state, xml_path)?;
    apply_click_through_to_instance(&app, &state, &pet_instance_id)?;
    runtime_status(app, state)
}

#[tauri::command]
pub fn pause_pet_runtime(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    pet_instance_id: Option<String>,
) -> Result<PetRuntimeStatus, String> {
    update_instance_pause(&state, pet_instance_id, true)?;
    runtime_status(app, state)
}

#[tauri::command]
pub fn resume_pet_runtime(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    pet_instance_id: Option<String>,
) -> Result<PetRuntimeStatus, String> {
    let pet_instance_id = update_instance_pause(&state, pet_instance_id, false)?;
    apply_click_through_to_instance(&app, &state, &pet_instance_id)?;
    runtime_status(app, state)
}

#[tauri::command]
pub fn close_pet_runtime(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    pet_instance_id: Option<String>,
) -> Result<PetRuntimeStatus, String> {
    let pet_instance_id = remove_instance(&state, pet_instance_id)?;
    windows::close_pet_window_for_instance(app.clone(), &pet_instance_id)?;
    runtime_status(app, state)
}

#[tauri::command]
pub fn close_all_pet_runtimes(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
) -> Result<PetRuntimeStatus, String> {
    let pet_instance_ids = {
        let mut store = lock_store(&state)?;
        let ids = store.instances.keys().cloned().collect::<Vec<_>>();
        store.instances.clear();
        store.active_instance_id = None;
        ids
    };

    for pet_instance_id in pet_instance_ids {
        windows::close_pet_window_for_instance(app.clone(), &pet_instance_id)?;
    }

    runtime_status(app, state)
}

#[tauri::command]
pub fn get_pet_runtime_status(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
) -> Result<PetRuntimeStatus, String> {
    runtime_status(app, state)
}

#[tauri::command]
pub fn set_window_collision_enabled(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    enabled: bool,
) -> Result<PetRuntimeStatus, String> {
    {
        let mut store = lock_store(&state)?;
        store.window_collision_enabled = enabled;
    }

    runtime_status(app, state)
}

#[tauri::command]
pub fn set_pet_click_through_enabled(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    enabled: bool,
) -> Result<PetRuntimeStatus, String> {
    let pet_instance_ids = {
        let mut store = lock_store(&state)?;
        store.click_through_enabled = enabled;
        store.instances.keys().cloned().collect::<Vec<_>>()
    };

    for pet_instance_id in pet_instance_ids {
        windows::set_pet_click_through(app.clone(), &pet_instance_id, enabled)?;
    }

    runtime_status(app, state)
}

#[tauri::command]
pub fn begin_pet_drag(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    pet_instance_id: String,
) -> Result<PetRuntimeStatus, String> {
    set_instance_interaction_animation(&state, &pet_instance_id, "drag", true)?;
    runtime_status(app, state)
}

#[tauri::command]
pub fn end_pet_drag(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    pet_instance_id: String,
) -> Result<PetRuntimeStatus, String> {
    let position = app
        .get_webview_window(&windows::pet_window_label(&pet_instance_id))
        .and_then(|window| window.outer_position().ok());
    let work_area = platform::primary_work_area().ok();

    {
        let mut store = lock_store(&state)?;
        let instance = store
            .instances
            .get_mut(&pet_instance_id)
            .ok_or_else(|| format!("pet instance does not exist: {pet_instance_id}"))?;
        instance.dragging = false;
        instance.pending_animation_name = None;

        if let (Some(position), Some(work_area), Some(runtime)) =
            (position, work_area, instance.runtime.as_mut())
        {
            runtime.set_position(position.x - work_area.x, position.y - work_area.y);
        }
    }

    runtime_status(app, state)
}

#[tauri::command]
pub fn trigger_pet_kill_or_close(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    pet_instance_id: String,
) -> Result<PetInteractionResult, String> {
    let used_animation =
        set_instance_interaction_animation(&state, &pet_instance_id, "kill", false)?;

    if !used_animation {
        remove_instance(&state, Some(pet_instance_id.clone()))?;
        windows::close_pet_window_for_instance(app.clone(), &pet_instance_id)?;
    }

    Ok(PetInteractionResult {
        status: runtime_status(app, state)?,
        used_animation,
    })
}

#[tauri::command]
pub fn exit_app(app: AppHandle, state: State<'_, PetRuntimeState>) -> Result<(), String> {
    clear_runtime_state(&state)?;
    app.exit(0);
    Ok(())
}

pub fn exit_app_from_handle(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<PetRuntimeState>();
    clear_runtime_state(&state)?;
    app.exit(0);
    Ok(())
}

fn clear_runtime_state(state: &State<'_, PetRuntimeState>) -> Result<(), String> {
    {
        let mut store = lock_store(&state)?;
        store.instances.clear();
        store.active_instance_id = None;
    }

    Ok(())
}

#[tauri::command]
pub fn next_pet_frame_for_instance(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    audio_state: State<'_, AudioState>,
    pet_instance_id: String,
    viewport_width: i32,
    viewport_height: i32,
) -> Result<PetFrame, String> {
    let mut store = lock_store(&state)?;
    let window_collision_enabled = store.window_collision_enabled;
    let instance = store
        .instances
        .get_mut(&pet_instance_id)
        .ok_or_else(|| format!("pet instance does not exist: {pet_instance_id}"))?;

    next_pet_frame_for_instance_inner(
        app,
        pet_instance_id,
        instance,
        &audio_state,
        window_collision_enabled,
        viewport_width,
        viewport_height,
    )
}

#[tauri::command]
pub fn next_active_pet_frame(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    audio_state: State<'_, AudioState>,
    viewport_width: i32,
    viewport_height: i32,
) -> Result<PetFrame, String> {
    let mut store = lock_store(&state)?;
    let pet_instance_id = active_instance_id(&store)?;
    let window_collision_enabled = store.window_collision_enabled;
    let instance = store
        .instances
        .get_mut(&pet_instance_id)
        .ok_or_else(|| format!("pet instance does not exist: {pet_instance_id}"))?;

    next_pet_frame_for_instance_inner(
        app,
        pet_instance_id,
        instance,
        &audio_state,
        window_collision_enabled,
        viewport_width,
        viewport_height,
    )
}

#[tauri::command]
pub fn next_pet_frame(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
    audio_state: State<'_, AudioState>,
    xml_path: String,
    viewport_width: i32,
    viewport_height: i32,
) -> Result<PetFrame, String> {
    let pet_instance_id = upsert_active_instance(&state, xml_path)?;
    let mut store = lock_store(&state)?;
    let window_collision_enabled = store.window_collision_enabled;
    let instance = store
        .instances
        .get_mut(&pet_instance_id)
        .ok_or_else(|| format!("pet instance does not exist: {pet_instance_id}"))?;

    next_pet_frame_for_instance_inner(
        app,
        pet_instance_id,
        instance,
        &audio_state,
        window_collision_enabled,
        viewport_width,
        viewport_height,
    )
}

fn next_pet_frame_for_instance_inner(
    app: AppHandle,
    pet_instance_id: String,
    instance: &mut PetRuntimeInstance,
    audio_state: &State<'_, AudioState>,
    window_collision_enabled: bool,
    viewport_width: i32,
    viewport_height: i32,
) -> Result<PetFrame, String> {
    if instance.paused {
        let mut frame = instance
            .latest_frame
            .clone()
            .ok_or_else(|| "pet runtime is paused before the first frame".to_owned())?;
        present_pet_frame(&app, &pet_instance_id, &mut frame)?;
        return Ok(frame);
    }

    let work_area = platform::primary_work_area().ok();
    let scale_factor = pet_window_scale_factor(&app, &pet_instance_id);

    if instance.runtime.is_none() {
        let manifest =
            parse_pet_manifest_file(&instance.xml_path).map_err(|error| error.to_string())?;
        let dimensions = sprite_dimensions(&manifest)?;
        let (runtime_width, runtime_height) = runtime_area(
            work_area.as_ref(),
            viewport_width,
            viewport_height,
            dimensions,
            scale_factor,
        );
        instance.runtime = Some(PetRuntime::new(
            instance.xml_path.clone(),
            manifest,
            dimensions,
            runtime_width,
            runtime_height,
        )?);
    }

    let dimensions = instance
        .runtime
        .as_ref()
        .map(|runtime| runtime.sprite_dimensions())
        .ok_or_else(|| "pet runtime is not initialized".to_owned())?;
    let (runtime_width, runtime_height) = runtime_area(
        work_area.as_ref(),
        viewport_width,
        viewport_height,
        dimensions,
        scale_factor,
    );
    let pending_animation_name = instance.pending_animation_name.take();
    let runtime = instance
        .runtime
        .as_mut()
        .ok_or_else(|| "pet runtime is not initialized".to_owned())?;
    runtime.set_area(runtime_width, runtime_height);
    if let Some(animation_name) = pending_animation_name {
        if !runtime.enter_animation_by_name(&animation_name)? && instance.dragging {
            instance.dragging = false;
        }
    }
    let window_floor_y = if window_collision_enabled {
        work_area
            .as_ref()
            .and_then(|area| detect_window_floor(runtime, area))
    } else {
        None
    };
    let mut frame = runtime.next_frame_with_window_floor(window_floor_y)?;

    if frame.sequence_step == 0 {
        if let Some(sound) = runtime.sound_for_animation(frame.animation_id) {
            audio::play_sound_for_animation(audio_state, sound);
        }
    }

    instance.latest_frame = Some(frame.clone());
    if instance.dragging {
        frame.x = 0;
        frame.y = 0;
    } else {
        present_pet_frame(&app, &pet_instance_id, &mut frame)?;
    }

    Ok(frame)
}

fn pet_window_scale_factor(app: &AppHandle, pet_instance_id: &str) -> f64 {
    app.get_webview_window(&windows::pet_window_label(pet_instance_id))
        .and_then(|window| window.scale_factor().ok())
        .filter(|scale_factor| scale_factor.is_finite() && *scale_factor > 0.0)
        .unwrap_or(1.0)
}

fn runtime_area(
    work_area: Option<&platform::RectInfo>,
    viewport_width: i32,
    viewport_height: i32,
    dimensions: SpriteDimensions,
    scale_factor: f64,
) -> (i32, i32) {
    let width = work_area.map(|area| area.width).unwrap_or(viewport_width);
    let height = work_area.map(|area| area.height).unwrap_or(viewport_height);

    (
        adjusted_runtime_axis(width, dimensions.tile_width, scale_factor),
        adjusted_runtime_axis(height, dimensions.tile_height, scale_factor),
    )
}

fn adjusted_runtime_axis(area_size: i32, sprite_size: i32, scale_factor: f64) -> i32 {
    let visual_sprite_size = ((sprite_size as f64) * scale_factor).round() as i32;
    (area_size - (visual_sprite_size - sprite_size).max(0)).max(sprite_size)
}

fn detect_window_floor(runtime: &PetRuntime, work_area: &platform::RectInfo) -> Option<i32> {
    let (x, y, width, height) = runtime.current_bounds();
    let pet_bounds = platform::RectInfo {
        x: work_area.x + x,
        y: work_area.y + y,
        width,
        height,
    };

    platform::find_window_under_pet(pet_bounds).map(|window| window.y - work_area.y)
}

fn set_instance_interaction_animation(
    state: &State<'_, PetRuntimeState>,
    pet_instance_id: &str,
    animation_name: &str,
    dragging: bool,
) -> Result<bool, String> {
    let mut store = lock_store(state)?;
    let instance = store
        .instances
        .get_mut(pet_instance_id)
        .ok_or_else(|| format!("pet instance does not exist: {pet_instance_id}"))?;

    instance.paused = false;
    instance.dragging = dragging;

    let entered = if let Some(runtime) = instance.runtime.as_mut() {
        runtime.enter_animation_by_name(animation_name)?
    } else {
        instance.pending_animation_name = Some(animation_name.to_owned());
        true
    };

    if !entered && dragging {
        instance.dragging = false;
    }

    store.active_instance_id = Some(pet_instance_id.to_owned());

    Ok(entered)
}

fn apply_click_through_to_instance(
    app: &AppHandle,
    state: &State<'_, PetRuntimeState>,
    pet_instance_id: &str,
) -> Result<(), String> {
    let enabled = {
        let store = lock_store(state)?;
        store.click_through_enabled
    };

    windows::set_pet_click_through(app.clone(), pet_instance_id, enabled)
}

fn present_pet_frame(
    app: &AppHandle,
    pet_instance_id: &str,
    frame: &mut PetFrame,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&windows::pet_window_label(pet_instance_id)) {
        if let Ok(area) = platform::primary_work_area() {
            let placement = pet_window_placement(&area, frame, window.scale_factor().unwrap_or(1.0));

            window
                .set_position(PhysicalPosition::new(
                    placement.window_x,
                    placement.window_y,
                ))
                .map_err(|error| error.to_string())?;
            window
                .set_always_on_top(true)
                .map_err(|error| error.to_string())?;
            frame.x = placement.frame_x;
            frame.y = placement.frame_y;
            frame.width = placement.frame_width;
            frame.height = placement.frame_height;
        }
    }

    Ok(())
}

struct PetWindowPlacement {
    window_x: i32,
    window_y: i32,
    frame_x: i32,
    frame_y: i32,
    frame_width: i32,
    frame_height: i32,
}

fn pet_window_placement(
    area: &platform::RectInfo,
    frame: &PetFrame,
    scale_factor: f64,
) -> PetWindowPlacement {
    let frame_width = frame.width.clamp(1, PET_WINDOW_WIDTH);
    let frame_height = frame.height.clamp(1, PET_WINDOW_HEIGHT);
    let visual_screen_x = area.x + frame.x;
    let visual_screen_y = area.y + frame.y;
    let sprite_y_in_window =
        ((PET_WINDOW_HEIGHT - frame_height) as f64 * scale_factor).round() as i32;
    let window_y = (visual_screen_y - sprite_y_in_window).max(area.y);

    PetWindowPlacement {
        window_x: visual_screen_x,
        window_y,
        frame_x: 0,
        frame_y: (((visual_screen_y - window_y) as f64) / scale_factor).round() as i32,
        frame_width,
        frame_height,
    }
}

fn upsert_active_instance(
    state: &State<'_, PetRuntimeState>,
    xml_path: String,
) -> Result<String, String> {
    let resolved_path = resolve_xml_path(&xml_path)?;
    let mut store = lock_store(state)?;

    if let Some(pet_instance_id) = store.active_instance_id.clone() {
        if let Some(instance) = store.instances.get_mut(&pet_instance_id) {
            let changed = instance.xml_path != resolved_path;
            instance.xml_path = resolved_path;
            instance.paused = false;
            if changed {
                instance.runtime = None;
                instance.latest_frame = None;
            }
            return Ok(pet_instance_id);
        }
    }

    create_instance_in_store(&mut store, resolved_path)
}

fn create_instance(
    state: &State<'_, PetRuntimeState>,
    xml_path: String,
) -> Result<String, String> {
    let resolved_path = resolve_xml_path(&xml_path)?;
    let mut store = lock_store(state)?;

    create_instance_in_store(&mut store, resolved_path)
}

fn create_instance_in_store(
    store: &mut PetRuntimeStore,
    xml_path: PathBuf,
) -> Result<String, String> {
    if store.instances.len() >= store.max_instances {
        return Err(format!(
            "maximum pet instances reached: {}",
            store.max_instances
        ));
    }

    store.next_instance_number += 1;
    let pet_instance_id = format!("pet_{}", store.next_instance_number);
    store.instances.insert(
        pet_instance_id.clone(),
        PetRuntimeInstance {
            xml_path,
            runtime: None,
            paused: false,
            latest_frame: None,
            dragging: false,
            pending_animation_name: None,
        },
    );
    store.active_instance_id = Some(pet_instance_id.clone());

    Ok(pet_instance_id)
}

fn update_instance_pause(
    state: &State<'_, PetRuntimeState>,
    pet_instance_id: Option<String>,
    paused: bool,
) -> Result<String, String> {
    let mut store = lock_store(state)?;
    let pet_instance_id = resolve_instance_id(&store, pet_instance_id)?;
    let instance = store
        .instances
        .get_mut(&pet_instance_id)
        .ok_or_else(|| format!("pet instance does not exist: {pet_instance_id}"))?;

    instance.paused = paused;
    store.active_instance_id = Some(pet_instance_id.clone());

    Ok(pet_instance_id)
}

fn remove_instance(
    state: &State<'_, PetRuntimeState>,
    pet_instance_id: Option<String>,
) -> Result<String, String> {
    let mut store = lock_store(state)?;
    let pet_instance_id = resolve_instance_id(&store, pet_instance_id)?;

    store.instances.remove(&pet_instance_id);
    if store.active_instance_id.as_deref() == Some(&pet_instance_id) {
        store.active_instance_id = store.instances.keys().next().cloned();
    }

    Ok(pet_instance_id)
}

fn runtime_status(
    app: AppHandle,
    state: State<'_, PetRuntimeState>,
) -> Result<PetRuntimeStatus, String> {
    let store = lock_store(&state)?;
    Ok(runtime_status_from_store(&app, &store))
}

fn runtime_status_from_store(app: &AppHandle, store: &PetRuntimeStore) -> PetRuntimeStatus {
    let active_instance_id = store.active_instance_id.clone();
    let active_instance = active_instance_id
        .as_ref()
        .and_then(|id| store.instances.get(id));
    let instances = store
        .instances
        .iter()
        .map(|(id, instance)| PetInstanceStatus {
            pet_instance_id: id.clone(),
            xml_path: instance.xml_path.display().to_string(),
            is_paused: instance.paused,
            window_open: app
                .get_webview_window(&windows::pet_window_label(id))
                .is_some(),
            latest_frame: instance.latest_frame.clone(),
        })
        .collect::<Vec<_>>();
    let window_open = active_instance_id
        .as_ref()
        .map(|id| {
            app.get_webview_window(&windows::pet_window_label(id))
                .is_some()
        })
        .unwrap_or(false);
    let is_paused = active_instance
        .map(|instance| instance.paused)
        .unwrap_or(false);

    PetRuntimeStatus {
        selected_xml_path: active_instance.map(|instance| instance.xml_path.display().to_string()),
        active_instance_id,
        is_running: window_open && !is_paused,
        is_paused,
        window_open,
        latest_frame: active_instance.and_then(|instance| instance.latest_frame.clone()),
        max_instances: store.max_instances,
        window_collision_enabled: store.window_collision_enabled,
        click_through_enabled: store.click_through_enabled,
        instances,
    }
}

fn active_instance_id(store: &PetRuntimeStore) -> Result<String, String> {
    resolve_instance_id(store, None)
}

fn resolve_instance_id(
    store: &PetRuntimeStore,
    pet_instance_id: Option<String>,
) -> Result<String, String> {
    pet_instance_id
        .or_else(|| store.active_instance_id.clone())
        .ok_or_else(|| "no pet runtime has been started".to_owned())
}

fn lock_store<'a>(
    state: &'a State<'_, PetRuntimeState>,
) -> Result<std::sync::MutexGuard<'a, PetRuntimeStore>, String> {
    state
        .store
        .lock()
        .map_err(|_| "failed to lock pet runtime state".to_owned())
}

pub(crate) fn build_manifest_summary(
    resolved_path: PathBuf,
    manifest: PetManifest,
) -> Result<PetManifestSummary, String> {
    let sprite_base64 = manifest.image.png_base64.trim();
    let dimensions = sprite_dimensions(&manifest)?;

    Ok(PetManifestSummary {
        source_path: resolved_path.display().to_string(),
        header: PetHeaderSummary {
            author: manifest.header.author,
            title: manifest.header.title,
            petname: manifest.header.petname,
            version: manifest.header.version,
            info: manifest.header.info,
            application: manifest.header.application,
        },
        sprite_sheet: SpriteSheetSummary {
            tiles_x: manifest.image.tiles_x,
            tiles_y: manifest.image.tiles_y,
            tile_width: dimensions.tile_width,
            tile_height: dimensions.tile_height,
            transparency: manifest.image.transparency,
            data_url: format!("data:image/png;base64,{sprite_base64}"),
        },
        animation_count: manifest.animations.len(),
        spawn_count: manifest.spawns.len(),
        sound_count: manifest.sounds.len(),
    })
}

pub(crate) fn sprite_dimensions(manifest: &PetManifest) -> Result<SpriteDimensions, String> {
    let bytes = STANDARD
        .decode(manifest.image.png_base64.trim())
        .map_err(|error| format!("failed to decode sprite sheet base64 image: {error}"))?;
    let (sheet_width, sheet_height) = read_png_dimensions(&bytes)?;
    let tiles_x = manifest.image.tiles_x as i32;
    let tiles_y = manifest.image.tiles_y as i32;

    if tiles_x <= 0 || tiles_y <= 0 {
        return Err("sprite sheet tile grid must be greater than zero".to_owned());
    }

    Ok(SpriteDimensions {
        sheet_width,
        sheet_height,
        tile_width: sheet_width / tiles_x,
        tile_height: sheet_height / tiles_y,
    })
}

struct PetManifestIndex {
    header: PetHeaderSummary,
    animation_count: usize,
    spawn_count: usize,
    sound_count: usize,
}

fn load_pet_manifest_index(path: &PathBuf) -> Result<PetManifestIndex, String> {
    let xml = fs::read_to_string(path)
        .map_err(|error| format!("failed to read pet manifest index: {error}"))?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut in_header = false;
    let mut current_header_field: Option<&'static str> = None;
    let mut header = PetHeaderSummary {
        author: String::new(),
        title: String::new(),
        petname: String::new(),
        version: String::new(),
        info: String::new(),
        application: String::new(),
    };
    let mut animation_count = 0;
    let mut spawn_count = 0;
    let mut sound_count = 0;

    loop {
        match reader
            .read_event()
            .map_err(|error| format!("failed to parse pet manifest index: {error}"))?
        {
            Event::Start(event) => {
                let tag = event.name();
                let name = local_name(tag.as_ref());

                match name {
                    b"header" => in_header = true,
                    b"animation" => animation_count += 1,
                    b"spawn" => spawn_count += 1,
                    b"sound" => sound_count += 1,
                    b"author" if in_header => current_header_field = Some("author"),
                    b"title" if in_header => current_header_field = Some("title"),
                    b"petname" if in_header => current_header_field = Some("petname"),
                    b"version" if in_header => current_header_field = Some("version"),
                    b"info" if in_header => current_header_field = Some("info"),
                    b"application" if in_header => current_header_field = Some("application"),
                    _ => {}
                }
            }
            Event::Empty(event) => {
                let tag = event.name();
                match local_name(tag.as_ref()) {
                    b"animation" => animation_count += 1,
                    b"spawn" => spawn_count += 1,
                    b"sound" => sound_count += 1,
                    _ => {}
                }
            }
            Event::Text(text) => {
                if let Some(field) = current_header_field {
                    let value = text
                        .decode()
                        .map_err(|error| format!("failed to decode pet manifest text: {error}"))?
                        .trim()
                        .to_owned();
                    set_header_field(&mut header, field, value);
                }
            }
            Event::CData(text) => {
                if let Some(field) = current_header_field {
                    let value = text
                        .decode()
                        .map_err(|error| format!("failed to decode pet manifest CDATA: {error}"))?
                        .trim()
                        .to_owned();
                    set_header_field(&mut header, field, value);
                }
            }
            Event::End(event) => {
                let tag = event.name();
                let name = local_name(tag.as_ref());

                if name == b"header" {
                    in_header = false;
                }

                if current_header_field
                    .map(|field| name == field.as_bytes())
                    .unwrap_or(false)
                {
                    current_header_field = None;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(PetManifestIndex {
        header,
        animation_count,
        spawn_count,
        sound_count,
    })
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn set_header_field(header: &mut PetHeaderSummary, field: &str, value: String) {
    match field {
        "author" => header.author = value,
        "title" => header.title = value,
        "petname" => header.petname = value,
        "version" => header.version = value,
        "info" => header.info = value,
        "application" => header.application = value,
        _ => {}
    }
}

pub(crate) fn read_png_dimensions(bytes: &[u8]) -> Result<(i32, i32), String> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    if bytes.len() < 24 || &bytes[..8] != PNG_SIGNATURE {
        return Err("sprite sheet is not a valid PNG image".to_owned());
    }

    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);

    Ok((width as i32, height as i32))
}

fn resolve_xml_path(xml_path: &str) -> Result<PathBuf, String> {
    let requested_path = PathBuf::from(xml_path);

    if requested_path.is_absolute() {
        return existing_file(requested_path);
    }

    let mut candidates = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir.join(&requested_path));
        candidates.push(current_dir.join("..").join(&requested_path));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_dir.join(&requested_path));
    candidates.push(manifest_dir.join("..").join(&requested_path));
    candidates.push(manifest_dir.join("..").join("..").join(&requested_path));

    for candidate in candidates {
        if candidate.is_file() {
            return candidate
                .canonicalize()
                .map_err(|error| format!("failed to resolve pet XML path: {error}"));
        }
    }

    Err(format!("pet XML file does not exist: {xml_path}"))
}

pub(crate) fn resolve_pets_dir() -> Result<PathBuf, String> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir.join("Pets"));
        candidates.push(current_dir.join("..").join("Pets"));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_dir.join("Pets"));
    candidates.push(manifest_dir.join("..").join("Pets"));
    candidates.push(manifest_dir.join("..").join("..").join("Pets"));

    for candidate in candidates {
        if candidate.is_dir() {
            return candidate
                .canonicalize()
                .map_err(|error| format!("failed to resolve Pets directory: {error}"));
        }
    }

    Err("Pets directory does not exist".to_owned())
}

fn existing_file(path: PathBuf) -> Result<PathBuf, String> {
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => path
            .canonicalize()
            .map_err(|error| format!("failed to resolve pet XML path: {error}")),
        Ok(_) => Err(format!("pet XML path is not a file: {}", path.display())),
        Err(error) => Err(format!(
            "pet XML file does not exist: {} ({error})",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        adjusted_runtime_axis, create_instance_in_store, list_available_pets_from_dirs,
        load_pet_manifest, load_pet_manifest_index, pet_window_placement, PetRuntimeStore,
    };
    use crate::{
        commands::pet_assets::{PetAssetSource, PetDirectory},
        pet::runtime::PetFrame,
        platform::RectInfo,
    };

    #[test]
    fn creates_unique_instances_until_configured_limit() {
        let mut store = PetRuntimeStore {
            max_instances: 2,
            ..PetRuntimeStore::default()
        };
        let first = create_instance_in_store(&mut store, PathBuf::from("a.xml")).expect("first");
        let second = create_instance_in_store(&mut store, PathBuf::from("b.xml")).expect("second");
        let error = create_instance_in_store(&mut store, PathBuf::from("c.xml"))
            .expect_err("limit should fail");

        assert_ne!(first, second);
        assert_eq!(2, store.instances.len());
        assert!(error.contains("maximum pet instances reached"));
    }

    #[test]
    fn lists_esheep_pet_from_pets_directory() {
        let pets = list_available_pets_from_dirs(vec![PetDirectory {
            id: "esheep64".to_owned(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("Pets")
                .join("esheep64"),
            source: PetAssetSource::Builtin,
        }])
        .expect("pets should scan");
        let esheep = pets
            .iter()
            .find(|pet| pet.id == "esheep64")
            .expect("esheep64 should be listed");

        assert!(esheep.xml_path.ends_with("animations.xml"));
        assert_eq!("eSheep", esheep.header.petname);
    }

    #[test]
    fn loads_esheep_manifest_summary() {
        let manifest = load_pet_manifest("../Pets/esheep64/animations.xml".to_owned())
            .expect("manifest should load");

        assert_eq!("eSheep", manifest.header.petname);
        assert_eq!("eSheep 64bit", manifest.header.title);
        assert_eq!(16, manifest.sprite_sheet.tiles_x);
        assert_eq!(11, manifest.sprite_sheet.tiles_y);
        assert_eq!(40, manifest.sprite_sheet.tile_width);
        assert_eq!(40, manifest.sprite_sheet.tile_height);
        assert_eq!(54, manifest.animation_count);
        assert_eq!(4, manifest.spawn_count);
        assert!(manifest
            .sprite_sheet
            .data_url
            .starts_with("data:image/png;base64,"));
    }

    #[test]
    fn indexes_esheep_manifest_without_loading_full_summary() {
        let index = load_pet_manifest_index(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("Pets")
                .join("esheep64")
                .join("animations.xml"),
        )
            .expect("index should load");

        assert_eq!("eSheep", index.header.petname);
        assert_eq!("eSheep 64bit", index.header.title);
        assert_eq!(54, index.animation_count);
        assert_eq!(4, index.spawn_count);
        assert_eq!(0, index.sound_count);
    }

    #[test]
    fn places_sprite_bottom_on_work_area_floor() {
        let area = RectInfo {
            x: 0,
            y: 0,
            width: 1920,
            height: 1032,
        };
        let frame = test_frame(500, 992, 40, 40);

        let placement = pet_window_placement(&area, &frame, 1.0);

        assert_eq!(872, placement.window_y);
        assert_eq!(120, placement.frame_y);
        assert_eq!(1032, placement.window_y + placement.frame_y + placement.frame_height);
    }

    #[test]
    fn scales_sprite_offset_for_high_dpi_windows() {
        let area = RectInfo {
            x: 0,
            y: 0,
            width: 1920,
            height: 1032,
        };
        let frame = test_frame(500, 992, 40, 40);

        let placement = pet_window_placement(&area, &frame, 1.25);

        assert_eq!(842, placement.window_y);
        assert_eq!(120, placement.frame_y);
        assert_eq!(992, placement.window_y + (placement.frame_y as f64 * 1.25).round() as i32);
    }

    #[test]
    fn reduces_runtime_floor_for_high_dpi_sprite_height() {
        let area_height = 1032;
        let sprite_height = 40;

        let adjusted_height = adjusted_runtime_axis(area_height, sprite_height, 1.25);

        assert_eq!(1022, adjusted_height);
        assert_eq!(982, adjusted_height - sprite_height);
        assert_eq!(1032, 982 + ((sprite_height as f64) * 1.25).round() as i32);
    }

    #[test]
    fn keeps_runtime_floor_for_regular_dpi_sprite_height() {
        assert_eq!(1032, adjusted_runtime_axis(1032, 40, 1.0));
    }

    #[test]
    fn clips_spawn_frames_above_work_area_without_moving_sprite_down() {
        let area = RectInfo {
            x: 0,
            y: 0,
            width: 1920,
            height: 1032,
        };
        let frame = test_frame(500, -60, 40, 40);

        let placement = pet_window_placement(&area, &frame, 1.0);

        assert_eq!(0, placement.window_y);
        assert_eq!(-60, placement.frame_y);
    }

    #[test]
    fn reports_missing_manifest_path() {
        let error = load_pet_manifest("../Pets/missing/animations.xml".to_owned())
            .expect_err("missing manifest should fail");

        assert!(error.contains("pet XML file does not exist"));
    }

    fn test_frame(x: i32, y: i32, width: i32, height: i32) -> PetFrame {
        PetFrame {
            animation_id: 1,
            animation_name: "test".to_owned(),
            frame_index: 0,
            sequence_step: 0,
            total_steps: 1,
            x,
            y,
            width,
            height,
            interval_ms: 100,
            offset_y: 0,
            opacity: 1.0,
            flipped: false,
        }
    }
}
