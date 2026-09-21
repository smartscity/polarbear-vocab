use chrono::Utc;
use polarbear_vocab_application::ApplicationError;
use polarbear_vocab_domain::DatasetSummary;
use rusqlite::{Transaction, params};
use uuid::Uuid;

use crate::{SqliteStore, database_error, schema};

impl SqliteStore {
    pub(crate) fn snapshot_vocabulary_as_dataset(
        &self,
        name: &str,
    ) -> Result<DatasetSummary, ApplicationError> {
        let vocabulary_uids = self.my_vocabulary_uids()?;
        if vocabulary_uids.is_empty() {
            return Err(ApplicationError::Conflict(
                "my vocabulary is empty".to_owned(),
            ));
        }
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().timestamp_millis();
        let mut content = self.content()?;
        let word_count = schema::in_immediate_transaction(&mut content, |transaction| {
            create_dataset(transaction, &id, name, created_at, &vocabulary_uids)
        })
        .map_err(map_snapshot_error)?;
        Ok(DatasetSummary {
            id,
            name: name.to_owned(),
            created_at,
            updated_at: created_at,
            preloaded: false,
            word_count,
        })
    }
}

fn create_dataset(
    transaction: &Transaction<'_>,
    id: &str,
    name: &str,
    created_at: i64,
    vocabulary_uids: &[String],
) -> rusqlite::Result<u32> {
    let existing = existing_sense_uids(transaction, vocabulary_uids)?;
    if existing.is_empty() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    transaction.execute(
        "INSERT INTO dataset(id, name, created_at, updated_at, preloaded)
         VALUES (?1, ?2, ?3, ?3, 0)",
        params![id, name, created_at],
    )?;
    for (sequence, sense_uid) in existing.iter().enumerate() {
        transaction.execute(
            "INSERT INTO dataset_item(dataset_id, sense_uid, sequence) VALUES (?1, ?2, ?3)",
            params![id, sense_uid, sequence as u32],
        )?;
    }
    schema::record_sync_change(transaction, "dataset", id, "upsert", created_at)?;
    Ok(existing.len() as u32)
}

fn existing_sense_uids(
    transaction: &Transaction<'_>,
    sense_uids: &[String],
) -> rusqlite::Result<Vec<String>> {
    let mut statement = transaction.prepare("SELECT 1 FROM sense WHERE uid = ?1")?;
    let mut existing = Vec::with_capacity(sense_uids.len());
    for sense_uid in sense_uids {
        if statement.exists([sense_uid])? {
            existing.push(sense_uid.clone());
        }
    }
    Ok(existing)
}

fn map_snapshot_error(error: rusqlite::Error) -> ApplicationError {
    if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
        ApplicationError::Conflict("my vocabulary has no available words".to_owned())
    } else {
        database_error(error)
    }
}
