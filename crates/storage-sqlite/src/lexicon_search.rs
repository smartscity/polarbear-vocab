use chrono::Utc;
use polarbear_vocab_application::{ApplicationError, LexiconRepository};
use polarbear_vocab_domain::LexiconEntryDto;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{SqliteStore, database_error};

struct ContentEntry {
    sense_uid: String,
    lemma: String,
    ipa: String,
    part_of_speech: String,
    zh_gloss: String,
    example_en: String,
    dataset_names: Vec<String>,
}

impl LexiconRepository for SqliteStore {
    fn search_lexicon(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<LexiconEntryDto>, ApplicationError> {
        let entries = {
            let content = self.content()?;
            search_content(&content, query, limit).map_err(database_error)?
        };
        let user = self.user()?;
        entries
            .into_iter()
            .map(|entry| enrich_entry(&user, entry).map_err(database_error))
            .collect()
    }

    fn add_to_my_vocabulary(&self, sense_uid: &str) -> Result<(), ApplicationError> {
        let exists = self
            .content()?
            .query_row(
                "SELECT 1 FROM sense WHERE uid = ?1",
                [sense_uid],
                |_| Ok(()),
            )
            .optional()
            .map_err(database_error)?
            .is_some();
        if !exists {
            return Err(ApplicationError::NotFound(sense_uid.to_owned()));
        }
        self.user()?
            .execute(
                "INSERT INTO my_vocabulary(sense_uid, added_at) VALUES (?1, ?2)
                 ON CONFLICT(sense_uid) DO NOTHING",
                params![sense_uid, Utc::now().timestamp_millis()],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn remove_from_my_vocabulary(&self, sense_uid: &str) -> Result<(), ApplicationError> {
        self.user()?
            .execute(
                "DELETE FROM my_vocabulary WHERE sense_uid = ?1",
                [sense_uid],
            )
            .map_err(database_error)?;
        Ok(())
    }
}

impl SqliteStore {
    pub(crate) fn my_vocabulary_uids(&self) -> Result<Vec<String>, ApplicationError> {
        let user = self.user()?;
        let mut statement = user
            .prepare("SELECT sense_uid FROM my_vocabulary ORDER BY added_at, sense_uid")
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| row.get(0))
            .map_err(database_error)?;
        rows.collect::<rusqlite::Result<_>>()
            .map_err(database_error)
    }
}

fn search_content(
    connection: &Connection,
    query: &str,
    limit: u32,
) -> rusqlite::Result<Vec<ContentEntry>> {
    let pattern = format!("{}%", escape_like(query));
    let mut statement = connection.prepare(
        "SELECT s.uid, w.lemma, COALESCE(p.ipa, ''), s.pos, s.zh_gloss,
                COALESCE(e.sentence_en, '')
         FROM sense s
         JOIN word w ON w.id = s.word_id
         LEFT JOIN pronunciation p ON p.sense_id = s.id AND p.accent = 'en-US'
         LEFT JOIN example e ON e.sense_id = s.id AND e.is_primary = 1
         WHERE lower(w.lemma) LIKE lower(?1) ESCAPE '\\'
         ORDER BY CASE WHEN lower(w.lemma) = lower(?2) THEN 0 ELSE 1 END,
                  length(w.lemma), w.lemma, s.uid
         LIMIT ?3",
    )?;
    let rows = statement.query_map(params![pattern, query, limit], |row| {
        Ok(ContentEntry {
            sense_uid: row.get(0)?,
            lemma: row.get(1)?,
            ipa: row.get(2)?,
            part_of_speech: row.get(3)?,
            zh_gloss: row.get(4)?,
            example_en: row.get(5)?,
            dataset_names: Vec::new(),
        })
    })?;
    let mut entries = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    for entry in &mut entries {
        entry.dataset_names = dataset_names(connection, &entry.sense_uid)?;
    }
    Ok(entries)
}

fn dataset_names(connection: &Connection, sense_uid: &str) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT d.name FROM dataset_item item
         JOIN dataset d ON d.id = item.dataset_id
         WHERE item.sense_uid = ?1 ORDER BY d.preloaded DESC, d.name",
    )?;
    let rows = statement.query_map([sense_uid], |row| row.get(0))?;
    rows.collect()
}

fn enrich_entry(user: &Connection, entry: ContentEntry) -> rusqlite::Result<LexiconEntryDto> {
    let stats = user
        .query_row(
            "SELECT attempt_count, correct_count, wrong_count
             FROM word_stat WHERE sense_uid = ?1",
            [&entry.sense_uid],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .unwrap_or((0, 0, 0));
    let in_my_vocabulary = user
        .query_row(
            "SELECT 1 FROM my_vocabulary WHERE sense_uid = ?1",
            [&entry.sense_uid],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    Ok(LexiconEntryDto {
        sense_uid: entry.sense_uid,
        lemma: entry.lemma,
        ipa: entry.ipa,
        part_of_speech: entry.part_of_speech,
        zh_gloss: entry.zh_gloss,
        example_en: entry.example_en,
        dataset_names: entry.dataset_names,
        attempt_count: stats.0,
        correct_count: stats.1,
        wrong_count: stats.2,
        in_my_vocabulary,
    })
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
