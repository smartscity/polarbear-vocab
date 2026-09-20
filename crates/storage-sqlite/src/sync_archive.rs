use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use polarbear_vocab_application::ApplicationError;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::sync_models::{ContentSyncPayload, SyncManifest, UserSyncPayload};

const MAX_JSON_BYTES: u64 = 256 * 1024 * 1024;

pub struct SyncPackage {
    pub manifest: SyncManifest,
    pub content: ContentSyncPayload,
    pub user: UserSyncPayload,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn write(
    path: &Path,
    manifest: &SyncManifest,
    content: &[u8],
    user: &[u8],
) -> Result<(), ApplicationError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let temporary = tempfile::NamedTempFile::new_in(parent).map_err(io_error)?;
    let mut archive = ZipWriter::new(temporary.reopen().map_err(io_error)?);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    write_entry(
        &mut archive,
        "manifest.json",
        &serde_json::to_vec_pretty(manifest).map_err(json_error)?,
        options,
    )?;
    write_entry(&mut archive, "content.json", content, options)?;
    write_entry(&mut archive, "user.json", user, options)?;
    archive
        .finish()
        .map_err(zip_error)?
        .sync_all()
        .map_err(io_error)?;
    temporary
        .persist(path)
        .map_err(|error| io_error(error.error))?;
    Ok(())
}

pub fn read(path: &Path) -> Result<SyncPackage, ApplicationError> {
    let mut archive = ZipArchive::new(File::open(path).map_err(io_error)?).map_err(zip_error)?;
    validate_entries(&mut archive)?;
    let manifest: SyncManifest = deserialize(read_entry(&mut archive, "manifest.json")?)?;
    validate_manifest(&manifest)?;
    let content_bytes = read_entry(&mut archive, "content.json")?;
    let user_bytes = read_entry(&mut archive, "user.json")?;
    if digest(&content_bytes) != manifest.content_sha256
        || digest(&user_bytes) != manifest.user_sha256
    {
        return Err(invalid("payload digest mismatch"));
    }
    Ok(SyncPackage {
        manifest,
        content: deserialize(content_bytes)?,
        user: deserialize(user_bytes)?,
    })
}

fn write_entry(
    archive: &mut ZipWriter<File>,
    name: &str,
    bytes: &[u8],
    options: SimpleFileOptions,
) -> Result<(), ApplicationError> {
    archive.start_file(name, options).map_err(zip_error)?;
    archive.write_all(bytes).map_err(io_error)
}

fn validate_entries(archive: &mut ZipArchive<File>) -> Result<(), ApplicationError> {
    let expected = ["manifest.json", "content.json", "user.json"];
    let mut names = Vec::new();
    for index in 0..archive.len() {
        names.push(
            archive
                .by_index(index)
                .map_err(zip_error)?
                .name()
                .to_owned(),
        );
    }
    if names.len() != expected.len()
        || expected
            .iter()
            .any(|name| names.iter().filter(|candidate| candidate == name).count() != 1)
    {
        return Err(invalid("archive entries are invalid"));
    }
    Ok(())
}

fn read_entry(archive: &mut ZipArchive<File>, name: &str) -> Result<Vec<u8>, ApplicationError> {
    let file = archive.by_name(name).map_err(zip_error)?;
    if file.is_dir() || file.size() > MAX_JSON_BYTES {
        return Err(invalid("sync entry is invalid or too large"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_JSON_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > MAX_JSON_BYTES {
        return Err(invalid("sync entry is too large"));
    }
    Ok(bytes)
}

fn validate_manifest(manifest: &SyncManifest) -> Result<(), ApplicationError> {
    if manifest.format != "polarbear-vocab-sync"
        || manifest.schema_version != 1
        || manifest.app_version.trim().is_empty()
        || uuid::Uuid::parse_str(&manifest.package_id).is_err()
        || uuid::Uuid::parse_str(&manifest.source_device_id).is_err()
        || chrono::DateTime::parse_from_rfc3339(&manifest.created_at).is_err()
        || manifest.content_cursor < 0
        || manifest.user_cursor < 0
        || manifest.acknowledgements.iter().any(|acknowledgement| {
            uuid::Uuid::parse_str(&acknowledgement.device_id).is_err()
                || acknowledgement.content_cursor < 0
                || acknowledgement.user_cursor < 0
        })
    {
        return Err(invalid("manifest is invalid or unsupported"));
    }
    Ok(())
}

fn deserialize<T: serde::de::DeserializeOwned>(bytes: Vec<u8>) -> Result<T, ApplicationError> {
    serde_json::from_slice(&bytes).map_err(json_error)
}

fn invalid(message: &str) -> ApplicationError {
    ApplicationError::InvalidInput(format!("invalid sync package: {message}"))
}

fn io_error(error: std::io::Error) -> ApplicationError {
    ApplicationError::Infrastructure(error.to_string())
}

fn zip_error(error: zip::result::ZipError) -> ApplicationError {
    ApplicationError::InvalidInput(format!("invalid sync archive: {error}"))
}

fn json_error(error: serde_json::Error) -> ApplicationError {
    ApplicationError::InvalidInput(format!("invalid sync JSON: {error}"))
}
