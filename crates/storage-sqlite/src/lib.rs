mod answer_history;
mod articles;
mod backup;
mod datasets;
mod distractor_index;
mod home;
mod lexicon_adapter;
mod lexicon_search;
mod mistakes;
mod read_model;
mod schema;
mod seed_upgrade;
mod settings;
mod study;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use polarbear_vocab_application::ApplicationError;
use rusqlite::{Connection, MAIN_DB, OpenFlags, OptionalExtension};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatabasePaths {
    pub content: PathBuf,
    pub user: PathBuf,
}

pub struct SqliteStore {
    content: Mutex<Connection>,
    user: Mutex<Connection>,
    paths: DatabasePaths,
}

impl SqliteStore {
    pub fn open(paths: &DatabasePaths) -> Result<Self, ApplicationError> {
        let content = open_content(&paths.content)?;
        let user = open_user(&paths.user)?;
        Ok(Self {
            content: Mutex::new(content),
            user: Mutex::new(user),
            paths: paths.clone(),
        })
    }

    fn content(&self) -> Result<MutexGuard<'_, Connection>, ApplicationError> {
        self.content
            .lock()
            .map_err(|_| ApplicationError::Infrastructure("content database lock poisoned".into()))
    }

    fn user(&self) -> Result<MutexGuard<'_, Connection>, ApplicationError> {
        self.user
            .lock()
            .map_err(|_| ApplicationError::Infrastructure("user database lock poisoned".into()))
    }
}

pub fn ensure_writable_content(
    seed: &Path,
    destination: &Path,
) -> Result<PathBuf, ApplicationError> {
    if destination.is_file() {
        let version = Connection::open(destination)
            .and_then(|connection| {
                connection.query_row(
                    "SELECT value FROM schema_meta WHERE key = 'version'",
                    [],
                    |row| row.get::<_, String>(0),
                )
            })
            .map_err(database_error)?;
        if version == "2" || version == "3" {
            seed_upgrade::sync_preloaded_seed(seed, destination)?;
            return Ok(destination.to_path_buf());
        }
        return Err(ApplicationError::Infrastructure(format!(
            "unsupported content database schema {version}; existing data was not overwritten"
        )));
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            ApplicationError::Infrastructure(format!("cannot create app data directory: {error}"))
        })?;
    }
    std::fs::copy(seed, destination).map_err(|error| {
        ApplicationError::Infrastructure(format!("cannot install content database: {error}"))
    })?;
    Ok(destination.to_path_buf())
}

fn open_content(path: &Path) -> Result<Connection, ApplicationError> {
    if !path.is_file() {
        return Err(ApplicationError::NotFound(format!(
            "content database does not exist: {}",
            path.display()
        )));
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut connection = Connection::open_with_flags(path, flags).map_err(database_error)?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(database_error)?;
    migrate_content_schema(&mut connection)?;
    Ok(connection)
}

pub(crate) fn migrate_content_schema(connection: &mut Connection) -> Result<(), ApplicationError> {
    let dataset_exists = connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'dataset'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(database_error)?
        .is_some();
    if !dataset_exists {
        return Ok(());
    }
    let meta_exists = connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_meta'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(database_error)?
        .is_some();
    if meta_exists {
        let version: Option<String> = connection
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'version'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(database_error)?;
        if let Some(version) = version
            && version != "2"
            && version != "3"
        {
            return Err(ApplicationError::Infrastructure(format!(
                "unsupported content database schema {version}"
            )));
        }
    }
    let mut statement = connection
        .prepare("PRAGMA table_info(dataset)")
        .map_err(database_error)?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(database_error)?;
    let has_updated_at = columns
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?
        .iter()
        .any(|name| name == "updated_at");
    drop(statement);
    schema::in_immediate_transaction(connection, |transaction| {
        if !has_updated_at {
            transaction.execute(
                "ALTER TABLE dataset ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
            transaction.execute("UPDATE dataset SET updated_at = created_at", [])?;
        }
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta(key, value) VALUES ('version', '3')
             ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
        )
    })
    .map_err(database_error)
}

fn open_user(path: &Path) -> Result<Connection, ApplicationError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            ApplicationError::Infrastructure(format!("cannot create app data directory: {error}"))
        })?;
    }
    let backup = path.with_extension("db.migration-backup");
    let existing = path.is_file();
    let mut connection = Connection::open(path).map_err(database_error)?;
    if existing {
        connection
            .backup(MAIN_DB, &backup, None)
            .map_err(database_error)?;
    }
    if let Err(error) = prepare_user_connection(&mut connection) {
        if existing {
            connection
                .restore(MAIN_DB, &backup, None::<fn(_)>)
                .map_err(|restore_error| {
                    ApplicationError::Infrastructure(format!(
                        "user database setup failed ({error}); restore failed ({restore_error})"
                    ))
                })?;
        }
        drop(connection);
        return Err(error);
    }
    if existing {
        std::fs::remove_file(backup).map_err(|error| {
            ApplicationError::Infrastructure(format!("cannot remove migration backup: {error}"))
        })?;
    }
    Ok(connection)
}

fn prepare_user_connection(connection: &mut Connection) -> Result<(), ApplicationError> {
    let meta_exists = connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_meta'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(database_error)?
        .is_some();
    if meta_exists {
        let version: Option<String> = connection
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'version'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(database_error)?;
        if let Some(version) = version {
            let number = version.parse::<u32>().unwrap_or_default();
            if number == 0 || number > 5 {
                return Err(ApplicationError::Infrastructure(format!(
                    "unsupported user database schema {version}"
                )));
            }
        }
    }
    connection
        .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
        .map_err(database_error)?;
    schema::initialize_user_schema(connection).map_err(database_error)?;
    let violations: String = connection
        .query_row(
            "SELECT COALESCE(group_concat(\"table\" || ':' || rowid), '')
             FROM pragma_foreign_key_check",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if violations.is_empty() {
        Ok(())
    } else {
        Err(ApplicationError::Infrastructure(format!(
            "user database foreign-key violations: {violations}"
        )))
    }
}

fn database_error(error: rusqlite::Error) -> ApplicationError {
    ApplicationError::Infrastructure(error.to_string())
}
