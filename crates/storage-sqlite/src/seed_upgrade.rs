use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use polarbear_vocab_application::ApplicationError;
use rusqlite::{Connection, MAIN_DB, OpenFlags, OptionalExtension, Transaction, params};
use sha2::{Digest, Sha256};

use crate::{database_error, migrate_content_schema};

pub(crate) fn sync_preloaded_seed(seed: &Path, destination: &Path) -> Result<(), ApplicationError> {
    let marker = format!("applied_seed:{}", file_digest(seed)?);
    let mut content = Connection::open(destination).map_err(database_error)?;
    let applied: Option<String> = content
        .query_row(
            "SELECT value FROM schema_meta WHERE key = ?1",
            [&marker],
            |row| row.get(0),
        )
        .optional()
        .map_err(database_error)?;
    if applied.is_some() {
        return Ok(());
    }
    validate_seed(seed)?;
    let backup_directory = tempfile::Builder::new()
        .prefix("polarbear-seed-upgrade-")
        .tempdir_in(destination.parent().unwrap_or_else(|| Path::new(".")))
        .map_err(|error| infrastructure(format!("cannot prepare seed backup: {error}")))?;
    let backup = backup_directory.path().join("content.db");
    content
        .backup(MAIN_DB, &backup, None)
        .map_err(database_error)?;
    let result =
        migrate_content_schema(&mut content).and_then(|()| merge_seed(&mut content, seed, &marker));
    if let Err(error) = result {
        if let Err(restore) = content.restore(MAIN_DB, &backup, None::<fn(_)>) {
            let retained = backup_directory.keep();
            return Err(infrastructure(format!(
                "seed upgrade failed ({error}); restore failed ({restore}); recovery backup retained at {}",
                retained.display()
            )));
        }
        return Err(error);
    }
    Ok(())
}

fn file_digest(path: &Path) -> Result<String, ApplicationError> {
    let file = File::open(path).map_err(|error| {
        infrastructure(format!("cannot read bundled content database: {error}"))
    })?;
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| infrastructure(format!("cannot hash bundled content: {error}")))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn validate_seed(path: &Path) -> Result<(), ApplicationError> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let seed = Connection::open_with_flags(path, flags).map_err(database_error)?;
    let version: String = seed
        .query_row(
            "SELECT value FROM schema_meta WHERE key = 'version'",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if version != "3" {
        return Err(infrastructure(format!(
            "unsupported bundled content schema {version}"
        )));
    }
    let integrity: String = seed
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(database_error)?;
    if integrity != "ok" {
        return Err(infrastructure(
            "bundled content failed integrity check".into(),
        ));
    }
    let violations: i64 = seed
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .map_err(database_error)?;
    if violations != 0 {
        return Err(infrastructure(
            "bundled content failed foreign-key check".into(),
        ));
    }
    Ok(())
}

fn merge_seed(content: &mut Connection, seed: &Path, marker: &str) -> Result<(), ApplicationError> {
    let path = seed.to_string_lossy();
    content
        .execute("ATTACH DATABASE ?1 AS bundled_seed", [path.as_ref()])
        .map_err(database_error)?;
    let merged = merge_transaction(content, marker).map_err(database_error);
    let detached = content
        .execute_batch("DETACH DATABASE bundled_seed")
        .map_err(database_error);
    merged.and(detached)
}

fn merge_transaction(content: &mut Connection, marker: &str) -> rusqlite::Result<()> {
    content.execute_batch("PRAGMA foreign_keys = ON")?;
    let transaction =
        content.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    map_sources(&transaction)?;
    insert_senses(&transaction)?;
    insert_details(&transaction)?;
    insert_memberships(&transaction)?;
    let violations: i64 =
        transaction.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    if violations != 0 {
        return Err(rusqlite::Error::InvalidQuery);
    }
    transaction.execute(
        "INSERT INTO schema_meta(key, value) VALUES (?1, '1')",
        [marker],
    )?;
    transaction
        .execute_batch("DROP TABLE temp.new_seed_sense; DROP TABLE temp.seed_source_map;")?;
    transaction.commit()
}

fn map_sources(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    transaction.execute_batch(
        "CREATE TEMP TABLE seed_source_map(seed_id INTEGER PRIMARY KEY, destination_id INTEGER NOT NULL);",
    )?;
    let mut statement = transaction.prepare(
        "SELECT id, name, license, url, attribution FROM bundled_seed.source ORDER BY id",
    )?;
    let sources = statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    for source in sources {
        let (seed_id, name, license, url, attribution) = source?;
        let destination_id =
            source_destination_id(transaction, &name, &license, url.as_deref(), &attribution)?;
        transaction.execute(
            "INSERT INTO temp.seed_source_map(seed_id, destination_id) VALUES (?1, ?2)",
            params![seed_id, destination_id],
        )?;
    }
    Ok(())
}

fn source_destination_id(
    transaction: &Transaction<'_>,
    name: &str,
    license: &str,
    url: Option<&str>,
    attribution: &str,
) -> rusqlite::Result<i64> {
    let existing: Option<i64> = transaction
        .query_row(
            "SELECT id FROM main.source WHERE name = ?1 AND license = ?2
             AND url IS ?3 AND attribution = ?4 ORDER BY id LIMIT 1",
            params![name, license, url, attribution],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    transaction.execute(
        "INSERT INTO main.source(name, license, url, attribution) VALUES (?1, ?2, ?3, ?4)",
        params![name, license, url, attribution],
    )?;
    Ok(transaction.last_insert_rowid())
}

fn insert_senses(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    transaction.execute_batch(
        "CREATE TEMP TABLE new_seed_sense(uid TEXT PRIMARY KEY);
         INSERT INTO temp.new_seed_sense(uid)
           SELECT uid FROM bundled_seed.sense
           WHERE uid NOT IN (SELECT uid FROM main.sense);
         INSERT OR IGNORE INTO main.word(uid, lemma, frequency_rank)
           SELECT uid, lemma, frequency_rank FROM bundled_seed.word;
         INSERT INTO main.sense(uid, word_id, source_id, pos, quiz_prompt_zh,
           zh_gloss, en_definition, cefr)
           SELECT sense.uid, word.id, source_map.destination_id, sense.pos,
             sense.quiz_prompt_zh, sense.zh_gloss, sense.en_definition, sense.cefr
           FROM bundled_seed.sense AS sense
           JOIN bundled_seed.word AS bundled_word ON bundled_word.id = sense.word_id
           JOIN main.word AS word ON word.uid = bundled_word.uid
           JOIN temp.seed_source_map AS source_map ON source_map.seed_id = sense.source_id
           JOIN temp.new_seed_sense AS new_sense ON new_sense.uid = sense.uid;",
    )
}

fn insert_details(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    transaction.execute_batch(
        "INSERT INTO main.pronunciation(sense_id, accent, ipa)
           SELECT content_sense.id, pronunciation.accent, pronunciation.ipa
           FROM bundled_seed.pronunciation AS pronunciation
           JOIN bundled_seed.sense AS seed_sense ON seed_sense.id = pronunciation.sense_id
           JOIN temp.new_seed_sense AS new_sense ON new_sense.uid = seed_sense.uid
           JOIN main.sense AS content_sense ON content_sense.uid = seed_sense.uid;
         INSERT INTO main.example(sense_id, sentence_en, sentence_zh, is_primary)
           SELECT content_sense.id, example.sentence_en, example.sentence_zh,
             example.is_primary
           FROM bundled_seed.example AS example
           JOIN bundled_seed.sense AS seed_sense ON seed_sense.id = example.sense_id
           JOIN temp.new_seed_sense AS new_sense ON new_sense.uid = seed_sense.uid
           JOIN main.sense AS content_sense ON content_sense.uid = seed_sense.uid;",
    )
}

fn insert_memberships(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    transaction.execute_batch(
        "INSERT OR IGNORE INTO main.dataset(id, name, created_at, updated_at, preloaded)
           SELECT id, name, created_at, updated_at, preloaded
           FROM bundled_seed.dataset WHERE preloaded = 1;
         INSERT OR IGNORE INTO main.dataset_item(dataset_id, sense_uid, sequence)
           SELECT item.dataset_id, item.sense_uid, item.sequence
           FROM bundled_seed.dataset_item AS item
           JOIN main.dataset AS dataset ON dataset.id = item.dataset_id
           JOIN main.sense AS sense ON sense.uid = item.sense_uid;
         UPDATE main.dataset_item AS item SET sequence = (
           SELECT seed_item.sequence FROM bundled_seed.dataset_item AS seed_item
           WHERE seed_item.dataset_id = item.dataset_id
             AND seed_item.sense_uid = item.sense_uid
         ) WHERE EXISTS (
           SELECT 1 FROM bundled_seed.dataset_item AS seed_item
           WHERE seed_item.dataset_id = item.dataset_id
             AND seed_item.sense_uid = item.sense_uid
         );
         INSERT OR IGNORE INTO main.distractor_edge(prompt_sense_uid,
           candidate_sense_uid, score, reason)
           SELECT edge.prompt_sense_uid, edge.candidate_sense_uid, edge.score,
             edge.reason
           FROM bundled_seed.distractor_edge AS edge
           JOIN main.sense AS prompt ON prompt.uid = edge.prompt_sense_uid
           JOIN main.sense AS candidate ON candidate.uid = edge.candidate_sense_uid;",
    )
}

fn infrastructure(message: String) -> ApplicationError {
    ApplicationError::Infrastructure(message)
}
