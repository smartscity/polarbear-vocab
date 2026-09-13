use polarbear_vocab_application::{HomeQueryPort, MistakeQueryPort, StudyPort};
use polarbear_vocab_domain::CollectionSpec;
use polarbear_vocab_storage_sqlite::{DatabasePaths, SqliteStore};
use rusqlite::{Connection, params};
use tempfile::TempDir;

#[test]
fn answer_updates_history_aggregates_and_dynamic_collections() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let session = store
        .start_collection(&CollectionSpec::Unseen {
            dataset_uid: "test".to_owned(),
        })
        .expect("unseen collection should start");
    let question = store
        .next_question(&session.collection_id)
        .expect("question lookup should succeed")
        .expect("collection should contain a question");
    let wrong_option = question
        .options
        .iter()
        .find(|option| option.sense_uid != question.sense_uid)
        .expect("question should contain distractors");

    let answer = store
        .submit_answer(
            &session.collection_id,
            &question.question_id,
            &wrong_option.option_id,
            Some(250),
        )
        .expect("answer should be recorded");

    assert!(!answer.correct);
    let home = store.get_home("test").expect("home query should work");
    assert_eq!(home.totals.explored, 1);
    assert_eq!(home.totals.mistakes, 1);
    assert_eq!(home.daily_activity.last().unwrap().attempt_count, 1);
    let wrong = store
        .start_collection(&CollectionSpec::Wrong {
            dataset_uid: Some("test".to_owned()),
            min_wrong_count: 1,
        })
        .expect("mistake collection should start");
    assert_eq!(wrong.total_count, 1);
    assert_eq!(
        store
            .list_wrong_words(Some("test"), 1, true)
            .expect("mistake list should load")
            .len(),
        1
    );
}

#[test]
fn invalid_option_does_not_write_answer_history() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let session = store
        .start_collection(&CollectionSpec::Dataset {
            dataset_uid: "test".to_owned(),
        })
        .expect("dataset collection should start");
    let question = store
        .next_question(&session.collection_id)
        .unwrap()
        .unwrap();

    assert!(
        store
            .submit_answer(
                &session.collection_id,
                &question.question_id,
                "not-an-option",
                None,
            )
            .is_err()
    );
    let home = store.get_home("test").expect("home query should work");
    assert_eq!(home.totals.explored, 0);
    assert_eq!(home.daily_activity.last().unwrap().attempt_count, 0);
}

struct Fixture {
    directory: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("temporary directory should be available");
        create_content(&directory.path().join("content.db"));
        Self { directory }
    }

    fn store(&self) -> SqliteStore {
        SqliteStore::open(&DatabasePaths {
            content: self.directory.path().join("content.db"),
            user: self.directory.path().join("user.db"),
        })
        .expect("fixture store should open")
    }
}

fn create_content(path: &std::path::Path) {
    let connection = Connection::open(path).expect("content fixture should open");
    connection
        .execute_batch(
            "CREATE TABLE dataset(id INTEGER PRIMARY KEY, uid TEXT, name TEXT, category TEXT, description TEXT, sort_order INTEGER, active INTEGER);
             CREATE TABLE word(id INTEGER PRIMARY KEY, uid TEXT, lemma TEXT, frequency_rank INTEGER);
             CREATE TABLE sense(id INTEGER PRIMARY KEY, uid TEXT, word_id INTEGER, quiz_prompt_zh TEXT, zh_gloss TEXT);
             CREATE TABLE pronunciation(id INTEGER PRIMARY KEY, sense_id INTEGER, accent TEXT, ipa TEXT);
             CREATE TABLE example(id INTEGER PRIMARY KEY, sense_id INTEGER, sentence_en TEXT, sentence_zh TEXT, is_primary INTEGER);
             CREATE TABLE dataset_item(dataset_id INTEGER, sense_id INTEGER, sequence INTEGER, importance INTEGER);
             CREATE TABLE distractor_edge(prompt_sense_uid TEXT, candidate_sense_uid TEXT, score REAL, reason TEXT);
             INSERT INTO dataset VALUES (1, 'test', 'Test', 'Tests', 'Fixture', 1, 1);",
        )
        .expect("fixture schema should be valid");
    for index in 0..5 {
        let id = index + 1;
        let uid = format!("word{id}.n.01");
        let lemma = format!("word{id}");
        connection
            .execute(
                "INSERT INTO word VALUES (?1, ?2, ?3, ?1)",
                params![id, format!("{lemma}.word"), lemma],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO sense VALUES (?1, ?2, ?1, ?3, ?4)",
                params![id, uid, format!("提示{id}"), format!("释义{id}")],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO pronunciation VALUES (?1, ?1, 'en-US', '/test/')",
                [id],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO example VALUES (?1, ?1, ?2, ?3, 1)",
                params![id, format!("Example {id}."), format!("例句{id}。")],
            )
            .unwrap();
        connection
            .execute("INSERT INTO dataset_item VALUES (1, ?1, ?1, 0)", [id])
            .unwrap();
    }
    for prompt in 1..=5 {
        for offset in 1..=3 {
            let candidate = (prompt + offset - 1) % 5 + 1;
            connection
                .execute(
                    "INSERT INTO distractor_edge VALUES (?1, ?2, ?3, 'fixture')",
                    params![
                        format!("word{prompt}.n.01"),
                        format!("word{candidate}.n.01"),
                        1.0 - f64::from(offset) / 10.0
                    ],
                )
                .unwrap();
        }
    }
}
