use chrono::Utc;
use polarbear_vocab_application::ApplicationError;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::sync_models::{
    ApplyCounts, SyncArticle, SyncCursor, SyncManifest, SyncReviewEvent, SyncVocabulary,
    UserSyncPayload,
};
use crate::{database_error, schema, sync_peer};

pub fn export(
    connection: &Connection,
    base_cursor: i64,
) -> Result<(UserSyncPayload, i64), ApplicationError> {
    let payload = UserSyncPayload {
        articles: export_articles(connection, base_cursor)?,
        review_events: export_review_events(connection, base_cursor)?,
        vocabulary: export_vocabulary(connection, base_cursor)?,
    };
    let cursor = connection
        .query_row("SELECT COALESCE(MAX(seq), 0) FROM sync_change", [], |row| {
            row.get(0)
        })
        .map_err(database_error)?;
    Ok((payload, cursor))
}

pub fn apply(
    connection: &mut Connection,
    payload: &UserSyncPayload,
    manifest: &SyncManifest,
    acknowledged: SyncCursor,
) -> Result<ApplyCounts, ApplicationError> {
    schema::in_immediate_transaction(connection, |transaction| {
        let mut counts = ApplyCounts::default();
        for article in &payload.articles {
            if article_conflicts(transaction, article, acknowledged.user)? {
                apply_article_conflict(transaction, article, &manifest.source_device_id)?;
                counts.conflicts += 1;
            } else {
                apply_article(transaction, article)?;
            }
            counts.articles += 1;
        }
        for vocabulary in &payload.vocabulary {
            apply_vocabulary(transaction, vocabulary)?;
        }
        for event in &payload.review_events {
            counts.review_events += u32::from(apply_review_event(transaction, event)?);
        }
        if counts.review_events > 0 {
            rebuild_statistics(transaction)?;
        }
        sync_peer::finish_import(
            transaction,
            manifest,
            acknowledged,
            Utc::now().timestamp_millis(),
        )?;
        Ok(counts)
    })
    .map_err(database_error)
}

pub fn pending_count(connection: &Connection, base_cursor: i64) -> rusqlite::Result<u32> {
    connection.query_row(
        "SELECT COUNT(DISTINCT entity_kind || ':' || entity_id)
         FROM sync_change WHERE seq > ?1",
        [base_cursor],
        |row| row.get(0),
    )
}

fn export_articles(
    connection: &Connection,
    base_cursor: i64,
) -> Result<Vec<SyncArticle>, ApplicationError> {
    let changes = latest_changes(connection, "article", base_cursor)?;
    let mut articles = Vec::new();
    for (id, operation, revision, _) in changes {
        if operation == "delete" {
            articles.push(deleted_article(id, revision));
        } else {
            articles.push(load_article(connection, &id, revision)?.ok_or_else(|| {
                ApplicationError::Infrastructure(format!("changed article {id} is missing"))
            })?);
        }
    }
    Ok(articles)
}

fn export_review_events(
    connection: &Connection,
    base_cursor: i64,
) -> Result<Vec<SyncReviewEvent>, ApplicationError> {
    let mut statement = connection
        .prepare(
            "SELECT event.id, event.session_id, event.dataset_id, event.sense_uid,
               event.collection_type, event.answered_at, event.correct,
               event.selected_sense_uid, event.latency_ms, event.options_json, change.seq
             FROM sync_change change
             JOIN review_event event ON event.id = change.entity_id
             WHERE change.entity_kind = 'reviewEvent' AND change.seq > ?1
             ORDER BY change.seq",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([base_cursor], |row| {
            Ok(SyncReviewEvent {
                id: row.get(0)?,
                session_id: row.get(1)?,
                dataset_id: row.get(2)?,
                sense_uid: row.get(3)?,
                collection_type: row.get(4)?,
                answered_at: row.get(5)?,
                correct: row.get(6)?,
                selected_sense_uid: row.get(7)?,
                latency_ms: row.get(8)?,
                options_json: row.get(9)?,
                revision: row.get(10)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<rusqlite::Result<_>>()
        .map_err(database_error)
}

fn export_vocabulary(
    connection: &Connection,
    base_cursor: i64,
) -> Result<Vec<SyncVocabulary>, ApplicationError> {
    let mut vocabulary = Vec::new();
    for (sense_uid, operation, revision, changed_at) in
        latest_changes(connection, "vocabulary", base_cursor)?
    {
        let added_at = connection
            .query_row(
                "SELECT added_at FROM my_vocabulary WHERE sense_uid = ?1",
                [&sense_uid],
                |row| row.get(0),
            )
            .optional()
            .map_err(database_error)?
            .unwrap_or(changed_at);
        vocabulary.push(SyncVocabulary {
            sense_uid,
            added_at,
            changed_at,
            revision,
            deleted: operation == "delete",
        });
    }
    Ok(vocabulary)
}

fn latest_changes(
    connection: &Connection,
    kind: &str,
    base_cursor: i64,
) -> Result<Vec<(String, String, i64, i64)>, ApplicationError> {
    let mut statement = connection
        .prepare(
            "SELECT change.entity_id, change.operation, change.seq, change.changed_at
             FROM sync_change change
             JOIN (SELECT entity_id, MAX(seq) AS seq FROM sync_change
               WHERE entity_kind = ?1 AND seq > ?2 GROUP BY entity_id) latest
             ON latest.seq = change.seq ORDER BY change.seq",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![kind, base_cursor], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(database_error)?;
    rows.collect::<rusqlite::Result<_>>()
        .map_err(database_error)
}

fn load_article(
    connection: &Connection,
    id: &str,
    revision: i64,
) -> Result<Option<SyncArticle>, ApplicationError> {
    connection
        .query_row(
            "SELECT id, title, body, translated_body, created_at, updated_at
             FROM article WHERE id = ?1",
            [id],
            |row| {
                Ok(SyncArticle {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    body: row.get(2)?,
                    translated_body: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    revision,
                    deleted: false,
                })
            },
        )
        .optional()
        .map_err(database_error)
}

fn article_conflicts(
    transaction: &Transaction<'_>,
    incoming: &SyncArticle,
    acknowledged_cursor: i64,
) -> rusqlite::Result<bool> {
    let changed: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM sync_change
         WHERE entity_kind = 'article' AND entity_id = ?1 AND seq > ?2)",
        params![incoming.id, acknowledged_cursor],
        |row| row.get(0),
    )?;
    if !changed || incoming.deleted {
        return Ok(changed);
    }
    let current =
        load_article(transaction, &incoming.id, incoming.revision).map_err(to_sql_error)?;
    Ok(!current.is_some_and(|article| same_article(&article, incoming)))
}

fn same_article(left: &SyncArticle, right: &SyncArticle) -> bool {
    left.title == right.title
        && left.body == right.body
        && left.translated_body == right.translated_body
}

fn apply_article_conflict(
    transaction: &Transaction<'_>,
    incoming: &SyncArticle,
    source_device_id: &str,
) -> rusqlite::Result<()> {
    if incoming.deleted {
        return Ok(());
    }
    let mut conflict = incoming.clone();
    conflict.id = Uuid::new_v4().to_string();
    conflict.title = format!(
        "{} (Conflict from {})",
        incoming.title,
        short_id(source_device_id)
    );
    apply_article(transaction, &conflict)?;
    sync_peer::record_local_change(transaction, "article", &conflict.id, conflict.updated_at)
}

fn apply_article(transaction: &Transaction<'_>, article: &SyncArticle) -> rusqlite::Result<()> {
    if article.deleted {
        transaction.execute("DELETE FROM article WHERE id = ?1", [&article.id])?;
        return Ok(());
    }
    transaction.execute(
        "INSERT INTO article(id, title, body, translated_body, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET title = excluded.title, body = excluded.body,
           translated_body = excluded.translated_body, updated_at = excluded.updated_at",
        params![
            article.id,
            article.title,
            article.body,
            article.translated_body,
            article.created_at,
            article.updated_at
        ],
    )?;
    Ok(())
}

fn apply_vocabulary(
    transaction: &Transaction<'_>,
    vocabulary: &SyncVocabulary,
) -> rusqlite::Result<()> {
    if vocabulary.deleted {
        transaction.execute(
            "DELETE FROM my_vocabulary WHERE sense_uid = ?1",
            [&vocabulary.sense_uid],
        )?;
    } else {
        transaction.execute(
            "INSERT INTO my_vocabulary(sense_uid, added_at) VALUES (?1, ?2)
             ON CONFLICT(sense_uid) DO UPDATE SET added_at = MIN(added_at, excluded.added_at)",
            params![vocabulary.sense_uid, vocabulary.added_at],
        )?;
    }
    Ok(())
}

fn apply_review_event(
    transaction: &Transaction<'_>,
    event: &SyncReviewEvent,
) -> rusqlite::Result<bool> {
    transaction.execute(
        "INSERT OR IGNORE INTO study_session(
           id, dataset_id, collection_type, collection_spec_json, started_at, ended_at
         ) VALUES (?1, ?2, ?3, '{}', ?4, ?4)",
        params![
            event.session_id,
            event.dataset_id,
            event.collection_type,
            event.answered_at
        ],
    )?;
    let inserted = transaction.execute(
        "INSERT OR IGNORE INTO review_event(
           id, session_id, dataset_id, sense_uid, collection_type, answered_at,
           correct, selected_sense_uid, latency_ms, options_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            event.id,
            event.session_id,
            event.dataset_id,
            event.sense_uid,
            event.collection_type,
            event.answered_at,
            event.correct,
            event.selected_sense_uid,
            event.latency_ms,
            event.options_json
        ],
    )?;
    Ok(inserted > 0)
}

fn rebuild_statistics(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    transaction.execute_batch(
        "DELETE FROM word_stat;
         INSERT INTO word_stat(
           sense_uid, attempt_count, correct_count, wrong_count, last_result,
           first_answered_at, last_answered_at, last_correct_at, last_wrong_at
         ) SELECT event.sense_uid, COUNT(*), SUM(event.correct), SUM(NOT event.correct),
           (SELECT CASE WHEN latest.correct = 1 THEN 'correct' ELSE 'wrong' END
            FROM review_event latest WHERE latest.sense_uid = event.sense_uid
            ORDER BY latest.answered_at DESC, latest.id DESC LIMIT 1),
           MIN(event.answered_at), MAX(event.answered_at),
           MAX(CASE WHEN event.correct = 1 THEN event.answered_at END),
           MAX(CASE WHEN event.correct = 0 THEN event.answered_at END)
         FROM review_event event GROUP BY event.sense_uid;
         DELETE FROM daily_stat;
         INSERT INTO daily_stat(
           local_date, attempt_count, correct_count, wrong_count,
           unique_word_count, last_activity_at
         ) SELECT date(answered_at / 1000, 'unixepoch', 'localtime'), COUNT(*),
           SUM(correct), SUM(NOT correct), COUNT(DISTINCT sense_uid), MAX(answered_at)
         FROM review_event GROUP BY date(answered_at / 1000, 'unixepoch', 'localtime');
         UPDATE study_session SET
           attempt_count = (SELECT COUNT(*) FROM review_event WHERE session_id = study_session.id),
           correct_count = (SELECT COUNT(*) FROM review_event WHERE session_id = study_session.id AND correct = 1),
           wrong_count = (SELECT COUNT(*) FROM review_event WHERE session_id = study_session.id AND correct = 0);",
    )
}

fn deleted_article(id: String, revision: i64) -> SyncArticle {
    SyncArticle {
        id,
        title: String::new(),
        body: String::new(),
        translated_body: None,
        created_at: 0,
        updated_at: 0,
        revision,
        deleted: true,
    }
}

fn short_id(value: &str) -> &str {
    value.get(..8).unwrap_or(value)
}

fn to_sql_error(error: ApplicationError) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}
