use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

pub fn initialize_user_schema(connection: &mut Connection) -> rusqlite::Result<()> {
    let previous_version = user_schema_version(connection)?;
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        INSERT OR IGNORE INTO schema_meta(key, value) VALUES ('version', '7');

        CREATE TABLE IF NOT EXISTS article (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            translated_body TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS my_vocabulary (
            sense_uid TEXT PRIMARY KEY,
            added_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS word_stat (
            sense_uid TEXT PRIMARY KEY,
            attempt_count INTEGER NOT NULL DEFAULT 0 CHECK(attempt_count >= 0),
            correct_count INTEGER NOT NULL DEFAULT 0 CHECK(correct_count >= 0),
            wrong_count INTEGER NOT NULL DEFAULT 0 CHECK(wrong_count >= 0),
            last_result TEXT CHECK(last_result IN ('correct', 'wrong')),
            first_answered_at INTEGER,
            last_answered_at INTEGER,
            last_correct_at INTEGER,
            last_wrong_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS study_session (
            id TEXT PRIMARY KEY,
            dataset_id TEXT,
            collection_type TEXT NOT NULL,
            collection_spec_json TEXT NOT NULL,
            started_at INTEGER NOT NULL,
            ended_at INTEGER,
            attempt_count INTEGER NOT NULL DEFAULT 0,
            correct_count INTEGER NOT NULL DEFAULT 0,
            wrong_count INTEGER NOT NULL DEFAULT 0,
            new_word_count INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS session_item (
            session_id TEXT NOT NULL REFERENCES study_session(id) ON DELETE CASCADE,
            ordinal INTEGER NOT NULL,
            sense_uid TEXT NOT NULL,
            answered INTEGER NOT NULL DEFAULT 0 CHECK(answered IN (0, 1)),
            PRIMARY KEY(session_id, ordinal),
            UNIQUE(session_id, sense_uid)
        );

        CREATE TABLE IF NOT EXISTS review_event (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES study_session(id),
            dataset_id TEXT,
            sense_uid TEXT NOT NULL,
            collection_type TEXT NOT NULL,
            answered_at INTEGER NOT NULL,
            correct INTEGER NOT NULL CHECK(correct IN (0, 1)),
            selected_sense_uid TEXT,
            latency_ms INTEGER,
            options_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_review_event_time ON review_event(answered_at);
        CREATE INDEX IF NOT EXISTS idx_review_event_sense
            ON review_event(sense_uid, answered_at DESC);

        CREATE TABLE IF NOT EXISTS daily_stat (
            local_date TEXT PRIMARY KEY,
            attempt_count INTEGER NOT NULL DEFAULT 0,
            correct_count INTEGER NOT NULL DEFAULT 0,
            wrong_count INTEGER NOT NULL DEFAULT 0,
            unique_word_count INTEGER NOT NULL DEFAULT 0,
            last_activity_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS setting (
            key TEXT PRIMARY KEY,
            value_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sync_change (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_kind TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            operation TEXT NOT NULL CHECK(operation IN ('upsert', 'delete')),
            changed_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_sync_change_entity
            ON sync_change(entity_kind, entity_id, seq DESC);

        CREATE TABLE IF NOT EXISTS sync_peer (
            device_id TEXT PRIMARY KEY,
            received_content_cursor INTEGER NOT NULL DEFAULT 0,
            received_user_cursor INTEGER NOT NULL DEFAULT 0,
            acknowledged_content_cursor INTEGER NOT NULL DEFAULT 0,
            acknowledged_user_cursor INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sync_package (
            package_id TEXT PRIMARY KEY,
            source_device_id TEXT NOT NULL,
            imported_at INTEGER NOT NULL
        );",
    )?;
    migrate_v1_dataset_columns(connection)?;
    if !has_column(connection, LegacyTable::Article, "translated_body")? {
        connection.execute("ALTER TABLE article ADD COLUMN translated_body TEXT", [])?;
    }
    if !has_column(connection, LegacyTable::Article, "updated_at")? {
        connection.execute(
            "ALTER TABLE article ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
        connection.execute("UPDATE article SET updated_at = created_at", [])?;
    }
    if !has_column(connection, LegacyTable::StudySession, "new_word_count")? {
        connection.execute(
            "ALTER TABLE study_session ADD COLUMN new_word_count INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
        connection.execute(
            "UPDATE study_session SET new_word_count = (
                SELECT COUNT(*) FROM review_event event
                JOIN word_stat stat ON stat.sense_uid = event.sense_uid
                WHERE event.session_id = study_session.id
                  AND stat.first_answered_at = event.answered_at
             )",
            [],
        )?;
    }
    if previous_version.unwrap_or_default() < 7 {
        bootstrap_sync_changes(connection)?;
    }
    connection.execute(
        "INSERT INTO schema_meta(key, value) VALUES ('version', '7')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [],
    )?;
    Ok(())
}

pub(crate) fn record_sync_change(
    transaction: &Transaction<'_>,
    entity_kind: &str,
    entity_id: &str,
    operation: &str,
    changed_at: i64,
) -> rusqlite::Result<i64> {
    transaction.execute(
        "INSERT INTO sync_change(entity_kind, entity_id, operation, changed_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![entity_kind, entity_id, operation, changed_at],
    )?;
    Ok(transaction.last_insert_rowid())
}

fn user_schema_version(connection: &Connection) -> rusqlite::Result<Option<u32>> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_meta')",
        [],
        |row| row.get(0),
    )?;
    if !exists {
        return Ok(None);
    }
    connection
        .query_row(
            "SELECT value FROM schema_meta WHERE key = 'version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map(|value| value.and_then(|version| version.parse().ok()))
}

fn bootstrap_sync_changes(connection: &mut Connection) -> rusqlite::Result<()> {
    let review_has_id = has_column(connection, LegacyTable::ReviewEvent, "id")?;
    in_immediate_transaction(connection, |transaction| {
        transaction.execute_batch(
            "INSERT INTO sync_change(entity_kind, entity_id, operation, changed_at)
               SELECT 'article', id, 'upsert', updated_at FROM article;
             INSERT INTO sync_change(entity_kind, entity_id, operation, changed_at)
               SELECT 'vocabulary', sense_uid, 'upsert', added_at FROM my_vocabulary;",
        )?;
        if review_has_id {
            transaction.execute(
                "INSERT INTO sync_change(entity_kind, entity_id, operation, changed_at)
                 SELECT 'reviewEvent', id, 'upsert', answered_at FROM review_event",
                [],
            )?;
        }
        Ok(())
    })
}

fn migrate_v1_dataset_columns(connection: &mut Connection) -> rusqlite::Result<()> {
    let session_legacy = has_column(connection, LegacyTable::StudySession, "dataset_uid")?;
    let event_legacy = has_column(connection, LegacyTable::ReviewEvent, "dataset_uid")?;
    if !session_legacy && !event_legacy {
        return Ok(());
    }
    in_immediate_transaction(connection, |transaction| {
        if session_legacy {
            transaction.execute(
                "ALTER TABLE study_session RENAME COLUMN dataset_uid TO dataset_id",
                [],
            )?;
        }
        if event_legacy {
            transaction.execute(
                "ALTER TABLE review_event RENAME COLUMN dataset_uid TO dataset_id",
                [],
            )?;
        }
        Ok(())
    })
}

#[derive(Clone, Copy)]
enum LegacyTable {
    Article,
    StudySession,
    ReviewEvent,
}

fn has_column(connection: &Connection, table: LegacyTable, column: &str) -> rusqlite::Result<bool> {
    let query = match table {
        LegacyTable::Article => "PRAGMA table_info(article)",
        LegacyTable::StudySession => "PRAGMA table_info(study_session)",
        LegacyTable::ReviewEvent => "PRAGMA table_info(review_event)",
    };
    let mut statement = connection.prepare(query)?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    for candidate in columns {
        if candidate? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn in_immediate_transaction<T>(
    connection: &mut Connection,
    action: impl FnOnce(&Transaction<'_>) -> rusqlite::Result<T>,
) -> rusqlite::Result<T> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    match action(&transaction) {
        Ok(value) => {
            transaction.commit()?;
            Ok(value)
        }
        Err(error) => {
            transaction.rollback()?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{in_immediate_transaction, initialize_user_schema};

    #[test]
    fn immediate_transaction_commits_successful_changes() {
        let mut connection = fixture();

        in_immediate_transaction(&mut connection, |transaction| {
            transaction.execute("INSERT INTO value(id) VALUES (1)", [])?;
            Ok(())
        })
        .expect("transaction should commit");

        assert_eq!(count(&connection), 1);
    }

    #[test]
    fn immediate_transaction_rolls_back_failed_changes() {
        let mut connection = fixture();

        let result = in_immediate_transaction(&mut connection, |transaction| {
            transaction.execute("INSERT INTO value(id) VALUES (1)", [])?;
            Err::<(), _>(rusqlite::Error::InvalidQuery)
        });

        assert!(result.is_err());
        assert_eq!(count(&connection), 0);
    }

    #[test]
    fn version_one_dataset_uid_columns_are_migrated() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE study_session(
                    id TEXT PRIMARY KEY, dataset_uid TEXT, collection_type TEXT NOT NULL,
                    collection_spec_json TEXT NOT NULL, started_at INTEGER NOT NULL,
                    ended_at INTEGER, attempt_count INTEGER NOT NULL DEFAULT 0,
                    correct_count INTEGER NOT NULL DEFAULT 0, wrong_count INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE TABLE review_event(
                    id TEXT PRIMARY KEY, session_id TEXT NOT NULL, dataset_uid TEXT,
                    sense_uid TEXT NOT NULL, collection_type TEXT NOT NULL,
                    answered_at INTEGER NOT NULL, correct INTEGER NOT NULL,
                    selected_sense_uid TEXT, latency_ms INTEGER, options_json TEXT NOT NULL
                 );",
            )
            .unwrap();

        initialize_user_schema(&mut connection).unwrap();

        assert!(has_named_column(&connection, "study_session", "dataset_id"));
        assert!(has_named_column(&connection, "review_event", "dataset_id"));
        assert!(!has_named_column(
            &connection,
            "study_session",
            "dataset_uid"
        ));
    }

    #[test]
    fn version_four_session_new_word_count_is_recovered_from_history() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE study_session(id TEXT PRIMARY KEY);
             INSERT INTO study_session(id) VALUES ('old-session');
             CREATE TABLE review_event(session_id TEXT, sense_uid TEXT, answered_at INTEGER,
                 correct INTEGER);
             INSERT INTO review_event VALUES ('old-session', 'earn.v.01', 42, 1);
             CREATE TABLE word_stat(sense_uid TEXT PRIMARY KEY, first_answered_at INTEGER);
             INSERT INTO word_stat VALUES ('earn.v.01', 42);",
            )
            .unwrap();

        initialize_user_schema(&mut connection).unwrap();
        let count: u32 = connection
            .query_row(
                "SELECT new_word_count FROM study_session WHERE id = 'old-session'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    fn fixture() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory database should open");
        connection
            .execute("CREATE TABLE value(id INTEGER PRIMARY KEY)", [])
            .expect("fixture schema should be valid");
        connection
    }

    fn count(connection: &Connection) -> u32 {
        connection
            .query_row("SELECT COUNT(*) FROM value", [], |row| row.get(0))
            .expect("fixture query should work")
    }

    fn has_named_column(connection: &Connection, table: &str, expected: &str) -> bool {
        let mut statement = connection
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .any(|column| column.unwrap() == expected)
    }
}
