use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use chrono::{SecondsFormat, Utc};
use polarbear_vocab_application::{ApplicationError, BackupRepository};
use polarbear_vocab_domain::{BackupManifest, BackupStatusDto, BackupVersionDto, RestoreResultDto};
use rusqlite::{Connection, MAIN_DB, OptionalExtension, params};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::{SqliteStore, backup_versions, database_error, migrate_content_schema, schema};

const BACKUP_FORMAT: &str = "polarbear-vocab-backup";
const BACKUP_SCHEMA_VERSION: u32 = 1;
const MAX_DATABASE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

impl BackupRepository for SqliteStore {
    fn backup_status(&self) -> Result<BackupStatusDto, ApplicationError> {
        let user = self.user()?;
        let value = user
            .query_row(
                "SELECT value_json FROM setting WHERE key = 'last_backup_at'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(database_error)?;
        Ok(BackupStatusDto {
            last_backup_at: value.and_then(|json| serde_json::from_str(&json).ok()),
        })
    }

    fn ensure_automatic_backup(
        &self,
        app_version: &str,
    ) -> Result<Vec<BackupVersionDto>, ApplicationError> {
        let versions = backup_versions::list(&self.paths.user)?;
        if backup_versions::automatic_due(&versions) {
            let path = backup_versions::new_path(&self.paths.user, "automatic")?;
            self.export_backup(&path.to_string_lossy(), app_version)?;
            backup_versions::prune(&self.paths.user)?;
            return backup_versions::list(&self.paths.user);
        }
        Ok(versions)
    }

    fn list_backup_versions(&self) -> Result<Vec<BackupVersionDto>, ApplicationError> {
        backup_versions::list(&self.paths.user)
    }

    fn export_backup(
        &self,
        path: &str,
        app_version: &str,
    ) -> Result<BackupStatusDto, ApplicationError> {
        let created_at = now();
        let temporary = tempfile::tempdir().map_err(io_error)?;
        let content_snapshot = temporary.path().join("content.db");
        let user_snapshot = temporary.path().join("user.db");
        self.content()?
            .backup(MAIN_DB, &content_snapshot, None)
            .map_err(database_error)?;
        self.user()?
            .backup(MAIN_DB, &user_snapshot, None)
            .map_err(database_error)?;
        let manifest = BackupManifest {
            format: BACKUP_FORMAT.to_owned(),
            schema_version: BACKUP_SCHEMA_VERSION,
            app_version: app_version.to_owned(),
            created_at: created_at.clone(),
        };
        write_archive(
            Path::new(path),
            &manifest,
            &content_snapshot,
            &user_snapshot,
        )?;
        self.record_last_backup(&created_at)?;
        Ok(BackupStatusDto {
            last_backup_at: Some(created_at),
        })
    }

    fn import_backup(
        &self,
        path: &str,
        app_version: &str,
    ) -> Result<RestoreResultDto, ApplicationError> {
        let extracted = tempfile::tempdir().map_err(io_error)?;
        extract_archive(Path::new(path), &extracted)?;
        validate_databases(extracted.path())?;
        let automatic_path = backup_versions::new_path(&self.paths.user, "pre-restore")?;
        let safety_status = self.export_backup(&automatic_path.to_string_lossy(), app_version)?;
        backup_versions::prune(&self.paths.user)?;
        let safety = tempfile::tempdir().map_err(io_error)?;
        extract_archive(&automatic_path, &safety)?;
        if let Err(error) = self.restore_databases(extracted.path()) {
            self.restore_databases(safety.path()).map_err(|rollback_error| {
                ApplicationError::Infrastructure(format!(
                    "restore failed ({error}); automatic rollback failed ({rollback_error}); safety backup is {}",
                    automatic_path.display()
                ))
            })?;
            return Err(error);
        }
        if let Some(created_at) = safety_status.last_backup_at {
            let _ = self.record_last_backup(&created_at);
        }
        Ok(RestoreResultDto {
            automatic_backup_path: automatic_path.to_string_lossy().into_owned(),
        })
    }

    fn restore_backup_version(
        &self,
        id: &str,
        app_version: &str,
    ) -> Result<RestoreResultDto, ApplicationError> {
        let path = backup_versions::resolve(&self.paths.user, id)?;
        self.import_backup(&path.to_string_lossy(), app_version)
    }
}

impl SqliteStore {
    fn record_last_backup(&self, created_at: &str) -> Result<(), ApplicationError> {
        self.user()?
            .execute(
                "INSERT INTO setting(key, value_json, updated_at) VALUES ('last_backup_at', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
                    updated_at = excluded.updated_at",
                params![
                    serde_json::to_string(created_at)
                        .map_err(|error| { ApplicationError::Infrastructure(error.to_string()) })?,
                    Utc::now().timestamp_millis()
                ],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn restore_databases(&self, directory: &Path) -> Result<(), ApplicationError> {
        let mut content = self.content()?;
        let mut user = self.user()?;
        content
            .restore(MAIN_DB, directory.join("content.db"), None::<fn(_)>)
            .map_err(database_error)?;
        migrate_content_schema(&mut content)?;
        user.restore(MAIN_DB, directory.join("user.db"), None::<fn(_)>)
            .map_err(database_error)?;
        schema::initialize_user_schema(&mut user).map_err(database_error)?;
        check_integrity(&content)?;
        check_integrity(&user)
    }
}

fn write_archive(
    destination: &Path,
    manifest: &BackupManifest,
    content: &Path,
    user: &Path,
) -> Result<(), ApplicationError> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let temporary =
        tempfile::NamedTempFile::new_in(destination.parent().unwrap_or_else(|| Path::new(".")))
            .map_err(io_error)?;
    let file = temporary.reopen().map_err(io_error)?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    archive
        .start_file("manifest.json", options)
        .map_err(zip_error)?;
    archive
        .write_all(&serde_json::to_vec_pretty(manifest).map_err(json_error)?)
        .map_err(io_error)?;
    write_file(&mut archive, "content.db", content, options)?;
    write_file(&mut archive, "user.db", user, options)?;
    archive
        .finish()
        .map_err(zip_error)?
        .sync_all()
        .map_err(io_error)?;
    temporary
        .persist(destination)
        .map_err(|error| io_error(error.error))?;
    Ok(())
}

fn write_file(
    archive: &mut ZipWriter<File>,
    name: &str,
    path: &Path,
    options: SimpleFileOptions,
) -> Result<(), ApplicationError> {
    archive.start_file(name, options).map_err(zip_error)?;
    std::io::copy(&mut File::open(path).map_err(io_error)?, archive).map_err(io_error)?;
    Ok(())
}

fn extract_archive(source: &Path, directory: &TempDir) -> Result<BackupManifest, ApplicationError> {
    let mut archive = ZipArchive::new(File::open(source).map_err(io_error)?).map_err(zip_error)?;
    let expected = ["manifest.json", "content.db", "user.db"];
    let names = (0..archive.len())
        .map(|index| archive.by_index(index).map(|file| file.name().to_owned()))
        .collect::<zip::result::ZipResult<Vec<_>>>()
        .map_err(zip_error)?;
    if names.len() != expected.len()
        || expected
            .iter()
            .any(|name| names.iter().filter(|candidate| *candidate == name).count() != 1)
    {
        return Err(invalid_backup(
            "archive must contain only manifest.json, content.db, and user.db",
        ));
    }
    let manifest_bytes = read_entry(&mut archive, "manifest.json", MAX_MANIFEST_BYTES)?;
    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes).map_err(json_error)?;
    if manifest.format != BACKUP_FORMAT || manifest.schema_version != BACKUP_SCHEMA_VERSION {
        return Err(invalid_backup(
            "unsupported backup format or schema version",
        ));
    }
    if manifest.app_version.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&manifest.created_at).is_err()
    {
        return Err(invalid_backup(
            "backup manifest version or creation time is invalid",
        ));
    }
    for name in ["content.db", "user.db"] {
        extract_database(&mut archive, name, directory.path())?;
    }
    Ok(manifest)
}

fn read_entry(
    archive: &mut ZipArchive<File>,
    name: &str,
    maximum: u64,
) -> Result<Vec<u8>, ApplicationError> {
    let file = archive.by_name(name).map_err(zip_error)?;
    if file.size() > maximum || file.is_dir() {
        return Err(invalid_backup("backup entry is invalid or too large"));
    }
    let mut bytes = Vec::new();
    file.take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > maximum {
        return Err(invalid_backup("backup entry is too large"));
    }
    Ok(bytes)
}

fn extract_database(
    archive: &mut ZipArchive<File>,
    name: &str,
    directory: &Path,
) -> Result<(), ApplicationError> {
    let file = archive.by_name(name).map_err(zip_error)?;
    if file.size() > MAX_DATABASE_BYTES || file.is_dir() {
        return Err(invalid_backup("database entry is invalid or too large"));
    }
    let mut destination = File::create(directory.join(name)).map_err(io_error)?;
    let copied = std::io::copy(&mut file.take(MAX_DATABASE_BYTES + 1), &mut destination)
        .map_err(io_error)?;
    if copied > MAX_DATABASE_BYTES {
        return Err(invalid_backup("database entry is too large"));
    }
    destination.sync_all().map_err(io_error)
}

fn validate_databases(directory: &Path) -> Result<(), ApplicationError> {
    let content = Connection::open(directory.join("content.db")).map_err(database_error)?;
    check_integrity(&content)?;
    let content_version = schema_version(&content)?;
    if content_version != "2" && content_version != "3" {
        return Err(invalid_backup("unsupported content database schema"));
    }
    check_required_tables(&content, &["dataset", "dataset_item", "word", "sense"])?;
    let user = Connection::open(directory.join("user.db")).map_err(database_error)?;
    check_integrity(&user)?;
    let user_version = schema_version(&user)?.parse::<u32>().unwrap_or_default();
    if user_version == 0 || user_version > 6 {
        return Err(invalid_backup("unsupported user database schema"));
    }
    check_required_tables(&user, &["study_session", "review_event", "setting"])?;
    Ok(())
}

fn check_required_tables(connection: &Connection, names: &[&str]) -> Result<(), ApplicationError> {
    for name in names {
        let exists = connection
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [name],
                |_| Ok(()),
            )
            .optional()
            .map_err(database_error)?
            .is_some();
        if !exists {
            return Err(invalid_backup("required database table is missing"));
        }
    }
    Ok(())
}

fn check_integrity(connection: &Connection) -> Result<(), ApplicationError> {
    let result: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(database_error)?;
    if result != "ok" {
        return Err(invalid_backup("database integrity check failed"));
    }
    let violations: i64 = connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .map_err(database_error)?;
    if violations > 0 {
        return Err(invalid_backup("database foreign-key check failed"));
    }
    Ok(())
}

fn schema_version(connection: &Connection) -> Result<String, ApplicationError> {
    connection
        .query_row(
            "SELECT value FROM schema_meta WHERE key = 'version'",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
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
