use polarbear_vocab_application::LexiconRepository;
use polarbear_vocab_storage_sqlite::{DatabasePaths, SqliteStore, ensure_writable_content};
use rusqlite::Connection;

#[test]
fn version_two_content_gains_updated_at_without_losing_existing_rows() {
    let directory = tempfile::tempdir().unwrap();
    let content = directory.path().join("content.db");
    let user = directory.path().join("user.db");
    let connection = Connection::open(&content).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE schema_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO schema_meta VALUES ('version', '2');
         CREATE TABLE dataset(id TEXT PRIMARY KEY, name TEXT NOT NULL,
             created_at INTEGER NOT NULL, preloaded INTEGER NOT NULL);
         INSERT INTO dataset VALUES ('legacy', 'Legacy', 42, 0);",
        )
        .unwrap();
    drop(connection);

    SqliteStore::open(&DatabasePaths {
        content: content.clone(),
        user,
    })
    .unwrap();
    let connection = Connection::open(content).unwrap();
    let updated_at: i64 = connection
        .query_row(
            "SELECT updated_at FROM dataset WHERE id = 'legacy'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let version: String = connection
        .query_row(
            "SELECT value FROM schema_meta WHERE key = 'version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(updated_at, 42);
    assert_eq!(version, "4");
}

#[test]
fn unsupported_existing_content_is_never_overwritten_by_a_seed() {
    let directory = tempfile::tempdir().unwrap();
    let content = directory.path().join("content.db");
    let seed = directory.path().join("seed.db");
    let connection = Connection::open(&content).unwrap();
    connection.execute_batch(
        "CREATE TABLE schema_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO schema_meta VALUES ('version', '99');
         CREATE TABLE dataset(id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL);",
    ).unwrap();
    drop(connection);
    std::fs::write(&seed, b"replacement").unwrap();

    assert!(ensure_writable_content(&seed, &content).is_err());
    assert!(
        SqliteStore::open(&DatabasePaths {
            content: content.clone(),
            user: directory.path().join("user.db"),
        })
        .is_err()
    );
    let connection = Connection::open(content).unwrap();
    let version: String = connection
        .query_row(
            "SELECT value FROM schema_meta WHERE key = 'version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, "99");
}

fn create_seed_upgrade_schema(connection: &Connection) {
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE schema_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('version', '3');
             CREATE TABLE source(id INTEGER PRIMARY KEY, name TEXT NOT NULL,
               license TEXT NOT NULL, url TEXT, attribution TEXT NOT NULL);
             CREATE TABLE dataset(id TEXT PRIMARY KEY, name TEXT NOT NULL,
               created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
               preloaded INTEGER NOT NULL);
             CREATE TABLE word(id INTEGER PRIMARY KEY, uid TEXT NOT NULL UNIQUE,
               lemma TEXT NOT NULL, frequency_rank INTEGER);
             CREATE TABLE sense(id INTEGER PRIMARY KEY, uid TEXT NOT NULL UNIQUE,
               word_id INTEGER NOT NULL REFERENCES word(id),
               source_id INTEGER NOT NULL REFERENCES source(id), pos TEXT NOT NULL,
               quiz_prompt_zh TEXT NOT NULL, zh_gloss TEXT NOT NULL,
               en_definition TEXT NOT NULL, cefr TEXT NOT NULL);
             CREATE TABLE pronunciation(id INTEGER PRIMARY KEY,
               sense_id INTEGER NOT NULL REFERENCES sense(id), accent TEXT NOT NULL,
               ipa TEXT NOT NULL, UNIQUE(sense_id, accent));
             CREATE TABLE example(id INTEGER PRIMARY KEY,
               sense_id INTEGER NOT NULL REFERENCES sense(id), sentence_en TEXT NOT NULL,
               sentence_zh TEXT NOT NULL, is_primary INTEGER NOT NULL);
             CREATE TABLE dataset_item(dataset_id TEXT NOT NULL REFERENCES dataset(id),
               sense_uid TEXT NOT NULL REFERENCES sense(uid), sequence INTEGER,
               PRIMARY KEY(dataset_id, sense_uid));
             CREATE TABLE distractor_edge(prompt_sense_uid TEXT NOT NULL,
               candidate_sense_uid TEXT NOT NULL, score REAL NOT NULL, reason TEXT,
               PRIMARY KEY(prompt_sense_uid, candidate_sense_uid));",
        )
        .unwrap();
}

#[test]
fn new_seed_expands_existing_preloads_without_erasing_custom_data() {
    let directory = tempfile::tempdir().unwrap();
    let content_path = directory.path().join("content.db");
    let seed_path = directory.path().join("seed.db");
    let content = Connection::open(&content_path).unwrap();
    create_seed_upgrade_schema(&content);
    content
        .execute_batch(
            "INSERT INTO source(name, license, attribution) VALUES ('starter', 'CC0', 'test');
         INSERT INTO source(name, license, attribution) VALUES ('user', 'private', 'test');
         INSERT INTO dataset VALUES ('cet4', 'CET-4', 0, 0, 1);
         INSERT INTO dataset VALUES ('my-words', 'My Words', 0, 0, 0);
         INSERT INTO word(uid, lemma) VALUES ('earn.word', 'earn');
         INSERT INTO word(uid, lemma) VALUES ('private.word', 'private');
         INSERT INTO sense(uid, word_id, source_id, pos, quiz_prompt_zh,
           zh_gloss, en_definition, cefr)
           VALUES ('earn.v.01', 1, 1, 'verb', '赚得', '赚得', '', ''),
                  ('private.a.01', 2, 2, 'adjective', '私有', '用户释义', '', '');
         INSERT INTO dataset_item VALUES ('cet4', 'earn.v.01', 99);
         INSERT INTO dataset_item VALUES ('my-words', 'private.a.01', 0);",
        )
        .unwrap();
    drop(content);

    let seed = Connection::open(&seed_path).unwrap();
    create_seed_upgrade_schema(&seed);
    seed.execute_batch(
        "INSERT INTO source(name, license, attribution) VALUES ('starter', 'CC0', 'test');
         INSERT INTO source(name, license, attribution) VALUES ('full', 'local', 'test');
         INSERT INTO dataset VALUES ('cet4', 'CET-4', 0, 0, 1);
         INSERT INTO dataset VALUES ('cet6', 'CET-6', 0, 0, 1);
         INSERT INTO word(uid, lemma) VALUES ('earn.word', 'earn');
         INSERT INTO word(uid, lemma) VALUES ('abundant.word', 'abundant');
         INSERT INTO sense(uid, word_id, source_id, pos, quiz_prompt_zh,
           zh_gloss, en_definition, cefr)
           VALUES ('earn.v.01', 1, 1, 'verb', '赚得', '赚得', '', ''),
                  ('abundant.a.01', 2, 2, 'adjective', '丰富', '丰富', '', '');
         INSERT INTO pronunciation(sense_id, accent, ipa)
           VALUES (2, 'en-US', '/əˈbʌndənt/');
         INSERT INTO example(sense_id, sentence_en, sentence_zh, is_primary)
           VALUES (2, 'Food was abundant.', '食物充足。', 1);
         INSERT INTO dataset_item VALUES ('cet4', 'earn.v.01', 0),
           ('cet4', 'abundant.a.01', 1), ('cet6', 'earn.v.01', 0),
           ('cet6', 'abundant.a.01', 1);
         INSERT INTO distractor_edge VALUES ('abundant.a.01', 'earn.v.01', 0.7, 'test');",
    )
    .unwrap();
    drop(seed);

    ensure_writable_content(&seed_path, &content_path).unwrap();
    ensure_writable_content(&seed_path, &content_path).unwrap();
    let content = Connection::open(&content_path).unwrap();
    let counts: (i64, i64, i64, i64) = content
        .query_row(
            "SELECT (SELECT COUNT(*) FROM sense), (SELECT COUNT(*) FROM dataset_item),
         (SELECT COUNT(*) FROM example), (SELECT COUNT(*) FROM source)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(counts, (3, 5, 1, 3));
    let custom_gloss: String = content
        .query_row(
            "SELECT zh_gloss FROM sense WHERE uid = 'private.a.01'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(custom_gloss, "用户释义");
    let seed_sequence: i64 = content
        .query_row(
            "SELECT sequence FROM dataset_item WHERE dataset_id = 'cet4' AND sense_uid = 'earn.v.01'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(seed_sequence, 0);
    drop(content);

    let store = SqliteStore::open(&DatabasePaths {
        content: content_path,
        user: directory.path().join("user.db"),
    })
    .unwrap();
    let matches = store.search_lexicon("abundant", 10).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].dataset_names, vec!["CET-4", "CET-6"]);
}

#[test]
fn invalid_bundled_seed_leaves_existing_content_untouched() {
    let directory = tempfile::tempdir().unwrap();
    let content_path = directory.path().join("content.db");
    let seed_path = directory.path().join("seed.db");
    let content = Connection::open(&content_path).unwrap();
    create_seed_upgrade_schema(&content);
    content
        .execute("INSERT INTO dataset VALUES ('mine', 'Mine', 0, 0, 0)", [])
        .unwrap();
    drop(content);
    let seed = Connection::open(&seed_path).unwrap();
    seed.execute_batch(
        "CREATE TABLE schema_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO schema_meta VALUES ('version', '99');",
    )
    .unwrap();
    drop(seed);
    assert!(ensure_writable_content(&seed_path, &content_path).is_err());
    let content = Connection::open(content_path).unwrap();
    let count: i64 = content
        .query_row("SELECT COUNT(*) FROM dataset", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
#[ignore = "requires a locally bundled full content.db"]
fn local_full_seed_merges_every_dataset_membership() {
    let seed_path = std::env::var("POLARBEAR_FULL_SEED_DB").expect("set local full seed path");
    let directory = tempfile::tempdir().unwrap();
    let content_path = directory.path().join("content.db");
    let content = Connection::open(&content_path).unwrap();
    create_seed_upgrade_schema(&content);
    drop(content);
    ensure_writable_content(std::path::Path::new(&seed_path), &content_path).unwrap();
    let merged = Connection::open(&content_path).unwrap();
    let seed = Connection::open(seed_path).unwrap();
    for table in ["sense", "dataset_item", "distractor_edge"] {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        let merged_count: i64 = merged.query_row(&sql, [], |row| row.get(0)).unwrap();
        let seed_count: i64 = seed.query_row(&sql, [], |row| row.get(0)).unwrap();
        assert_eq!(merged_count, seed_count, "{table} rows must all be merged");
    }
    let cet4_count: i64 = merged
        .query_row(
            "SELECT COUNT(*) FROM dataset_item WHERE dataset_id = 'cet4'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        cet4_count > 500,
        "CET-4 must contain the full supplied dataset"
    );
}
