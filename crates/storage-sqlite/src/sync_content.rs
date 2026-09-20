use polarbear_vocab_application::ApplicationError;
use polarbear_vocab_domain::ImportedSense;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::sync_models::{ApplyCounts, ContentSyncPayload, SyncDataset};
use crate::{database_error, datasets, distractor_index, schema};

pub fn export(
    connection: &Connection,
    base_cursor: i64,
) -> Result<(ContentSyncPayload, i64), ApplicationError> {
    let cursor = maximum_cursor(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT change.entity_id, change.operation, change.seq
             FROM sync_change change
             JOIN (
               SELECT entity_id, MAX(seq) AS seq FROM sync_change
               WHERE entity_kind = 'dataset' AND seq > ?1 GROUP BY entity_id
             ) latest ON latest.seq = change.seq
             ORDER BY change.seq",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([base_cursor], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(database_error)?;
    let changes = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;
    let mut payload = ContentSyncPayload::default();
    for (id, operation, revision) in changes {
        payload.datasets.push(if operation == "delete" {
            deleted_dataset(id, revision)
        } else {
            dataset_snapshot(connection, &id, revision)?.ok_or_else(|| {
                ApplicationError::Infrastructure(format!("changed dataset {id} is missing"))
            })?
        });
    }
    Ok((payload, cursor))
}

pub fn apply(
    connection: &mut Connection,
    payload: &ContentSyncPayload,
    acknowledged_local_cursor: i64,
    source_device_id: &str,
    package_id: &str,
) -> Result<ApplyCounts, ApplicationError> {
    schema::in_immediate_transaction(connection, |transaction| {
        let mut counts = ApplyCounts::default();
        for dataset in &payload.datasets {
            let conflict = is_conflict(transaction, dataset, acknowledged_local_cursor)?;
            if conflict {
                apply_conflict(transaction, dataset, source_device_id, package_id)?;
                counts.conflicts += 1;
            } else {
                apply_dataset(transaction, dataset)?;
            }
            counts.datasets += 1;
        }
        Ok(counts)
    })
    .map_err(database_error)
}

pub fn pending_count(connection: &Connection, base_cursor: i64) -> rusqlite::Result<u32> {
    connection.query_row(
        "SELECT COUNT(DISTINCT entity_kind || ':' || entity_id)
         FROM sync_change WHERE seq > ?1",
        [base_cursor],
        |row| row.get(0),
    )
}

fn maximum_cursor(connection: &Connection) -> Result<i64, ApplicationError> {
    connection
        .query_row("SELECT COALESCE(MAX(seq), 0) FROM sync_change", [], |row| {
            row.get(0)
        })
        .map_err(database_error)
}

fn dataset_snapshot(
    connection: &Connection,
    id: &str,
    revision: i64,
) -> Result<Option<SyncDataset>, ApplicationError> {
    let header = connection
        .query_row(
            "SELECT name, created_at, updated_at FROM dataset
             WHERE id = ?1 AND preloaded = 0",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()
        .map_err(database_error)?;
    let Some((name, created_at, updated_at)) = header else {
        return Ok(None);
    };
    Ok(Some(SyncDataset {
        id: id.to_owned(),
        name,
        created_at,
        updated_at,
        revision,
        deleted: false,
        entries: dataset_entries(connection, id)?,
    }))
}

fn dataset_entries(
    connection: &Connection,
    dataset_id: &str,
) -> Result<Vec<ImportedSense>, ApplicationError> {
    let mut statement = connection
        .prepare(
            "SELECT s.uid, w.lemma, s.pos, s.quiz_prompt_zh, s.zh_gloss,
               COALESCE(us.ipa, ''), COALESCE(uk.ipa, ''),
               COALESCE(example.sentence_en, ''), COALESCE(example.sentence_zh, '')
             FROM dataset_item item
             JOIN sense s ON s.uid = item.sense_uid
             JOIN word w ON w.id = s.word_id
             LEFT JOIN pronunciation us ON us.sense_id = s.id AND us.accent = 'en-US'
             LEFT JOIN pronunciation uk ON uk.sense_id = s.id AND uk.accent = 'en-GB'
             LEFT JOIN example ON example.sense_id = s.id AND example.is_primary = 1
             WHERE item.dataset_id = ?1 ORDER BY item.sequence, s.uid",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([dataset_id], |row| {
            Ok(ImportedSense {
                sense_uid: row.get(0)?,
                lemma: row.get(1)?,
                part_of_speech: row.get(2)?,
                quiz_prompt_zh: row.get(3)?,
                gloss_zh: row.get(4)?,
                ipa_us: row.get(5)?,
                ipa_uk: row.get(6)?,
                example_en: row.get(7)?,
                example_zh: row.get(8)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<rusqlite::Result<_>>()
        .map_err(database_error)
}

fn is_conflict(
    transaction: &Transaction<'_>,
    incoming: &SyncDataset,
    acknowledged_local_cursor: i64,
) -> rusqlite::Result<bool> {
    let changed: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM sync_change
         WHERE entity_kind = 'dataset' AND entity_id = ?1 AND seq > ?2)",
        params![incoming.id, acknowledged_local_cursor],
        |row| row.get(0),
    )?;
    if !changed {
        return Ok(false);
    }
    if incoming.deleted {
        return Ok(true);
    }
    let current =
        dataset_snapshot(transaction, &incoming.id, incoming.revision).map_err(to_sql_error)?;
    Ok(!current.is_some_and(|current| same_dataset(&current, incoming)))
}

fn same_dataset(left: &SyncDataset, right: &SyncDataset) -> bool {
    left.name == right.name && left.entries == right.entries
}

fn apply_conflict(
    transaction: &Transaction<'_>,
    incoming: &SyncDataset,
    source_device_id: &str,
    package_id: &str,
) -> rusqlite::Result<()> {
    if incoming.deleted {
        return Ok(());
    }
    let suffix = short_id(package_id);
    let mut conflict = incoming.clone();
    conflict.id = Uuid::new_v4().to_string();
    conflict.name = format!(
        "{} (Conflict from {})",
        incoming.name,
        short_id(source_device_id)
    );
    for entry in &mut conflict.entries {
        entry.sense_uid = format!("{}.sync.{suffix}", entry.sense_uid);
    }
    apply_dataset(transaction, &conflict)?;
    schema::record_sync_change(
        transaction,
        "dataset",
        &conflict.id,
        "upsert",
        conflict.updated_at,
    )?;
    Ok(())
}

fn apply_dataset(transaction: &Transaction<'_>, dataset: &SyncDataset) -> rusqlite::Result<()> {
    if dataset.deleted {
        transaction.execute(
            "DELETE FROM dataset WHERE id = ?1 AND preloaded = 0",
            [&dataset.id],
        )?;
        return Ok(());
    }
    transaction.execute(
        "INSERT INTO dataset(id, name, created_at, updated_at, preloaded)
         VALUES (?1, ?2, ?3, ?4, 0)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name,
           updated_at = excluded.updated_at WHERE dataset.preloaded = 0",
        params![
            dataset.id,
            dataset.name,
            dataset.created_at,
            dataset.updated_at
        ],
    )?;
    transaction.execute(
        "DELETE FROM dataset_item WHERE dataset_id = ?1",
        [&dataset.id],
    )?;
    let source_id = datasets::ensure_import_source(transaction)?;
    for (sequence, entry) in dataset.entries.iter().enumerate() {
        datasets::upsert_sense(transaction, entry, source_id)?;
        transaction.execute(
            "INSERT INTO dataset_item(dataset_id, sense_uid, sequence) VALUES (?1, ?2, ?3)",
            params![dataset.id, entry.sense_uid, sequence as u32],
        )?;
    }
    let uids: Vec<String> = dataset
        .entries
        .iter()
        .map(|entry| entry.sense_uid.clone())
        .collect();
    distractor_index::refresh_for(transaction, &uids)
}

fn deleted_dataset(id: String, revision: i64) -> SyncDataset {
    SyncDataset {
        id,
        name: String::new(),
        created_at: 0,
        updated_at: 0,
        revision,
        deleted: true,
        entries: Vec::new(),
    }
}

fn short_id(value: &str) -> &str {
    value.get(..8).unwrap_or(value)
}

fn to_sql_error(error: ApplicationError) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}
