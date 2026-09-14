use polarbear_vocab_application::{HomeQueryPort, MistakeQueryPort, SettingsPort, StudyPort};
use polarbear_vocab_domain::{CollectionSpec, SettingsDto};
use polarbear_vocab_storage_sqlite::{DatabasePaths, SqliteStore};
use rusqlite::{Connection, params};
use tempfile::TempDir;

#[test]
fn answer_updates_history_aggregates_and_dynamic_collections() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let session = store
        .start_collection(&CollectionSpec::Unseen {
            dataset_id: "test".to_owned(),
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
            dataset_id: Some("test".to_owned()),
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
            dataset_id: "test".to_owned(),
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

#[test]
fn stale_question_cannot_create_a_duplicate_history_event() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let session = store
        .start_collection(&CollectionSpec::Dataset {
            dataset_id: "test".to_owned(),
        })
        .unwrap();
    let question = store
        .next_question(&session.collection_id)
        .unwrap()
        .unwrap();
    let correct_option = question
        .options
        .iter()
        .find(|option| option.sense_uid == question.sense_uid)
        .unwrap();
    store
        .submit_answer(
            &session.collection_id,
            &question.question_id,
            &correct_option.option_id,
            None,
        )
        .unwrap();

    assert!(
        store
            .submit_answer(
                &session.collection_id,
                &question.question_id,
                &correct_option.option_id,
                None,
            )
            .is_err()
    );
    let home = store.get_home("test").unwrap();
    assert_eq!(home.daily_activity.last().unwrap().attempt_count, 1);
}

#[test]
fn settings_round_trip_all_preferences_together() {
    let fixture = Fixture::new();
    let store = fixture.store();
    assert_eq!(store.get_settings().unwrap(), SettingsDto::default());
    let settings = SettingsDto {
        speech_locale: "en-GB".to_owned(),
        speech_rate_percent: 125,
        ui_language: "zh-CN".to_owned(),
        ui_theme: "dark".to_owned(),
    };

    store.update_settings(&settings).unwrap();

    assert_eq!(store.get_settings().unwrap(), settings);
}

#[test]
fn sqlite_store_exposes_the_lexicon_read_model() {
    let fixture = Fixture::new();
    let store = fixture.store();

    let sense = store.sense("word1.n.01").unwrap().unwrap();

    assert_eq!(sense.lemma, "word1");
    assert_eq!(sense.part_of_speech, "noun");
    assert_eq!(sense.quiz_prompt_zh, "提示1");
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
                "INSERT INTO sense VALUES (?1, ?2, ?1, 1, 'noun', ?3, ?4, '', 'A1')",
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
            .execute(
                "INSERT INTO dataset_item VALUES ('test', ?1, ?2)",
                params![uid, id],
            )
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
use polarbear_lexicon::LexiconReader;
