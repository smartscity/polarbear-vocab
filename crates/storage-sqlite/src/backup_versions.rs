use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};
use polarbear_vocab_application::ApplicationError;
use polarbear_vocab_domain::{BackupManifest, BackupVersionDto};
use uuid::Uuid;
use zip::ZipArchive;

const BACKUP_EXTENSION: &str = ".polarbear-vocab-backup";
const BACKUP_FORMAT: &str = "polarbear-vocab-backup";
const BACKUP_SCHEMA_VERSION: u32 = 1;
const MANAGED_BACKUP_LIMIT: usize = 10;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

pub fn new_path(user_path: &Path, reason: &str) -> Result<PathBuf, ApplicationError> {
    let directory = directory(user_path);
    std::fs::create_dir_all(&directory).map_err(io_error)?;
    Ok(directory.join(format!(
        "{reason}-{}-{}{BACKUP_EXTENSION}",
        Utc::now().format("%Y%m%d-%H%M%S"),
        Uuid::new_v4()
    )))
}

pub fn resolve(user_path: &Path, id: &str) -> Result<PathBuf, ApplicationError> {
    if id.is_empty()
        || id.len() > 160
        || id.contains(['/', '\\'])
        || !id.ends_with(BACKUP_EXTENSION)
    {
        return Err(ApplicationError::InvalidInput(
            "backup version id is invalid".to_owned(),
        ));
    }
    let path = directory(user_path).join(id);
    if !path.is_file() {
        return Err(ApplicationError::NotFound(id.to_owned()));
    }
    Ok(path)
}

pub fn list(user_path: &Path) -> Result<Vec<BackupVersionDto>, ApplicationError> {
    let directory = directory(user_path);
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut versions = Vec::new();
    for entry in std::fs::read_dir(directory).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        if let Some(version) = read_version(&entry.path())? {
            versions.push(version);
        }
    }
    versions.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(versions)
}

pub fn automatic_due(versions: &[BackupVersionDto]) -> bool {
    let Some(latest) = versions
        .iter()
        .find(|version| version.reason == "automatic")
    else {
        return true;
    };
    let Ok(created_at) = chrono::DateTime::parse_from_rfc3339(&latest.created_at) else {
        return true;
    };
    Utc::now().signed_duration_since(created_at) >= Duration::hours(24)
}

pub fn prune(user_path: &Path) -> Result<(), ApplicationError> {
    for version in list(user_path)?.into_iter().skip(MANAGED_BACKUP_LIMIT) {
        let path = resolve(user_path, &version.id)?;
        std::fs::remove_file(path).map_err(io_error)?;
    }
    Ok(())
}

fn directory(user_path: &Path) -> PathBuf {
    user_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups")
}

fn read_version(path: &Path) -> Result<Option<BackupVersionDto>, ApplicationError> {
    let Some(id) = path.file_name().and_then(|name| name.to_str()) else {
        return Ok(None);
    };
    let Some(reason) = reason(id) else {
        return Ok(None);
    };
    let manifest = read_manifest(path)?;
    let size_bytes = path.metadata().map_err(io_error)?.len();
    Ok(Some(BackupVersionDto {
        id: id.to_owned(),
        created_at: manifest.created_at,
        size_bytes,
        reason: reason.to_owned(),
    }))
}

fn reason(id: &str) -> Option<&'static str> {
    if id.starts_with("automatic-") && id.ends_with(BACKUP_EXTENSION) {
        Some("automatic")
    } else if id.starts_with("pre-restore-") && id.ends_with(BACKUP_EXTENSION) {
        Some("preRestore")
    } else if id.starts_with("pre-sync-") && id.ends_with(BACKUP_EXTENSION) {
        Some("preSync")
    } else {
        None
    }
}

fn read_manifest(path: &Path) -> Result<BackupManifest, ApplicationError> {
    let file = File::open(path).map_err(io_error)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let entry = archive.by_name("manifest.json").map_err(zip_error)?;
    if entry.size() > MAX_MANIFEST_BYTES || entry.is_dir() {
        return Err(invalid_backup("manifest is invalid or too large"));
    }
    let mut bytes = Vec::new();
    entry
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    let manifest: BackupManifest = serde_json::from_slice(&bytes).map_err(json_error)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &BackupManifest) -> Result<(), ApplicationError> {
    if manifest.format != BACKUP_FORMAT
        || manifest.schema_version != BACKUP_SCHEMA_VERSION
        || manifest.app_version.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&manifest.created_at).is_err()
    {
        return Err(invalid_backup("managed backup manifest is invalid"));
    }
    Ok(())
}

fn invalid_backup(message: &str) -> ApplicationError {
    ApplicationError::InvalidInput(format!("invalid backup: {message}"))
}

fn io_error(error: std::io::Error) -> ApplicationError {
    ApplicationError::Infrastructure(error.to_string())
}

fn zip_error(error: zip::result::ZipError) -> ApplicationError {
    ApplicationError::InvalidInput(format!("invalid backup archive: {error}"))
}

fn json_error(error: serde_json::Error) -> ApplicationError {
    ApplicationError::InvalidInput(format!("invalid backup manifest: {error}"))
}
