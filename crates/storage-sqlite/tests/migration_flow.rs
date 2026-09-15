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
    assert_eq!(version, "3");
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
