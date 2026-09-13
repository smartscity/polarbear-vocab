mod datasets;
mod distractor_index;
mod home;
mod mistakes;
mod read_model;
mod schema;
mod settings;
mod study;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use polarbear_vocab_application::ApplicationError;
use rusqlite::{Connection, OpenFlags};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatabasePaths {
    pub content: PathBuf,
    pub user: PathBuf,
}

pub struct SqliteStore {
    content: Mutex<Connection>,
    user: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(paths: &DatabasePaths) -> Result<Self, ApplicationError> {
        let content = open_content(&paths.content)?;
        let user = open_user(&paths.user)?;
        Ok(Self {
            content: Mutex::new(content),
            user: Mutex::new(user),
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
    let current = destination.is_file()
        && Connection::open(destination)
            .and_then(|connection| {
                connection.query_row(
                    "SELECT value FROM schema_meta WHERE key = 'version'",
                    [],
                    |row| row.get::<_, String>(0),
                )
            })
            .is_ok_and(|version| version == "2");
    if current {
        return Ok(destination.to_path_buf());
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
    let connection = Connection::open_with_flags(path, flags).map_err(database_error)?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(database_error)?;
    Ok(connection)
}

fn open_user(path: &Path) -> Result<Connection, ApplicationError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            ApplicationError::Infrastructure(format!("cannot create app data directory: {error}"))
        })?;
    }
    let backup = path.with_extension("db.migration-backup");
    let existing = path.is_file();
    if existing {
        std::fs::copy(path, &backup).map_err(|error| {
            ApplicationError::Infrastructure(format!("cannot back up user database: {error}"))
        })?;
    }
    let mut connection = Connection::open(path).map_err(database_error)?;
    if let Err(error) = prepare_user_connection(&mut connection) {
        drop(connection);
        if existing {
            std::fs::copy(&backup, path).map_err(|restore_error| {
                ApplicationError::Infrastructure(format!(
                    "user database setup failed ({error}); restore failed ({restore_error})"
                ))
            })?;
        }
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
