use polarbear_vocab_application::{DatasetRepository, HomeQueryPort};
use polarbear_vocab_domain::{DatasetImportPlan, ImportedSense};
use polarbear_vocab_storage_sqlite::{DatabasePaths, SqliteStore};
use rusqlite::{Connection, params};
use tempfile::TempDir;

#[test]
fn dataset_crud_and_import_are_persisted() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let created = store.create_dataset("Travel").unwrap();
    let plan = DatasetImportPlan {
        entries: vec![imported_sense("travel.v.01", "travel", "旅行")],
    };

    let first = store.import_dataset(&created.id, &plan).unwrap();
    let second = store.import_dataset(&created.id, &plan).unwrap();
    store.rename_dataset(&created.id, "Travel English").unwrap();

    assert_eq!(first.inserted_senses, 1);
    assert_eq!(second.updated_senses, 1);
    let dataset = store
        .list_datasets()
        .unwrap()
        .into_iter()
        .find(|dataset| dataset.id == created.id)
        .unwrap();
    assert_eq!(dataset.name, "Travel English");
    assert_eq!(dataset.word_count, 1);
    store.delete_dataset(&created.id).unwrap();
    assert!(
        store
            .list_datasets()
            .unwrap()
            .iter()
            .all(|dataset| dataset.id != created.id)
    );
}

#[test]
fn missing_dataset_rolls_back_the_complete_import() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let plan = DatasetImportPlan {
        entries: vec![imported_sense("rollback.v.01", "rollback", "回滚")],
    };

    assert!(store.import_dataset("missing", &plan).is_err());

    let connection = Connection::open(fixture.content_path()).unwrap();
    let inserted: u32 = connection
        .query_row(
            "SELECT COUNT(*) FROM sense WHERE uid = 'rollback.v.01'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(inserted, 0);
}

fn imported_sense(uid: &str, lemma: &str, gloss: &str) -> ImportedSense {
    ImportedSense {
        sense_uid: uid.to_owned(),
        lemma: lemma.to_owned(),
        part_of_speech: "verb".to_owned(),
        quiz_prompt_zh: gloss.to_owned(),
        gloss_zh: gloss.to_owned(),
        ipa_us: String::new(),
        ipa_uk: String::new(),
        example_en: format!("We {lemma} today."),
        example_zh: "示例。".to_owned(),
    }
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

    fn content_path(&self) -> std::path::PathBuf {
        self.directory.path().join("content.db")
    }

    fn store(&self) -> SqliteStore {
        SqliteStore::open(&DatabasePaths {
            content: self.content_path(),
            user: self.directory.path().join("user.db"),
        })
        .unwrap()
    }
}

fn create_content(path: &std::path::Path) {
    let connection = Connection::open(path).unwrap();
    connection.execute_batch(
        "CREATE TABLE source(id INTEGER PRIMARY KEY, name TEXT, license TEXT, url TEXT, attribution TEXT);
         CREATE TABLE dataset(id TEXT PRIMARY KEY, name TEXT, created_at INTEGER, preloaded INTEGER);
         CREATE TABLE word(id INTEGER PRIMARY KEY, uid TEXT UNIQUE, lemma TEXT, frequency_rank INTEGER);
         CREATE TABLE sense(id INTEGER PRIMARY KEY, uid TEXT UNIQUE, word_id INTEGER, source_id INTEGER, pos TEXT, quiz_prompt_zh TEXT, zh_gloss TEXT, en_definition TEXT, cefr TEXT);
         CREATE TABLE pronunciation(id INTEGER PRIMARY KEY, sense_id INTEGER, accent TEXT, ipa TEXT, UNIQUE(sense_id, accent));
         CREATE TABLE example(id INTEGER PRIMARY KEY, sense_id INTEGER, sentence_en TEXT, sentence_zh TEXT, is_primary INTEGER);
         CREATE TABLE dataset_item(dataset_id TEXT, sense_uid TEXT, sequence INTEGER, PRIMARY KEY(dataset_id, sense_uid));
         CREATE TABLE distractor_edge(prompt_sense_uid TEXT, candidate_sense_uid TEXT, score REAL, reason TEXT, PRIMARY KEY(prompt_sense_uid, candidate_sense_uid));
         INSERT INTO source VALUES (1, 'Fixture', 'CC0-1.0', NULL, 'Fixture');",
    ).unwrap();
    for index in 1..=4 {
        connection
            .execute(
                "INSERT INTO word VALUES (?1, ?2, ?3, ?1)",
                params![index, format!("base{index}.word"), format!("base{index}")],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO sense VALUES (?1, ?2, ?1, 1, 'noun', ?3, ?3, '', 'A1')",
                params![index, format!("base{index}.n.01"), format!("基础{index}")],
            )
            .unwrap();
    }
}
