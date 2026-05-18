use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::commands::pet::{
    read_png_dimensions, resolve_pets_dir, sprite_dimensions,
};
use crate::pet::{model::PetManifest, xml::parse_pet_manifest};

const USER_PETS_DIR_NAME: &str = "UserPets";
const MAX_ARCHIVE_BYTES: usize = 20 * 1024 * 1024;
const MAX_ARCHIVE_FILES: usize = 64;
const MAX_EXTRACTED_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct PetDirectory {
    pub id: String,
    pub path: PathBuf,
    pub source: PetAssetSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PetAssetSource {
    Builtin,
    User,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageablePet {
    pub id: String,
    pub display_name: String,
    pub author: String,
    pub xml_path: String,
    pub source: PetAssetSource,
    pub can_delete: bool,
    pub animation_count: usize,
    pub spawn_count: usize,
    pub sound_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetArchiveValidation {
    pub valid: bool,
    pub normalized_id: String,
    pub display_name: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub summary: Option<PetArchiveSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetArchiveSummary {
    pub author: String,
    pub title: String,
    pub pet_name: String,
    pub version: String,
    pub animation_count: usize,
    pub spawn_count: usize,
    pub sound_count: usize,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub tile_width: i32,
    pub tile_height: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedPetArchive {
    pub file_name: String,
    pub archive_base64: String,
}

#[derive(Debug)]
struct ValidatedArchive {
    normalized_id: String,
    display_name: String,
    manifest: PetManifest,
    entries: Vec<ArchiveEntry>,
    warnings: Vec<String>,
}

#[derive(Debug)]
struct ArchiveEntry {
    relative_path: PathBuf,
    contents: Vec<u8>,
}

#[tauri::command]
pub fn list_manageable_pets(app: AppHandle) -> Result<Vec<ManageablePet>, String> {
    let mut pets = Vec::new();

    for pet_dir in available_pet_directories(&app)? {
        let xml_path = pet_dir.path.join("animations.xml");
        if !xml_path.is_file() {
            continue;
        }

        let manifest = crate::pet::xml::parse_pet_manifest_file(&xml_path)
            .map_err(|error| format!("failed to parse pet {}: {error}", pet_dir.id))?;
        let resolved_path = xml_path
            .canonicalize()
            .map_err(|error| format!("failed to resolve pet XML path: {error}"))?;

        pets.push(ManageablePet {
            id: pet_dir.id,
            display_name: manifest.header.petname.clone(),
            author: manifest.header.author.clone(),
            xml_path: resolved_path.display().to_string(),
            source: pet_dir.source,
            can_delete: pet_dir.source == PetAssetSource::User,
            animation_count: manifest.animations.len(),
            spawn_count: manifest.spawns.len(),
            sound_count: manifest.sounds.len(),
        });
    }

    pets.sort_by(|left, right| {
        left.source
            .source_order()
            .cmp(&right.source.source_order())
            .then_with(|| left.id.cmp(&right.id))
    });

    Ok(pets)
}

#[tauri::command]
pub fn validate_pet_archive(name: String, archive_base64: String) -> PetArchiveValidation {
    match validate_archive(&name, &archive_base64) {
        Ok(validated) => validation_success(validated),
        Err(errors) => PetArchiveValidation {
            valid: false,
            normalized_id: normalize_pet_id(&name).unwrap_or_default(),
            display_name: name.trim().to_owned(),
            errors,
            warnings: Vec::new(),
            summary: None,
        },
    }
}

#[tauri::command]
pub fn save_uploaded_pet(
    app: AppHandle,
    name: String,
    archive_base64: String,
) -> Result<ManageablePet, String> {
    let validated = validate_archive(&name, &archive_base64).map_err(|errors| errors.join("\n"))?;
    let user_pets_dir = resolve_user_pets_dir(&app)?;
    let target_dir = user_pets_dir.join(&validated.normalized_id);

    if target_dir.exists() || pet_id_exists(&app, &validated.normalized_id)? {
        return Err(format!("pet already exists: {}", validated.normalized_id));
    }

    fs::create_dir_all(&user_pets_dir)
        .map_err(|error| format!("failed to create user pet directory: {error}"))?;

    let tmp_dir = user_pets_dir.join(format!(
        ".upload-{}-{}",
        validated.normalized_id,
        unique_suffix()
    ));
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)
            .map_err(|error| format!("failed to reset temporary pet directory: {error}"))?;
    }
    fs::create_dir_all(&tmp_dir)
        .map_err(|error| format!("failed to create temporary pet directory: {error}"))?;

    if let Err(error) = write_validated_entries(&tmp_dir, &validated.entries) {
        let _ = fs::remove_dir_all(&tmp_dir);
        return Err(error);
    }

    fs::rename(&tmp_dir, &target_dir)
        .map_err(|error| format!("failed to save uploaded pet: {error}"))?;

    manageable_pet_from_dir(PetDirectory {
        id: validated.normalized_id,
        path: target_dir,
        source: PetAssetSource::User,
    })
}

#[tauri::command]
pub fn delete_uploaded_pet(app: AppHandle, pet_id: String) -> Result<(), String> {
    let normalized_id = normalize_pet_id(&pet_id)?;
    let user_pets_dir = resolve_user_pets_dir(&app)?;
    let target_dir = user_pets_dir.join(&normalized_id);
    let resolved_user_dir = user_pets_dir
        .canonicalize()
        .map_err(|error| format!("failed to resolve user pets directory: {error}"))?;
    let resolved_target = target_dir
        .canonicalize()
        .map_err(|error| format!("user pet does not exist: {normalized_id} ({error})"))?;

    if !resolved_target.starts_with(&resolved_user_dir) {
        return Err("refusing to delete a path outside the user pet directory".to_owned());
    }

    if resolve_builtin_pet_dir(&normalized_id)?.is_some() {
        return Err(format!("built-in pet cannot be deleted: {normalized_id}"));
    }

    fs::remove_dir_all(&resolved_target)
        .map_err(|error| format!("failed to delete user pet {normalized_id}: {error}"))
}

#[tauri::command]
pub fn export_pet_archive(app: AppHandle, pet_id: String) -> Result<ExportedPetArchive, String> {
    let pet_dir = find_pet_directory(&app, &pet_id)?;
    let archive_bytes = zip_pet_directory(&pet_dir.path)?;

    Ok(ExportedPetArchive {
        file_name: format!("{}.zip", pet_dir.id),
        archive_base64: STANDARD.encode(archive_bytes),
    })
}

pub(crate) fn available_pet_directories(app: &AppHandle) -> Result<Vec<PetDirectory>, String> {
    let mut directories = Vec::new();
    collect_pet_directories(&resolve_pets_dir()?, PetAssetSource::Builtin, &mut directories)?;

    let user_pets_dir = resolve_user_pets_dir(app)?;
    if user_pets_dir.is_dir() {
        collect_pet_directories(&user_pets_dir, PetAssetSource::User, &mut directories)?;
    }

    Ok(directories)
}

fn collect_pet_directories(
    parent: &Path,
    source: PetAssetSource,
    output: &mut Vec<PetDirectory>,
) -> Result<(), String> {
    for entry in fs::read_dir(parent)
        .map_err(|error| format!("failed to read pet directory {}: {error}", parent.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read pet entry: {error}"))?;
        let metadata = entry
            .metadata()
            .map_err(|error| format!("failed to read pet entry metadata: {error}"))?;

        if !metadata.is_dir() {
            continue;
        }

        if !entry.path().join("animations.xml").is_file() {
            continue;
        }

        output.push(PetDirectory {
            id: entry.file_name().to_string_lossy().to_string(),
            path: entry.path(),
            source,
        });
    }

    Ok(())
}

fn manageable_pet_from_dir(pet_dir: PetDirectory) -> Result<ManageablePet, String> {
    let xml_path = pet_dir.path.join("animations.xml");
    let manifest = crate::pet::xml::parse_pet_manifest_file(&xml_path)
        .map_err(|error| format!("failed to parse saved pet: {error}"))?;
    let resolved_path = xml_path
        .canonicalize()
        .map_err(|error| format!("failed to resolve saved pet XML path: {error}"))?;

    Ok(ManageablePet {
        id: pet_dir.id,
        display_name: manifest.header.petname.clone(),
        author: manifest.header.author.clone(),
        xml_path: resolved_path.display().to_string(),
        source: pet_dir.source,
        can_delete: pet_dir.source == PetAssetSource::User,
        animation_count: manifest.animations.len(),
        spawn_count: manifest.spawns.len(),
        sound_count: manifest.sounds.len(),
    })
}

fn validate_archive(name: &str, archive_base64: &str) -> Result<ValidatedArchive, Vec<String>> {
    let normalized_id = normalize_pet_id(name).map_err(|error| vec![error])?;
    let display_name = name.trim().to_owned();
    let archive_bytes = decode_archive_base64(archive_base64).map_err(|error| vec![error])?;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if archive_bytes.len() > MAX_ARCHIVE_BYTES {
        errors.push(format!(
            "ZIP archive is too large: {} bytes (limit: {MAX_ARCHIVE_BYTES})",
            archive_bytes.len()
        ));
    }

    let mut archive = ZipArchive::new(Cursor::new(&archive_bytes))
        .map_err(|error| vec![format!("failed to read ZIP archive: {error}")])?;

    if archive.len() > MAX_ARCHIVE_FILES {
        errors.push(format!(
            "ZIP archive has too many files: {} (limit: {MAX_ARCHIVE_FILES})",
            archive.len()
        ));
    }

    let mut entries = Vec::new();
    let mut manifest_candidates = Vec::new();
    let mut extracted_bytes = 0_u64;
    let mut has_icon = false;
    let mut has_readme = false;

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| vec![format!("failed to read ZIP entry {index}: {error}")])?;
        let Some(path) = file.enclosed_name().map(PathBuf::from) else {
            errors.push(format!("ZIP entry has an unsafe path: {}", file.name()));
            continue;
        };

        if !is_safe_relative_path(&path) {
            errors.push(format!("ZIP entry has an unsafe path: {}", file.name()));
            continue;
        }

        if file.is_dir() {
            continue;
        }

        extracted_bytes = extracted_bytes.saturating_add(file.size());
        if extracted_bytes > MAX_EXTRACTED_BYTES {
            errors.push(format!(
                "ZIP extracted content is too large: {extracted_bytes} bytes (limit: {MAX_EXTRACTED_BYTES})"
            ));
            break;
        }

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|error| vec![format!("failed to read ZIP entry {}: {error}", file.name())])?;

        if file_name_eq(&path, "animations.xml") {
            manifest_candidates.push(path.clone());
        }
        if file_name_eq(&path, "icon.png") {
            has_icon = true;
        }
        if file_name_eq(&path, "README.md") || file_name_eq(&path, "readme.md") {
            has_readme = true;
        }

        entries.push(ArchiveEntry {
            relative_path: path,
            contents,
        });
    }

    if !has_icon {
        warnings.push("icon.png is recommended for previewing the pet".to_owned());
    }
    if !has_readme {
        warnings.push("README.md is recommended for credits and usage notes".to_owned());
    }

    let manifest_path = match manifest_candidates.as_slice() {
        [] => {
            errors.push("ZIP archive must contain animations.xml".to_owned());
            PathBuf::new()
        }
        [path] if valid_manifest_depth(path) => path.clone(),
        [_] => {
            errors.push("animations.xml must be at the ZIP root or inside one top-level folder".to_owned());
            PathBuf::new()
        }
        _ => {
            errors.push("ZIP archive must contain exactly one animations.xml".to_owned());
            PathBuf::new()
        }
    };

    if !errors.is_empty() {
        return Err(errors);
    }

    let manifest_prefix = manifest_path.parent().unwrap_or_else(|| Path::new(""));
    if !manifest_prefix.as_os_str().is_empty() {
        for entry in &entries {
            if !entry.relative_path.starts_with(manifest_prefix) {
                return Err(vec![
                    "all ZIP files must be under the same top-level pet folder".to_owned(),
                ]);
            }
        }
    }

    let manifest_xml = entries
        .iter()
        .find(|entry| entry.relative_path == manifest_path)
        .ok_or_else(|| vec!["animations.xml was not found after ZIP validation".to_owned()])?;
    let manifest_text = std::str::from_utf8(&manifest_xml.contents)
        .map_err(|error| vec![format!("animations.xml must be valid UTF-8: {error}")])?;
    let manifest = parse_pet_manifest(manifest_text)
        .map_err(|error| vec![format!("failed to parse animations.xml: {error}")])?;

    validate_manifest(&manifest).map_err(|error| vec![error])?;

    let normalized_entries = entries
        .into_iter()
        .map(|entry| ArchiveEntry {
            relative_path: strip_manifest_prefix(&entry.relative_path, manifest_prefix),
            contents: entry.contents,
        })
        .collect::<Vec<_>>();

    Ok(ValidatedArchive {
        normalized_id,
        display_name,
        manifest,
        entries: normalized_entries,
        warnings,
    })
}

fn validation_success(validated: ValidatedArchive) -> PetArchiveValidation {
    let dimensions = sprite_dimensions(&validated.manifest).ok();

    PetArchiveValidation {
        valid: true,
        normalized_id: validated.normalized_id,
        display_name: validated.display_name,
        errors: Vec::new(),
        warnings: validated.warnings,
        summary: Some(PetArchiveSummary {
            author: validated.manifest.header.author,
            title: validated.manifest.header.title,
            pet_name: validated.manifest.header.petname,
            version: validated.manifest.header.version,
            animation_count: validated.manifest.animations.len(),
            spawn_count: validated.manifest.spawns.len(),
            sound_count: validated.manifest.sounds.len(),
            tiles_x: validated.manifest.image.tiles_x,
            tiles_y: validated.manifest.image.tiles_y,
            tile_width: dimensions.map(|value| value.tile_width).unwrap_or_default(),
            tile_height: dimensions.map(|value| value.tile_height).unwrap_or_default(),
        }),
    }
}

fn validate_manifest(manifest: &PetManifest) -> Result<(), String> {
    require_field("header.author", &manifest.header.author)?;
    require_field("header.title", &manifest.header.title)?;
    require_field("header.petname", &manifest.header.petname)?;
    require_field("header.version", &manifest.header.version)?;
    require_field("header.info", &manifest.header.info)?;
    require_field("header.application", &manifest.header.application)?;
    require_field("header.icon", &manifest.header.icon_base64)?;

    if manifest.image.tiles_x == 0 || manifest.image.tiles_y == 0 {
        return Err("image.tilesx and image.tilesy must be greater than zero".to_owned());
    }

    let image_bytes = STANDARD
        .decode(manifest.image.png_base64.trim())
        .map_err(|error| format!("image.png base64 is invalid: {error}"))?;
    let (width, height) = read_png_dimensions(&image_bytes)?;

    if width <= 0 || height <= 0 {
        return Err("sprite PNG dimensions must be greater than zero".to_owned());
    }

    if width % manifest.image.tiles_x as i32 != 0 {
        return Err(format!(
            "sprite PNG width {width}px is not divisible by tilesx {}",
            manifest.image.tiles_x
        ));
    }
    if height % manifest.image.tiles_y as i32 != 0 {
        return Err(format!(
            "sprite PNG height {height}px is not divisible by tilesy {}",
            manifest.image.tiles_y
        ));
    }

    if manifest.spawns.is_empty() {
        return Err("at least one spawn is required".to_owned());
    }
    if manifest.animations.is_empty() {
        return Err("at least one animation is required".to_owned());
    }

    Ok(())
}

fn require_field(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} is required"))
    } else {
        Ok(())
    }
}

fn write_validated_entries(target_dir: &Path, entries: &[ArchiveEntry]) -> Result<(), String> {
    for entry in entries {
        if !is_safe_relative_path(&entry.relative_path) {
            return Err(format!(
                "refusing to write unsafe ZIP entry: {}",
                entry.relative_path.display()
            ));
        }

        let target_path = target_dir.join(&entry.relative_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create pet directory: {error}"))?;
        }
        fs::write(&target_path, &entry.contents)
            .map_err(|error| format!("failed to write pet file {}: {error}", target_path.display()))?;
    }

    Ok(())
}

fn zip_pet_directory(pet_dir: &Path) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
    add_directory_to_zip(&mut writer, pet_dir, Path::new(""), options)?;
    add_sprite_sheet_reference_to_zip(&mut writer, pet_dir, Path::new(""), options)?;
    writer
        .finish()
        .map_err(|error| format!("failed to finish ZIP archive: {error}"))
        .map(|cursor| cursor.into_inner())
}

fn add_directory_to_zip(
    writer: &mut ZipWriter<Cursor<Vec<u8>>>,
    source_dir: &Path,
    zip_prefix: &Path,
    options: FileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(source_dir)
        .map_err(|error| format!("failed to read pet directory {}: {error}", source_dir.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read pet file: {error}"))?;
        let path = entry.path();
        let zip_path = zip_prefix.join(entry.file_name());

        if path.is_dir() {
            add_directory_to_zip(writer, &path, &zip_path, options)?;
            continue;
        }

        let zip_name = zip_path.to_string_lossy().replace('\\', "/");
        writer
            .start_file(zip_name, options)
            .map_err(|error| format!("failed to add ZIP file: {error}"))?;
        let bytes = fs::read(&path)
            .map_err(|error| format!("failed to read pet file {}: {error}", path.display()))?;
        writer
            .write_all(&bytes)
            .map_err(|error| format!("failed to write ZIP file: {error}"))?;
    }

    Ok(())
}

fn add_sprite_sheet_reference_to_zip(
    writer: &mut ZipWriter<Cursor<Vec<u8>>>,
    pet_dir: &Path,
    zip_prefix: &Path,
    options: FileOptions,
) -> Result<(), String> {
    let sprite_path = pet_dir.join("sprite-sheet.png");
    if sprite_path.exists() {
        return Ok(());
    }

    let manifest = crate::pet::xml::parse_pet_manifest_file(pet_dir.join("animations.xml"))
        .map_err(|error| format!("failed to parse pet manifest for sprite export: {error}"))?;
    let sprite_bytes = STANDARD
        .decode(manifest.image.png_base64.trim())
        .map_err(|error| format!("failed to decode sprite sheet for export: {error}"))?;
    read_png_dimensions(&sprite_bytes)?;

    let zip_name = zip_prefix
        .join("sprite-sheet.png")
        .to_string_lossy()
        .replace('\\', "/");
    writer
        .start_file(zip_name, options)
        .map_err(|error| format!("failed to add sprite sheet ZIP file: {error}"))?;
    writer
        .write_all(&sprite_bytes)
        .map_err(|error| format!("failed to write sprite sheet ZIP file: {error}"))
}

fn find_pet_directory(app: &AppHandle, pet_id: &str) -> Result<PetDirectory, String> {
    let normalized_id = normalize_pet_id(pet_id)?;
    available_pet_directories(app)?
        .into_iter()
        .find(|pet_dir| pet_dir.id == normalized_id)
        .ok_or_else(|| format!("pet does not exist: {normalized_id}"))
}

fn pet_id_exists(app: &AppHandle, pet_id: &str) -> Result<bool, String> {
    Ok(available_pet_directories(app)?
        .into_iter()
        .any(|pet_dir| pet_dir.id == pet_id))
}

fn resolve_builtin_pet_dir(pet_id: &str) -> Result<Option<PathBuf>, String> {
    let pets_dir = resolve_pets_dir()?;
    let candidate = pets_dir.join(pet_id);

    if candidate.join("animations.xml").is_file() {
        Ok(Some(candidate))
    } else {
        Ok(None)
    }
}

fn resolve_user_pets_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join(USER_PETS_DIR_NAME))
        .map_err(|error| format!("failed to resolve app data directory: {error}"))
}

fn decode_archive_base64(value: &str) -> Result<Vec<u8>, String> {
    let trimmed = value.trim();
    let payload = trimmed
        .split_once(',')
        .map(|(_, payload)| payload)
        .unwrap_or(trimmed);

    STANDARD
        .decode(payload)
        .map_err(|error| format!("ZIP base64 is invalid: {error}"))
}

fn normalize_pet_id(name: &str) -> Result<String, String> {
    let normalized = name.trim().replace(' ', "_").to_lowercase();

    if normalized.is_empty() {
        return Err("pet name is required".to_owned());
    }
    if normalized == "." || normalized == ".." {
        return Err("pet name is reserved".to_owned());
    }
    if normalized.len() > 64 {
        return Err("pet name is too long; use at most 64 characters".to_owned());
    }
    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(
            "pet name may only contain letters, numbers, spaces, underscores, and hyphens"
                .to_owned(),
        );
    }

    Ok(normalized)
}

fn is_safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path.is_relative()
        && path.components().all(|component| {
            matches!(component, Component::Normal(_))
                || matches!(component, Component::CurDir)
        })
}

fn valid_manifest_depth(path: &Path) -> bool {
    let count = path.components().count();
    count == 1 || count == 2
}

fn strip_manifest_prefix(path: &Path, prefix: &Path) -> PathBuf {
    if prefix.as_os_str().is_empty() {
        path.to_path_buf()
    } else {
        path.strip_prefix(prefix).unwrap_or(path).to_path_buf()
    }
}

fn file_name_eq(path: &Path, expected: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

impl PetAssetSource {
    fn source_order(self) -> u8 {
        match self {
            Self::Builtin => 0,
            Self::User => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG_1X1_BASE64: &str =
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    #[test]
    fn normalizes_safe_pet_names() {
        assert_eq!("my_pet-1", normalize_pet_id("My Pet-1").expect("valid"));
        assert!(normalize_pet_id("../bad").is_err());
        assert!(normalize_pet_id("").is_err());
    }

    #[test]
    fn validates_good_archive() {
        let archive = build_test_archive("sample/animations.xml", &valid_manifest_xml());
        let result = validate_archive("Sample Pet", &STANDARD.encode(archive)).expect("valid");

        assert_eq!("sample_pet", result.normalized_id);
        assert!(result
            .entries
            .iter()
            .any(|entry| entry.relative_path == PathBuf::from("animations.xml")));
        assert_eq!(1, result.manifest.animations.len());
    }

    #[test]
    fn rejects_archive_without_manifest() {
        let archive = build_test_archive("README.md", "missing");
        let errors = validate_archive("Missing", &STANDARD.encode(archive)).expect_err("invalid");

        assert!(errors
            .iter()
            .any(|error| error.contains("must contain animations.xml")));
    }

    #[test]
    fn rejects_invalid_png_base64() {
        let xml = valid_manifest_xml().replace(PNG_1X1_BASE64, "not-base64");
        let archive = build_test_archive("animations.xml", &xml);
        let errors = validate_archive("Bad Png", &STANDARD.encode(archive)).expect_err("invalid");

        assert!(errors
            .iter()
            .any(|error| error.contains("image.png base64 is invalid")));
    }

    #[test]
    fn rejects_path_traversal_entries() {
        let archive = build_raw_zip(vec![("../animations.xml", valid_manifest_xml().into_bytes())]);
        let errors = validate_archive("Unsafe", &STANDARD.encode(archive)).expect_err("invalid");

        assert!(errors
            .iter()
            .any(|error| error.contains("must contain animations.xml")));
    }

    #[test]
    fn rejects_zero_tile_grid() {
        let xml = valid_manifest_xml().replace("<tilesx>1</tilesx>", "<tilesx>0</tilesx>");
        let archive = build_test_archive("animations.xml", &xml);
        let errors = validate_archive("Bad Tiles", &STANDARD.encode(archive)).expect_err("invalid");

        assert!(errors
            .iter()
            .any(|error| error.contains("tilesx and image.tilesy")));
    }

    fn build_test_archive(path: &str, contents: &str) -> Vec<u8> {
        build_raw_zip(vec![(path, contents.as_bytes().to_vec())])
    }

    fn build_raw_zip(files: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(cursor);
        let options = FileOptions::default().compression_method(CompressionMethod::Stored);

        for (path, contents) in files {
            writer.start_file(path, options).expect("start file");
            writer.write_all(&contents).expect("write file");
        }

        writer.finish().expect("finish").into_inner()
    }

    fn valid_manifest_xml() -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<animations>
  <header>
    <author>Tester</author>
    <title>Test Pet</title>
    <petname>Test Pet</petname>
    <version>1.0</version>
    <info>Test info</info>
    <application>1</application>
    <icon>AA==</icon>
  </header>
  <image>
    <tilesx>1</tilesx>
    <tilesy>1</tilesy>
    <png>{PNG_1X1_BASE64}</png>
    <transparency>Magenta</transparency>
  </image>
  <spawns>
    <spawn id="1" probability="100">
      <x>0</x>
      <y>0</y>
      <next probability="100">1</next>
    </spawn>
  </spawns>
  <animations>
    <animation id="1">
      <name>idle</name>
      <start>
        <x>0</x>
        <y>0</y>
        <interval>100</interval>
      </start>
      <sequence repeat="true">
        <frame>0</frame>
      </sequence>
    </animation>
  </animations>
</animations>"#
        )
    }
}
