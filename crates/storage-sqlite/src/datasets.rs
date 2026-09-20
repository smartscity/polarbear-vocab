use std::collections::HashSet;

use chrono::Utc;
use polarbear_vocab_application::{ApplicationError, DatasetRepository};
use polarbear_vocab_domain::{
    CsvImportResult, DatasetImportPlan, DatasetImportStrategy, DatasetSummary, ImportedSense,
};
use rusqlite::{OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::{SqliteStore, database_error, dataset_order, distractor_index, read_model, schema};

impl DatasetRepository for SqliteStore {
    fn create_dataset(&self, name: &str) -> Result<DatasetSummary, ApplicationError> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().timestamp_millis();
        let mut content = self.content()?;
        schema::in_immediate_transaction(&mut content, |transaction| {
            transaction.execute(
                "INSERT INTO dataset(id, name, created_at, updated_at, preloaded)
                 VALUES (?1, ?2, ?3, ?3, 0)",
                params![id, name, created_at],
            )?;
            schema::record_sync_change(transaction, "dataset", &id, "upsert", created_at)?;
            Ok(())
        })
        .map_err(database_error)?;
        Ok(DatasetSummary {
            id,
            name: name.to_owned(),
            created_at,
            updated_at: created_at,
            preloaded: false,
            word_count: 0,
        })
    }

    fn reorder_datasets(&self, dataset_ids: &[String]) -> Result<(), ApplicationError> {
        let available: HashSet<String> = {
            let content = self.content()?;
            read_model::list_datasets(&content)
                .map_err(database_error)?
                .into_iter()
                .map(|dataset| dataset.id)
                .collect()
        };
        let requested: HashSet<&str> = dataset_ids.iter().map(String::as_str).collect();
        if requested.len() != dataset_ids.len()
            || available.len() != requested.len()
            || !available.iter().all(|id| requested.contains(id.as_str()))
        {
            return Err(ApplicationError::InvalidInput(
                "dataset order must contain every current dataset exactly once".to_owned(),
            ));
        }
        let mut user = self.user()?;
        dataset_order::write(&mut user, dataset_ids)
    }

    fn rename_dataset(&self, dataset_id: &str, name: &str) -> Result<(), ApplicationError> {
        let mut content = self.content()?;
        let changed_at = Utc::now().timestamp_millis();
        let changed = schema::in_immediate_transaction(&mut content, |transaction| {
            let changed = transaction.execute(
                "UPDATE dataset SET name = ?2, updated_at = ?3 WHERE id = ?1",
                params![dataset_id, name, changed_at],
            )?;
            if changed > 0 {
                schema::record_sync_change(
                    transaction,
                    "dataset",
                    dataset_id,
                    "upsert",
                    changed_at,
                )?;
            }
            Ok(changed)
        })
        .map_err(database_error)?;
        require_change(changed, dataset_id)
    }

    fn delete_dataset(&self, dataset_id: &str) -> Result<(), ApplicationError> {
        let mut content = self.content()?;
        let changed_at = Utc::now().timestamp_millis();
        let changed = schema::in_immediate_transaction(&mut content, |transaction| {
            let changed = transaction.execute("DELETE FROM dataset WHERE id = ?1", [dataset_id])?;
            if changed > 0 {
                schema::record_sync_change(
                    transaction,
                    "dataset",
                    dataset_id,
                    "delete",
                    changed_at,
                )?;
            }
            Ok(changed)
        })
        .map_err(database_error)?;
        require_change(changed, dataset_id)
    }

    fn import_dataset(
        &self,
        dataset_id: &str,
        plan: &DatasetImportPlan,
        strategy: DatasetImportStrategy,
    ) -> Result<CsvImportResult, ApplicationError> {
        let mut content = self.content()?;
        schema::in_immediate_transaction(&mut content, |transaction| {
            ensure_dataset(transaction, dataset_id)?;
            if strategy == DatasetImportStrategy::ReplaceDataset {
                transaction.execute(
                    "DELETE FROM dataset_item WHERE dataset_id = ?1",
                    [dataset_id],
                )?;
            }
            let source_id = ensure_import_source(transaction)?;
            let mut inserted_senses = 0;
            let mut updated_senses = 0;
            for (sequence, entry) in plan.entries.iter().enumerate() {
                let exists = sense_exists(transaction, &entry.sense_uid)?;
                if !exists || strategy != DatasetImportStrategy::AddOnly {
                    upsert_sense(transaction, entry, source_id)?;
                }
                if !exists {
                    inserted_senses += 1;
                } else if strategy != DatasetImportStrategy::AddOnly {
                    updated_senses += 1;
                }
                transaction.execute(
                    "INSERT INTO dataset_item(dataset_id, sense_uid, sequence)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(dataset_id, sense_uid) DO UPDATE
                     SET sequence = excluded.sequence",
                    params![dataset_id, entry.sense_uid, sequence as u32],
                )?;
            }
            let imported_uids: Vec<String> = plan
                .entries
                .iter()
                .map(|entry| entry.sense_uid.clone())
                .collect();
            distractor_index::refresh_for(transaction, &imported_uids)?;
            let changed_at = Utc::now().timestamp_millis();
            transaction.execute(
                "UPDATE dataset SET updated_at = ?2 WHERE id = ?1",
                params![dataset_id, changed_at],
            )?;
            schema::record_sync_change(transaction, "dataset", dataset_id, "upsert", changed_at)?;
            Ok(CsvImportResult {
                imported_items: plan.entries.len() as u32,
                inserted_senses,
                updated_senses,
            })
        })
        .map_err(database_error)
    }

    fn export_dataset(&self, dataset_id: &str, path: &str) -> Result<u32, ApplicationError> {
        let content = self.content()?;
        ensure_dataset_exists(&content, dataset_id)?;
        let mut statement = content
            .prepare(
                "SELECT s.uid, w.lemma, s.pos, s.quiz_prompt_zh, s.zh_gloss,
                    COALESCE((SELECT ipa FROM pronunciation WHERE sense_id = s.id AND accent = 'en-US'), ''),
                    COALESCE((SELECT ipa FROM pronunciation WHERE sense_id = s.id AND accent = 'en-GB'), ''),
                    COALESCE(e.sentence_en, ''), COALESCE(e.sentence_zh, '')
                 FROM dataset_item item
                 JOIN sense s ON s.uid = item.sense_uid
                 JOIN word w ON w.id = s.word_id
                 LEFT JOIN example e ON e.sense_id = s.id AND e.is_primary = 1
                 WHERE item.dataset_id = ?1 ORDER BY item.sequence, s.uid",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([dataset_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })
            .map_err(database_error)?;
        let mut writer = csv::Writer::from_path(path).map_err(csv_error)?;
        writer
            .write_record([
                "sense_uid",
                "lemma",
                "pos",
                "quiz_prompt_zh",
                "gloss_zh",
                "ipa_us",
                "ipa_uk",
                "example_en",
                "example_zh",
            ])
            .map_err(csv_error)?;
        let mut count = 0;
        for row in rows {
            let row = row.map_err(database_error)?;
            writer.serialize(row).map_err(csv_error)?;
            count += 1;
        }
        writer
            .flush()
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))?;
        Ok(count)
    }
}

fn ensure_dataset_exists(
    connection: &rusqlite::Connection,
    dataset_id: &str,
) -> Result<(), ApplicationError> {
    connection
        .query_row("SELECT 1 FROM dataset WHERE id = ?1", [dataset_id], |_| {
            Ok(())
        })
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| ApplicationError::NotFound(dataset_id.to_owned()))
}

fn sense_exists(transaction: &Transaction<'_>, sense_uid: &str) -> rusqlite::Result<bool> {
    Ok(transaction
        .query_row(
            "SELECT 1 FROM sense WHERE uid = ?1",
            [sense_uid],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

fn ensure_dataset(transaction: &Transaction<'_>, dataset_id: &str) -> rusqlite::Result<()> {
    transaction.query_row("SELECT 1 FROM dataset WHERE id = ?1", [dataset_id], |_| {
        Ok(())
    })
}

pub(crate) fn ensure_import_source(transaction: &Transaction<'_>) -> rusqlite::Result<i64> {
    let existing = transaction
        .query_row(
            "SELECT id FROM source WHERE name = 'User CSV Import'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    transaction.execute(
        "INSERT INTO source(name, license, url, attribution)
         VALUES ('User CSV Import', 'User supplied', NULL, 'Imported locally by the user')",
        [],
    )?;
    Ok(transaction.last_insert_rowid())
}

pub(crate) fn upsert_sense(
    transaction: &Transaction<'_>,
    entry: &ImportedSense,
    source_id: i64,
) -> rusqlite::Result<bool> {
    let existing_word_id: Option<i64> = transaction
        .query_row(
            "SELECT word_id FROM sense WHERE uid = ?1",
            [&entry.sense_uid],
            |row| row.get(0),
        )
        .optional()?;
    let inserted = existing_word_id.is_none();
    let word_id = match existing_word_id {
        Some(word_id) => word_id,
        None => insert_word(transaction, entry)?,
    };
    if inserted {
        transaction.execute(
            "INSERT INTO sense(
                uid, word_id, source_id, pos, quiz_prompt_zh, zh_gloss,
                en_definition, cefr
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', 'UNSPECIFIED')",
            params![
                entry.sense_uid,
                word_id,
                source_id,
                entry.part_of_speech,
                entry.quiz_prompt_zh,
                entry.gloss_zh
            ],
        )?;
    } else {
        transaction.execute(
            "UPDATE word SET lemma = ?2 WHERE id = ?1",
            params![word_id, entry.lemma],
        )?;
        transaction.execute(
            "UPDATE sense SET source_id = ?2, pos = ?3, quiz_prompt_zh = ?4,
                              zh_gloss = ?5 WHERE uid = ?1",
            params![
                entry.sense_uid,
                source_id,
                entry.part_of_speech,
                entry.quiz_prompt_zh,
                entry.gloss_zh
            ],
        )?;
    }
    let sense_id: i64 = transaction.query_row(
        "SELECT id FROM sense WHERE uid = ?1",
        [&entry.sense_uid],
        |row| row.get(0),
    )?;
    upsert_pronunciation(transaction, sense_id, entry)?;
    upsert_example(transaction, sense_id, entry)?;
    Ok(inserted)
}

fn insert_word(transaction: &Transaction<'_>, entry: &ImportedSense) -> rusqlite::Result<i64> {
    transaction.execute(
        "INSERT INTO word(uid, lemma, frequency_rank) VALUES (?1, ?2, NULL)",
        params![Uuid::new_v4().to_string(), entry.lemma],
    )?;
    Ok(transaction.last_insert_rowid())
}

fn upsert_pronunciation(
    transaction: &Transaction<'_>,
    sense_id: i64,
    entry: &ImportedSense,
) -> rusqlite::Result<()> {
    for (accent, ipa) in [("en-US", &entry.ipa_us), ("en-GB", &entry.ipa_uk)] {
        transaction.execute(
            "INSERT INTO pronunciation(sense_id, accent, ipa) VALUES (?1, ?2, ?3)
             ON CONFLICT(sense_id, accent) DO UPDATE SET ipa = excluded.ipa",
            params![sense_id, accent, ipa],
        )?;
    }
    Ok(())
}

fn upsert_example(
    transaction: &Transaction<'_>,
    sense_id: i64,
    entry: &ImportedSense,
) -> rusqlite::Result<()> {
    let updated = transaction.execute(
        "UPDATE example SET sentence_en = ?2, sentence_zh = ?3
         WHERE sense_id = ?1 AND is_primary = 1",
        params![sense_id, entry.example_en, entry.example_zh],
    )?;
    if updated == 0 {
        transaction.execute(
            "INSERT INTO example(sense_id, sentence_en, sentence_zh, is_primary)
             VALUES (?1, ?2, ?3, 1)",
            params![sense_id, entry.example_en, entry.example_zh],
        )?;
    }
    Ok(())
}

fn require_change(changed: usize, dataset_id: &str) -> Result<(), ApplicationError> {
    if changed == 0 {
        return Err(ApplicationError::NotFound(dataset_id.to_owned()));
    }
    Ok(())
}

fn csv_error(error: csv::Error) -> ApplicationError {
    ApplicationError::Infrastructure(format!("cannot export dataset CSV: {error}"))
}
