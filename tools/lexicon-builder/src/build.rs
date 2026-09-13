use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, ensure};
use rusqlite::{Connection, params};

use crate::model::{Entry, Manifest};

pub fn build(source_directory: &Path, output: &Path) -> anyhow::Result<()> {
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(source_directory.join("manifest.json"))?)?;
    let entries = read_entries(&source_directory.join("entries.tsv"))?;
    validate(&manifest, &entries)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = output.with_extension("db.building");
    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }
    let mut connection = Connection::open(&temporary)?;
    create_schema(&connection)?;
    populate(&mut connection, &manifest, &entries)?;
    connection.close().map_err(|(_, error)| error)?;
    fs::rename(&temporary, output)?;
    Ok(())
}

fn read_entries(path: &Path) -> anyhow::Result<Vec<Entry>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    reader
        .deserialize()
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn validate(manifest: &Manifest, entries: &[Entry]) -> anyhow::Result<()> {
    ensure!(
        !manifest.source.name.trim().is_empty(),
        "missing source name"
    );
    ensure!(
        !manifest.source.license.trim().is_empty(),
        "missing source license"
    );
    ensure!(!entries.is_empty(), "lexicon has no entries");
    let datasets: HashSet<&str> = manifest
        .datasets
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let mut sense_uids = HashSet::new();
    let mut examples = HashSet::new();
    for entry in entries {
        ensure!(
            sense_uids.insert(&entry.sense_uid),
            "duplicate sense_uid: {}",
            entry.sense_uid
        );
        ensure!(
            examples.insert(&entry.example_en),
            "duplicate example: {}",
            entry.example_en
        );
        ensure!(
            ["noun", "verb", "adjective", "adverb"].contains(&entry.pos.as_str()),
            "invalid POS: {}",
            entry.pos
        );
        ensure!(
            ["A1", "A2", "B1", "B2", "C1", "C2"].contains(&entry.cefr.as_str()),
            "invalid CEFR: {}",
            entry.cefr
        );
        ensure!(
            !entry.quiz_prompt_zh.trim().is_empty(),
            "missing Chinese prompt: {}",
            entry.sense_uid
        );
        ensure!(
            !entry.ipa_us.trim().is_empty() && !entry.ipa_uk.trim().is_empty(),
            "missing IPA: {}",
            entry.sense_uid
        );
        ensure!(
            !entry.example_en.trim().is_empty(),
            "missing primary example: {}",
            entry.sense_uid
        );
        for dataset_id in entry.dataset_ids() {
            ensure!(
                datasets.contains(dataset_id),
                "missing dataset {dataset_id} for {}",
                entry.sense_uid
            );
        }
    }
    Ok(())
}

fn create_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;
         CREATE TABLE source(id INTEGER PRIMARY KEY, name TEXT NOT NULL, license TEXT NOT NULL, url TEXT, attribution TEXT NOT NULL);
         CREATE TABLE schema_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO schema_meta(key, value) VALUES ('version', '2');
         CREATE TABLE dataset(id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL, preloaded INTEGER NOT NULL DEFAULT 0 CHECK(preloaded IN (0, 1)));
         CREATE TABLE word(id INTEGER PRIMARY KEY, uid TEXT NOT NULL UNIQUE, lemma TEXT NOT NULL, frequency_rank INTEGER);
         CREATE TABLE sense(id INTEGER PRIMARY KEY, uid TEXT NOT NULL UNIQUE, word_id INTEGER NOT NULL REFERENCES word(id), source_id INTEGER NOT NULL REFERENCES source(id), pos TEXT NOT NULL, quiz_prompt_zh TEXT NOT NULL, zh_gloss TEXT NOT NULL, en_definition TEXT NOT NULL, cefr TEXT NOT NULL);
         CREATE TABLE pronunciation(id INTEGER PRIMARY KEY, sense_id INTEGER NOT NULL REFERENCES sense(id), accent TEXT NOT NULL, ipa TEXT NOT NULL, UNIQUE(sense_id, accent));
         CREATE TABLE example(id INTEGER PRIMARY KEY, sense_id INTEGER NOT NULL REFERENCES sense(id), sentence_en TEXT NOT NULL, sentence_zh TEXT NOT NULL, is_primary INTEGER NOT NULL DEFAULT 0);
         CREATE TABLE dataset_item(dataset_id TEXT NOT NULL REFERENCES dataset(id) ON DELETE CASCADE, sense_uid TEXT NOT NULL REFERENCES sense(uid), sequence INTEGER, PRIMARY KEY(dataset_id, sense_uid));
         CREATE INDEX idx_dataset_item_dataset ON dataset_item(dataset_id);
         CREATE TABLE distractor_edge(prompt_sense_uid TEXT NOT NULL, candidate_sense_uid TEXT NOT NULL, score REAL NOT NULL, reason TEXT, PRIMARY KEY(prompt_sense_uid, candidate_sense_uid));",
    )
}

fn populate(
    connection: &mut Connection,
    manifest: &Manifest,
    entries: &[Entry],
) -> anyhow::Result<()> {
    let transaction = connection.transaction()?;
    transaction.execute(
        "INSERT INTO source(name, license, url, attribution) VALUES (?1, ?2, ?3, ?4)",
        params![
            manifest.source.name,
            manifest.source.license,
            manifest.source.url,
            manifest.source.attribution
        ],
    )?;
    for dataset in &manifest.datasets {
        transaction.execute(
            "INSERT INTO dataset(id, name, created_at, preloaded) VALUES (?1, ?2, 0, 1)",
            params![dataset.id, dataset.name],
        )?;
    }
    for (index, entry) in entries.iter().enumerate() {
        transaction.execute(
            "INSERT INTO word(uid, lemma, frequency_rank) VALUES (?1, ?2, ?3)",
            params![
                format!("{}.word", entry.lemma),
                entry.lemma,
                index as u32 + 1
            ],
        )?;
        let word_id = transaction.last_insert_rowid();
        transaction.execute(
            "INSERT INTO sense(uid, word_id, source_id, pos, quiz_prompt_zh, zh_gloss, en_definition, cefr) VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, ?7)",
            params![entry.sense_uid, word_id, entry.pos, entry.quiz_prompt_zh, entry.zh_gloss, entry.definition_en, entry.cefr],
        )?;
        let sense_id = transaction.last_insert_rowid();
        transaction.execute("INSERT INTO pronunciation(sense_id, accent, ipa) VALUES (?1, 'en-US', ?2), (?1, 'en-GB', ?3)", params![sense_id, entry.ipa_us, entry.ipa_uk])?;
        transaction.execute("INSERT INTO example(sense_id, sentence_en, sentence_zh, is_primary) VALUES (?1, ?2, ?3, 1)", params![sense_id, entry.example_en, entry.example_zh])?;
        for (sequence, dataset_id) in entry.dataset_ids().enumerate() {
            transaction.execute(
                "INSERT INTO dataset_item(dataset_id, sense_uid, sequence) VALUES (?1, ?2, ?3)",
                params![dataset_id, entry.sense_uid, index as u32 + sequence as u32],
            )?;
        }
    }
    insert_distractors(&transaction, entries)?;
    let violations: String = transaction.query_row(
        "SELECT COALESCE(group_concat(\"table\" || ':' || rowid), '')
         FROM pragma_foreign_key_check",
        [],
        |row| row.get(0),
    )?;
    ensure!(
        violations.is_empty(),
        "foreign-key violations: {violations}"
    );
    transaction.commit()?;
    Ok(())
}

fn insert_distractors(
    transaction: &rusqlite::Transaction<'_>,
    entries: &[Entry],
) -> anyhow::Result<()> {
    let offsets = [17_usize, 41, 73];
    for (index, entry) in entries.iter().enumerate() {
        let mut candidates = HashSet::new();
        for offset in offsets {
            let candidate = &entries[(index + offset) % entries.len()];
            ensure!(
                candidate.lemma != entry.lemma,
                "same lemma distractor for {}",
                entry.sense_uid
            );
            ensure!(
                candidate.zh_gloss != entry.zh_gloss,
                "ambiguous exact-gloss distractor for {}",
                entry.sense_uid
            );
            ensure!(
                candidates.insert(&candidate.sense_uid),
                "duplicate distractor for {}",
                entry.sense_uid
            );
            transaction.execute(
                "INSERT INTO distractor_edge(prompt_sense_uid, candidate_sense_uid, score, reason) VALUES (?1, ?2, ?3, 'curated-distance-v1')",
                params![entry.sense_uid, candidate.sense_uid, 1.0 - candidates.len() as f64 / 10.0],
            )?;
        }
    }
    Ok(())
}
