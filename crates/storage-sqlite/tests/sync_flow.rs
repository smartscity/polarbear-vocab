use std::path::{Path, PathBuf};

use polarbear_vocab_application::{
    ArticleRepository, DatasetRepository, HomeQueryPort, LexiconRepository, StudyPort,
    SyncRepository,
};
use polarbear_vocab_domain::{
    CollectionSpec, DatasetImportPlan, DatasetImportStrategy, ImportedSense,
};
use polarbear_vocab_storage_sqlite::{DatabasePaths, SqliteStore};
use rusqlite::{Connection, params};
use tempfile::TempDir;

#[test]
fn two_devices_exchange_incremental_changes_in_both_directions() {
    let fixture = Fixture::new();
    let mac = fixture.store("mac");
    let phone = fixture.store("phone");
    let travel = create_dataset(&mac, "Travel", "travel.v.01");
    let article = mac
        .save_article("Mac article", "Created on macOS.")
        .unwrap();
    mac.save_article_translation(&article.id, "在 macOS 上创建。")
        .unwrap();
    mac.add_to_my_vocabulary("word1.n.01").unwrap();
    answer_one_word(&mac);
    let first = fixture.sync_path("mac-1");

    let exported = mac.export_sync(path(&first), "0.2.7").unwrap();
    let imported = phone.import_sync(path(&first), "0.2.7").unwrap();

    assert_eq!(exported.dataset_count, 1);
    assert_eq!(imported.review_event_count, 1);
    assert_eq!(phone.get_home("test").unwrap().totals.explored, 1);
    assert_eq!(
        phone.list_articles().unwrap()[0].translated_body.as_deref(),
        Some("在 macOS 上创建。")
    );
    assert!(phone.search_lexicon("word1", 1).unwrap()[0].in_my_vocabulary);
    phone.remove_from_my_vocabulary("word1.n.01").unwrap();
    phone.rename_dataset(&travel, "Travel from phone").unwrap();
    create_dataset(&phone, "Phone words", "phone.n.01");
    phone
        .save_article("Phone article", "Created on iPhone.")
        .unwrap();
    mac.save_article("Second Mac article", "Only this is new.")
        .unwrap();

    let second = fixture.sync_path("phone-1");
    phone.export_sync(path(&second), "0.2.7").unwrap();
    mac.import_sync(path(&second), "0.2.7").unwrap();
    let third = fixture.sync_path("mac-2");
    let incremental = mac.export_sync(path(&third), "0.2.7").unwrap();
    phone.import_sync(path(&third), "0.2.7").unwrap();

    assert_eq!(incremental.dataset_count, 0);
    assert_eq!(incremental.article_count, 1);
    assert_eq!(incremental.review_event_count, 0);
    assert_eq!(dataset_names(&mac), dataset_names(&phone));
    assert_eq!(article_titles(&mac), article_titles(&phone));
    assert!(!mac.search_lexicon("word1", 1).unwrap()[0].in_my_vocabulary);
    assert!(
        mac.import_sync(path(&second), "0.2.7")
            .unwrap()
            .already_applied
    );
}

#[test]
fn concurrent_dataset_edits_are_preserved_as_conflict_copies() {
    let fixture = Fixture::new();
    let mac = fixture.store("mac");
    let phone = fixture.store("phone");
    let dataset_id = create_dataset(&mac, "Shared", "shared.v.01");
    let initial = fixture.sync_path("initial");
    mac.export_sync(path(&initial), "0.2.7").unwrap();
    phone.import_sync(path(&initial), "0.2.7").unwrap();
    mac.rename_dataset(&dataset_id, "Mac edit").unwrap();
    phone.rename_dataset(&dataset_id, "Phone edit").unwrap();
    let mac_change = fixture.sync_path("mac-change");
    let phone_change = fixture.sync_path("phone-change");
    mac.export_sync(path(&mac_change), "0.2.7").unwrap();
    phone.export_sync(path(&phone_change), "0.2.7").unwrap();

    let mac_result = mac.import_sync(path(&phone_change), "0.2.7").unwrap();
    let phone_result = phone.import_sync(path(&mac_change), "0.2.7").unwrap();

    assert_eq!(mac_result.conflict_count, 1);
    assert_eq!(phone_result.conflict_count, 1);
    assert!(dataset_names(&mac).iter().any(|name| name == "Mac edit"));
    assert!(
        dataset_names(&mac)
            .iter()
            .any(|name| name.contains("Conflict from"))
    );
}

fn create_dataset(store: &SqliteStore, name: &str, sense_uid: &str) -> String {
    let dataset = store.create_dataset(name).unwrap();
    store
        .import_dataset(
            &dataset.id,
            &DatasetImportPlan {
                entries: vec![ImportedSense {
                    sense_uid: sense_uid.to_owned(),
                    lemma: sense_uid.split('.').next().unwrap().to_owned(),
                    part_of_speech: "noun".to_owned(),
                    quiz_prompt_zh: name.to_owned(),
                    gloss_zh: name.to_owned(),
                    ipa_us: String::new(),
                    ipa_uk: String::new(),
                    example_en: String::new(),
                    example_zh: String::new(),
                }],
            },
            DatasetImportStrategy::ReplaceDataset,
        )
        .unwrap();
    dataset.id
}

fn answer_one_word(store: &SqliteStore) {
    let session = store
        .start_collection(
            &CollectionSpec::Dataset {
                dataset_id: "test".to_owned(),
            },
            Some(1),
        )
        .unwrap();
    let question = store
        .next_question(&session.collection_id)
        .unwrap()
        .unwrap();
    let answer = question
        .options
        .iter()
        .find(|option| option.sense_uid == question.sense_uid)
        .unwrap();
    store
        .submit_answer(
            &session.collection_id,
            &question.question_id,
            &answer.option_id,
            Some(900),
        )
        .unwrap();
}

fn dataset_names(store: &SqliteStore) -> Vec<String> {
    let mut names: Vec<String> = store
        .list_datasets()
        .unwrap()
        .into_iter()
        .filter(|dataset| !dataset.preloaded)
        .map(|dataset| dataset.name)
        .collect();
    names.sort();
    names
}

fn article_titles(store: &SqliteStore) -> Vec<String> {
    let mut titles: Vec<String> = store
        .list_articles()
        .unwrap()
        .into_iter()
        .map(|article| article.title)
        .collect();
    titles.sort();
    titles
}

struct Fixture {
    directory: TempDir,
}

impl Fixture {
    fn new() -> Self {
        Self {
            directory: tempfile::tempdir().unwrap(),
        }
    }

    fn store(&self, device: &str) -> SqliteStore {
        let directory = self.directory.path().join(device);
        std::fs::create_dir_all(&directory).unwrap();
        let content = directory.join("content.db");
        create_content(&content);
        SqliteStore::open(&DatabasePaths {
            content,
            user: directory.join("user.db"),
        })
        .unwrap()
    }

    fn sync_path(&self, name: &str) -> PathBuf {
        self.directory
            .path()
            .join(format!("{name}.polarbear-vocab-sync"))
    }
}

fn create_content(path: &Path) {
    let connection = Connection::open(path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE source(id INTEGER PRIMARY KEY, name TEXT, license TEXT, url TEXT, attribution TEXT);
             CREATE TABLE dataset(id TEXT PRIMARY KEY, name TEXT, created_at INTEGER, preloaded INTEGER);
             CREATE TABLE word(id INTEGER PRIMARY KEY, uid TEXT UNIQUE, lemma TEXT, frequency_rank INTEGER);
             CREATE TABLE sense(id INTEGER PRIMARY KEY, uid TEXT UNIQUE, word_id INTEGER, source_id INTEGER, pos TEXT, quiz_prompt_zh TEXT, zh_gloss TEXT, en_definition TEXT, cefr TEXT);
             CREATE TABLE pronunciation(id INTEGER PRIMARY KEY, sense_id INTEGER, accent TEXT, ipa TEXT, UNIQUE(sense_id, accent));
             CREATE TABLE example(id INTEGER PRIMARY KEY, sense_id INTEGER, sentence_en TEXT, sentence_zh TEXT, is_primary INTEGER);
             CREATE TABLE dataset_item(dataset_id TEXT, sense_uid TEXT, sequence INTEGER, PRIMARY KEY(dataset_id, sense_uid));
             CREATE TABLE distractor_edge(prompt_sense_uid TEXT, candidate_sense_uid TEXT, score REAL, reason TEXT);
             INSERT INTO source VALUES (1, 'Fixture', 'CC0-1.0', NULL, 'Fixture');
             INSERT INTO dataset VALUES ('test', 'Test', 0, 1);",
        )
        .unwrap();
    for index in 1..=5 {
        let uid = format!("word{index}.n.01");
        connection
            .execute(
                "INSERT INTO word VALUES (?1, ?2, ?3, ?1)",
                params![index, format!("word{index}.word"), format!("word{index}")],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO sense VALUES (?1, ?2, ?1, 1, 'noun', ?3, ?3, '', 'A1')",
                params![index, uid, format!("提示{index}")],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO dataset_item VALUES ('test', ?1, ?2)",
                params![uid, index],
            )
            .unwrap();
    }
    for prompt in 1..=5 {
        for offset in 1..=3 {
            let candidate = (prompt + offset - 1) % 5 + 1;
            connection
                .execute(
                    "INSERT INTO distractor_edge VALUES (?1, ?2, 1.0, 'fixture')",
                    params![
                        format!("word{prompt}.n.01"),
                        format!("word{candidate}.n.01")
                    ],
                )
                .unwrap();
        }
    }
}

fn path(value: &Path) -> &str {
    value.to_str().unwrap()
}
