use chrono::{Local, Utc};
use polarbear_vocab_application::ApplicationError;
use rusqlite::{OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::{SqliteStore, database_error, schema};

pub(super) struct AnswerWrite<'a> {
    pub collection_id: &'a str,
    pub ordinal: u32,
    pub sense_uid: &'a str,
    pub dataset_id: Option<&'a str>,
    pub collection_type: &'a str,
    pub selected_option_id: &'a str,
    pub latency_ms: Option<u32>,
    pub correct: bool,
    pub options_json: String,
}

pub(super) fn record_answer(
    store: &SqliteStore,
    write: AnswerWrite<'_>,
) -> Result<bool, ApplicationError> {
    let answered_at = Utc::now().timestamp_millis();
    let local_date = Local::now().date_naive().to_string();
    let event_id = Uuid::new_v4().to_string();
    let mut user = store.user()?;
    schema::in_immediate_transaction(&mut user, |transaction| {
        ensure_unanswered(transaction, write.collection_id, write.ordinal)?;
        let was_new = transaction
            .query_row(
                "SELECT 1 FROM word_stat WHERE sense_uid = ?1",
                [write.sense_uid],
                |_| Ok(()),
            )
            .optional()?
            .is_none();
        let is_unique = is_first_attempt_today(transaction, write.sense_uid, &local_date)?;
        insert_review_event(transaction, &write, &event_id, answered_at)?;
        schema::record_sync_change(transaction, "reviewEvent", &event_id, "upsert", answered_at)?;
        update_word_stat(transaction, &write, answered_at)?;
        update_daily_stat(transaction, &write, &local_date, answered_at, is_unique)?;
        update_session(transaction, &write, was_new)?;
        Ok(was_new)
    })
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            ApplicationError::Conflict("question was already answered".to_owned())
        }
        other => database_error(other),
    })
}

fn ensure_unanswered(
    transaction: &Transaction<'_>,
    session_id: &str,
    ordinal: u32,
) -> rusqlite::Result<()> {
    transaction.query_row(
        "SELECT 1 FROM session_item
         WHERE session_id = ?1 AND ordinal = ?2 AND answered = 0",
        params![session_id, ordinal],
        |_| Ok(()),
    )
}

fn is_first_attempt_today(
    transaction: &Transaction<'_>,
    sense_uid: &str,
    local_date: &str,
) -> rusqlite::Result<bool> {
    transaction.query_row(
        "SELECT NOT EXISTS(
            SELECT 1 FROM review_event
            WHERE sense_uid = ?1
              AND date(answered_at / 1000, 'unixepoch', 'localtime') = ?2
         )",
        params![sense_uid, local_date],
        |row| row.get(0),
    )
}

fn insert_review_event(
    transaction: &Transaction<'_>,
    write: &AnswerWrite<'_>,
    event_id: &str,
    answered_at: i64,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO review_event(
            id, session_id, dataset_id, sense_uid, collection_type, answered_at,
            correct, selected_sense_uid, latency_ms, options_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            event_id,
            write.collection_id,
            write.dataset_id,
            write.sense_uid,
            write.collection_type,
            answered_at,
            write.correct,
            write.selected_option_id,
            write.latency_ms,
            write.options_json
        ],
    )?;
    Ok(())
}

fn update_word_stat(
    transaction: &Transaction<'_>,
    write: &AnswerWrite<'_>,
    answered_at: i64,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO word_stat(
            sense_uid, attempt_count, correct_count, wrong_count, last_result,
            first_answered_at, last_answered_at, last_correct_at, last_wrong_at
         ) VALUES (?1, 1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)
         ON CONFLICT(sense_uid) DO UPDATE SET
            attempt_count = attempt_count + 1,
            correct_count = correct_count + excluded.correct_count,
            wrong_count = wrong_count + excluded.wrong_count,
            last_result = excluded.last_result,
            last_answered_at = excluded.last_answered_at,
            last_correct_at = COALESCE(excluded.last_correct_at, last_correct_at),
            last_wrong_at = COALESCE(excluded.last_wrong_at, last_wrong_at)",
        params![
            write.sense_uid,
            u8::from(write.correct),
            u8::from(!write.correct),
            if write.correct { "correct" } else { "wrong" },
            answered_at,
            write.correct.then_some(answered_at),
            (!write.correct).then_some(answered_at)
        ],
    )?;
    Ok(())
}

fn update_daily_stat(
    transaction: &Transaction<'_>,
    write: &AnswerWrite<'_>,
    local_date: &str,
    answered_at: i64,
    is_unique: bool,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO daily_stat(
            local_date, attempt_count, correct_count, wrong_count,
            unique_word_count, last_activity_at
         ) VALUES (?1, 1, ?2, ?3, ?4, ?5)
         ON CONFLICT(local_date) DO UPDATE SET
            attempt_count = attempt_count + 1,
            correct_count = correct_count + excluded.correct_count,
            wrong_count = wrong_count + excluded.wrong_count,
            unique_word_count = unique_word_count + excluded.unique_word_count,
            last_activity_at = excluded.last_activity_at",
        params![
            local_date,
            u8::from(write.correct),
            u8::from(!write.correct),
            u8::from(is_unique),
            answered_at
        ],
    )?;
    Ok(())
}

fn update_session(
    transaction: &Transaction<'_>,
    write: &AnswerWrite<'_>,
    was_new: bool,
) -> rusqlite::Result<()> {
    transaction.execute(
        "UPDATE session_item SET answered = 1
         WHERE session_id = ?1 AND ordinal = ?2",
        params![write.collection_id, write.ordinal],
    )?;
    transaction.execute(
        "UPDATE study_session SET
            attempt_count = attempt_count + 1,
            correct_count = correct_count + ?2,
            wrong_count = wrong_count + ?3,
            new_word_count = new_word_count + ?4
         WHERE id = ?1",
        params![
            write.collection_id,
            u8::from(write.correct),
            u8::from(!write.correct),
            u8::from(was_new)
        ],
    )?;
    Ok(())
}
