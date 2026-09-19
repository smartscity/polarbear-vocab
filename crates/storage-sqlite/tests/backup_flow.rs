use std::path::{Path, PathBuf};

use polarbear_vocab_application::{BackupRepository, DatasetRepository, SettingsPort};
use polarbear_vocab_domain::SettingsDto;
use polarbear_vocab_storage_sqlite::{DatabasePaths, SqliteStore};
use rusqlite::Connection;
use tempfile::TempDir;

#[test]
fn backup_round_trip_restores_both_databases_and_keeps_a_safety_copy() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let original = store.create_dataset("Original").unwrap();
    let settings = SettingsDto {
        ui_theme: "dark".to_owned(),
        ..SettingsDto::default()
    };
    store.update_settings(&settings).unwrap();
    let backup = fixture
        .directory
        .path()
        .join("saved.polarbear-vocab-backup");

    let status = store
        .export_backup(backup.to_str().unwrap(), "0.1.0")
        .unwrap();
    store.rename_dataset(&original.id, "Changed").unwrap();
    store
        .update_settings(&SettingsDto {
            ui_theme: "light".to_owned(),
            ..SettingsDto::default()
        })
        .unwrap();
    let restored = store
        .import_backup(backup.to_str().unwrap(), "0.1.0")
        .unwrap();

    assert!(status.last_backup_at.is_some());
    assert_eq!(store.get_settings().unwrap(), settings);
    let datasets = polarbear_vocab_application::HomeQueryPort::list_datasets(&store).unwrap();
    assert_eq!(
        datasets
            .iter()
            .find(|dataset| dataset.id == original.id)
            .unwrap()
            .name,
        "Original"
    );
    assert!(Path::new(&restored.automatic_backup_path).is_file());
}

#[test]
fn invalid_backup_is_rejected_before_existing_data_is_changed() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let dataset = store.create_dataset("Keep this data").unwrap();
    let invalid = fixture
        .directory
        .path()
        .join("invalid.polarbear-vocab-backup");
    std::fs::write(&invalid, b"not a ZIP archive").unwrap();

    assert!(
        store
            .import_backup(invalid.to_str().unwrap(), "0.1.0")
            .is_err()
    );
    let datasets = polarbear_vocab_application::HomeQueryPort::list_datasets(&store).unwrap();
    assert!(datasets.iter().any(|candidate| candidate.id == dataset.id));
    assert!(!fixture.directory.path().read_dir().unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("before-restore-")
    }));
}

#[test]
fn automatic_backups_are_versioned_once_per_day_and_can_be_restored() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let dataset = store.create_dataset("Original").unwrap();

    let first = store.ensure_automatic_backup("0.1.0").unwrap();
    let second = store.ensure_automatic_backup("0.1.0").unwrap();

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(first[0].reason, "automatic");
    assert!(first[0].size_bytes > 0);
    store.rename_dataset(&dataset.id, "Changed").unwrap();
    store.restore_backup_version(&first[0].id, "0.1.0").unwrap();

    let datasets = polarbear_vocab_application::HomeQueryPort::list_datasets(&store).unwrap();
    assert_eq!(
        datasets
            .iter()
            .find(|candidate| candidate.id == dataset.id)
            .unwrap()
            .name,
        "Original"
    );
    let versions = store.list_backup_versions().unwrap();
    assert_eq!(versions.len(), 2);
    assert!(
        versions
            .iter()
            .any(|version| version.reason == "preRestore")
    );
}

struct Fixture {
    directory: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        create_content(&directory.path().join("content.db"));
        Self { directory }
    }

    fn store(&self) -> SqliteStore {
        SqliteStore::open(&DatabasePaths {
            content: self.directory.path().join("content.db"),
            user: self.directory.path().join("user.db"),
        })
        .unwrap()
    }
}

fn create_content(path: &PathBuf) {
    Connection::open(path)
        .unwrap()
        .execute_batch(
            "CREATE TABLE schema_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('version', '2');
             CREATE TABLE source(id INTEGER PRIMARY KEY, name TEXT, license TEXT, url TEXT, attribution TEXT);
             CREATE TABLE dataset(id TEXT PRIMARY KEY, name TEXT, created_at INTEGER, preloaded INTEGER);
             CREATE TABLE word(id INTEGER PRIMARY KEY, uid TEXT UNIQUE, lemma TEXT, frequency_rank INTEGER);
             CREATE TABLE sense(id INTEGER PRIMARY KEY, uid TEXT UNIQUE, word_id INTEGER, source_id INTEGER, pos TEXT, quiz_prompt_zh TEXT, zh_gloss TEXT, en_definition TEXT, cefr TEXT);
             CREATE TABLE pronunciation(id INTEGER PRIMARY KEY, sense_id INTEGER, accent TEXT, ipa TEXT, UNIQUE(sense_id, accent));
             CREATE TABLE example(id INTEGER PRIMARY KEY, sense_id INTEGER, sentence_en TEXT, sentence_zh TEXT, is_primary INTEGER);
             CREATE TABLE dataset_item(dataset_id TEXT, sense_uid TEXT, sequence INTEGER, PRIMARY KEY(dataset_id, sense_uid));
             CREATE TABLE distractor_edge(prompt_sense_uid TEXT, candidate_sense_uid TEXT, score REAL, reason TEXT, PRIMARY KEY(prompt_sense_uid, candidate_sense_uid));",
        )
        .unwrap();
}
