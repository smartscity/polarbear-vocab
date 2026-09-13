use chrono::Utc;
use polarbear_vocab_application::{ApplicationError, DatasetRepository};
use polarbear_vocab_domain::{CsvImportResult, DatasetImportPlan, DatasetSummary, ImportedSense};
use rusqlite::{OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::{SqliteStore, database_error, distractor_index, schema};

impl DatasetRepository for SqliteStore {
    fn create_dataset(&self, name: &str) -> Result<DatasetSummary, ApplicationError> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().timestamp_millis();
        let mut content = self.content()?;
        schema::in_immediate_transaction(&mut content, |transaction| {
            transaction.execute(
                "INSERT INTO dataset(id, name, created_at, preloaded)
                 VALUES (?1, ?2, ?3, 0)",
                params![id, name, created_at],
            )?;
            Ok(())
        })
        .map_err(database_error)?;
        Ok(DatasetSummary {
            id,
            name: name.to_owned(),
            created_at,
            preloaded: false,
            word_count: 0,
        })
    }

    fn rename_dataset(&self, dataset_id: &str, name: &str) -> Result<(), ApplicationError> {
        let mut content = self.content()?;
        let changed = schema::in_immediate_transaction(&mut content, |transaction| {
            transaction.execute(
                "UPDATE dataset SET name = ?2 WHERE id = ?1",
                params![dataset_id, name],
            )
        })
        .map_err(database_error)?;
        require_change(changed, dataset_id)
    }

    fn delete_dataset(&self, dataset_id: &str) -> Result<(), ApplicationError> {
        let mut content = self.content()?;
        let changed = schema::in_immediate_transaction(&mut content, |transaction| {
            transaction.execute("DELETE FROM dataset WHERE id = ?1", [dataset_id])
        })
        .map_err(database_error)?;
        require_change(changed, dataset_id)
    }

    fn import_dataset(
        &self,
        dataset_id: &str,
        plan: &DatasetImportPlan,
    ) -> Result<CsvImportResult, ApplicationError> {
        let mut content = self.content()?;
        schema::in_immediate_transaction(&mut content, |transaction| {
            ensure_dataset(transaction, dataset_id)?;
            let source_id = ensure_import_source(transaction)?;
            let mut inserted_senses = 0;
            let mut updated_senses = 0;
            for (sequence, entry) in plan.entries.iter().enumerate() {
                if upsert_sense(transaction, entry, source_id)? {
                    inserted_senses += 1;
                } else {
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
            Ok(CsvImportResult {
                imported_items: plan.entries.len() as u32,
                inserted_senses,
                updated_senses,
            })
        })
        .map_err(database_error)
    }
}

fn ensure_dataset(transaction: &Transaction<'_>, dataset_id: &str) -> rusqlite::Result<()> {
    transaction.query_row("SELECT 1 FROM dataset WHERE id = ?1", [dataset_id], |_| {
        Ok(())
    })
}

fn ensure_import_source(transaction: &Transaction<'_>) -> rusqlite::Result<i64> {
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

fn upsert_sense(
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
