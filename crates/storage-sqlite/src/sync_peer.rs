use chrono::Utc;
use polarbear_vocab_application::ApplicationError;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::sync_models::{SyncAcknowledgement, SyncCursor, SyncManifest};
use crate::{database_error, schema};

const DEVICE_ID_KEY: &str = "sync_device_id";
const LAST_SYNC_KEY: &str = "last_sync_at";

pub fn device_id(connection: &mut Connection) -> Result<String, ApplicationError> {
    if let Some(id) = setting_string(connection, DEVICE_ID_KEY)? {
        if Uuid::parse_str(&id).is_ok() {
            return Ok(id);
        }
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    connection
        .execute(
            "INSERT INTO setting(key, value_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
               updated_at = excluded.updated_at",
            params![DEVICE_ID_KEY, serde_json::to_string(&id).unwrap(), now],
        )
        .map_err(database_error)?;
    Ok(id)
}

pub fn base_cursors(connection: &Connection) -> Result<SyncCursor, ApplicationError> {
    connection
        .query_row(
            "SELECT COALESCE(MIN(acknowledged_content_cursor), 0),
                    COALESCE(MIN(acknowledged_user_cursor), 0)
             FROM sync_peer",
            [],
            |row| {
                Ok(SyncCursor {
                    content: row.get(0)?,
                    user: row.get(1)?,
                })
            },
        )
        .map_err(database_error)
}

pub fn acknowledgements(
    connection: &Connection,
) -> Result<Vec<SyncAcknowledgement>, ApplicationError> {
    let mut statement = connection
        .prepare(
            "SELECT device_id, received_content_cursor, received_user_cursor
             FROM sync_peer ORDER BY device_id",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(SyncAcknowledgement {
                device_id: row.get(0)?,
                content_cursor: row.get(1)?,
                user_cursor: row.get(2)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<rusqlite::Result<_>>()
        .map_err(database_error)
}

pub fn local_acknowledged_by_source(manifest: &SyncManifest, local_device_id: &str) -> SyncCursor {
    manifest
        .acknowledgements
        .iter()
        .find(|ack| ack.device_id == local_device_id)
        .map(|ack| SyncCursor {
            content: ack.content_cursor,
            user: ack.user_cursor,
        })
        .unwrap_or_default()
}

pub fn package_applied(
    connection: &Connection,
    package_id: &str,
) -> Result<bool, ApplicationError> {
    connection
        .query_row(
            "SELECT 1 FROM sync_package WHERE package_id = ?1",
            [package_id],
            |_| Ok(()),
        )
        .optional()
        .map(|value| value.is_some())
        .map_err(database_error)
}

pub fn finish_import(
    transaction: &Transaction<'_>,
    manifest: &SyncManifest,
    acknowledged: SyncCursor,
    imported_at: i64,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO sync_peer(
           device_id, received_content_cursor, received_user_cursor,
           acknowledged_content_cursor, acknowledged_user_cursor, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(device_id) DO UPDATE SET
           received_content_cursor = MAX(received_content_cursor, excluded.received_content_cursor),
           received_user_cursor = MAX(received_user_cursor, excluded.received_user_cursor),
           acknowledged_content_cursor = MAX(acknowledged_content_cursor, excluded.acknowledged_content_cursor),
           acknowledged_user_cursor = MAX(acknowledged_user_cursor, excluded.acknowledged_user_cursor),
           updated_at = excluded.updated_at",
        params![
            manifest.source_device_id,
            manifest.content_cursor,
            manifest.user_cursor,
            acknowledged.content,
            acknowledged.user,
            imported_at
        ],
    )?;
    transaction.execute(
        "INSERT INTO sync_package(package_id, source_device_id, imported_at)
         VALUES (?1, ?2, ?3)",
        params![manifest.package_id, manifest.source_device_id, imported_at],
    )?;
    transaction.execute(
        "INSERT INTO setting(key, value_json, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
           updated_at = excluded.updated_at",
        params![
            LAST_SYNC_KEY,
            serde_json::to_string(&Utc::now().to_rfc3339()).unwrap(),
            imported_at
        ],
    )?;
    Ok(())
}

pub fn peer_count(connection: &Connection) -> Result<u32, ApplicationError> {
    connection
        .query_row("SELECT COUNT(*) FROM sync_peer", [], |row| row.get(0))
        .map_err(database_error)
}

pub fn last_sync_at(connection: &Connection) -> Result<Option<String>, ApplicationError> {
    setting_string(connection, LAST_SYNC_KEY)
}

fn setting_string(connection: &Connection, key: &str) -> Result<Option<String>, ApplicationError> {
    let value = connection
        .query_row(
            "SELECT value_json FROM setting WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(database_error)?;
    Ok(value.and_then(|json| serde_json::from_str(&json).ok()))
}

pub fn record_local_change(
    transaction: &Transaction<'_>,
    kind: &str,
    id: &str,
    changed_at: i64,
) -> rusqlite::Result<()> {
    schema::record_sync_change(transaction, kind, id, "upsert", changed_at)?;
    Ok(())
}
