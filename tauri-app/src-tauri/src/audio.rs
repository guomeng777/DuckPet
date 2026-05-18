use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use tauri::State;

use crate::pet::model::Sound;

#[derive(Clone, Default)]
pub struct AudioState {
    inner: Arc<Mutex<AudioInner>>,
}

#[derive(Default)]
struct AudioInner {
    muted: bool,
    last_error: Option<String>,
    next_alias_id: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStatus {
    pub muted: bool,
    pub last_error: Option<String>,
}

#[tauri::command]
pub fn get_audio_status(state: State<'_, AudioState>) -> Result<AudioStatus, String> {
    audio_status(&state)
}

#[tauri::command]
pub fn set_audio_muted(
    state: State<'_, AudioState>,
    muted: bool,
) -> Result<AudioStatus, String> {
    {
        let mut inner = lock_audio(&state)?;
        inner.muted = muted;
        if muted {
            inner.last_error = None;
        }
    }

    audio_status(&state)
}

pub fn play_sound_for_animation(state: &State<'_, AudioState>, sound: Sound) {
    let (inner, alias_id) = match reserve_alias(state) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("audio disabled: {error}");
            return;
        }
    };

    thread::spawn(move || {
        if let Err(error) = play_sound_file(alias_id, sound) {
            record_error(&inner, Some(error));
        }
    });
}

fn reserve_alias(state: &State<'_, AudioState>) -> Result<(Arc<Mutex<AudioInner>>, u64), String> {
    let mut inner = lock_audio(state)?;

    if inner.muted {
        return Err("muted".to_owned());
    }

    inner.next_alias_id = inner.next_alias_id.wrapping_add(1);
    inner.last_error = None;

    Ok((state.inner.clone(), inner.next_alias_id))
}

fn audio_status(state: &State<'_, AudioState>) -> Result<AudioStatus, String> {
    let inner = lock_audio(state)?;

    Ok(AudioStatus {
        muted: inner.muted,
        last_error: inner.last_error.clone(),
    })
}

fn lock_audio<'a>(
    state: &'a State<'_, AudioState>,
) -> Result<std::sync::MutexGuard<'a, AudioInner>, String> {
    state
        .inner
        .lock()
        .map_err(|_| "failed to lock audio state".to_owned())
}

fn record_error(inner: &Arc<Mutex<AudioInner>>, error: Option<String>) {
    if let Ok(mut inner) = inner.lock() {
        inner.last_error = error;
    }
}

fn play_sound_file(alias_id: u64, sound: Sound) -> Result<(), String> {
    let bytes = decode_base64_audio(&sound.base64)?;
    let path = write_temp_audio(alias_id, &bytes)?;
    let alias = format!("duckpet_sound_{alias_id}");
    let loop_count = sound.loop_count.max(0);

    let result = play_with_platform_backend(&path, &alias, loop_count);
    let _ = fs::remove_file(&path);
    result
}

fn decode_base64_audio(base64_audio: &str) -> Result<Vec<u8>, String> {
    let trimmed = base64_audio.trim();
    let payload = trimmed
        .find(";base64,")
        .map(|index| &trimmed[index + 8..])
        .unwrap_or(trimmed);

    STANDARD
        .decode(payload)
        .map_err(|error| format!("failed to decode sound base64: {error}"))
}

fn write_temp_audio(alias_id: u64, bytes: &[u8]) -> Result<PathBuf, String> {
    let audio_dir = std::env::temp_dir().join("DuckPet").join("audio");
    fs::create_dir_all(&audio_dir)
        .map_err(|error| format!("failed to create temp audio directory: {error}"))?;

    let path = audio_dir.join(format!("sound-{alias_id}.mp3"));
    fs::write(&path, bytes).map_err(|error| format!("failed to write temp sound file: {error}"))?;

    Ok(path)
}

#[cfg(windows)]
fn play_with_platform_backend(path: &PathBuf, alias: &str, loop_count: i32) -> Result<(), String> {
    let path = path
        .to_str()
        .ok_or_else(|| "temp sound path is not valid UTF-8".to_owned())?;
    let open = format!("open \"{path}\" type mpegvideo alias {alias}");

    mci_send(&open)?;

    let play_result = (0..=loop_count).try_for_each(|_| mci_send(&format!("play {alias} wait")));
    let close_result = mci_send(&format!("close {alias}"));

    play_result.and(close_result)
}

#[cfg(windows)]
fn mci_send(command: &str) -> Result<(), String> {
    use windows::{core::HSTRING, Win32::Media::Multimedia::mciSendStringW};

    let result = unsafe { mciSendStringW(&HSTRING::from(command), None, None) };

    if result == 0 {
        Ok(())
    } else {
        Err(format!("Windows MCI error {result} while running `{command}`"))
    }
}

#[cfg(not(windows))]
fn play_with_platform_backend(_path: &PathBuf, _alias: &str, _loop_count: i32) -> Result<(), String> {
    Err("sound playback is only implemented on Windows".to_owned())
}

#[cfg(test)]
mod tests {
    use super::decode_base64_audio;

    #[test]
    fn decodes_plain_base64_audio_payload() {
        let bytes = decode_base64_audio("AQIDBA==").expect("decode");

        assert_eq!(bytes, vec![1, 2, 3, 4]);
    }

    #[test]
    fn decodes_data_url_base64_audio_payload() {
        let bytes = decode_base64_audio("data:audio/mpeg;base64,AQIDBA==").expect("decode");

        assert_eq!(bytes, vec![1, 2, 3, 4]);
    }
}
