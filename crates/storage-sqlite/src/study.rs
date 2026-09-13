use std::cmp::Reverse;
use std::collections::HashMap;

use chrono::{Local, Utc};
use polarbear_vocab_application::{ApplicationError, StudyPort};
use polarbear_vocab_domain::{AnswerResultDto, CollectionSession, CollectionSpec, QuizQuestionDto};
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use crate::{SqliteStore, database_error, read_model, schema};

#[derive(Clone, Debug, Default)]
struct WordProgress {
    attempt_count: u32,
    correct_count: u32,
    wrong_count: u32,
    last_result: Option<String>,
    last_wrong_at: Option<i64>,
}

impl StudyPort for SqliteStore {
    fn start_collection(
        &self,
        spec: &CollectionSpec,
    ) -> Result<CollectionSession, ApplicationError> {
        let sense_uids = self.resolve_collection(spec)?;
        let session_id = Uuid::new_v4().to_string();
        let started_at = Utc::now().timestamp_millis();
        let spec_json = serde_json::to_string(spec)
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))?;
        let mut user = self.user()?;
        schema::in_immediate_transaction(&mut user, |transaction| {
            transaction.execute(
                "INSERT INTO study_session(
                    id, dataset_id, collection_type, collection_spec_json, started_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    session_id,
                    spec.dataset_id(),
                    spec.collection_type(),
                    spec_json,
                    started_at
                ],
            )?;
            let mut insert = transaction.prepare(
                "INSERT INTO session_item(session_id, ordinal, sense_uid)
                 VALUES (?1, ?2, ?3)",
            )?;
            for (ordinal, sense_uid) in sense_uids.iter().enumerate() {
                insert.execute(params![session_id, ordinal as u32, sense_uid])?;
            }
            Ok(())
        })
        .map_err(database_error)?;
        Ok(CollectionSession {
            collection_id: session_id.clone(),
            session_id,
            dataset_id: spec.dataset_id().map(str::to_owned),
            collection_type: spec.collection_type().to_owned(),
            total_count: sense_uids.len() as u32,
        })
    }

    fn next_question(
        &self,
        collection_id: &str,
    ) -> Result<Option<QuizQuestionDto>, ApplicationError> {
        let next = {
            let user = self.user()?;
            user.query_row(
                "SELECT item.ordinal, item.sense_uid,
                        (SELECT COUNT(*) FROM session_item WHERE session_id = ?1 AND answered = 1),
                        (SELECT COUNT(*) FROM session_item WHERE session_id = ?1)
                 FROM session_item item
                 WHERE item.session_id = ?1 AND item.answered = 0
                 ORDER BY item.ordinal
                 LIMIT 1",
                [collection_id],
                |row| {
                    Ok((
                        row.get::<_, u32>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, u32>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(database_error)?
        };
        let Some((ordinal, sense_uid, answered_count, total_count)) = next else {
            return Ok(None);
        };
        let content = self.content()?;
        read_model::build_question(
            &content,
            collection_id,
            ordinal,
            &sense_uid,
            answered_count,
            total_count,
        )
        .map_err(database_error)
    }

    fn submit_answer(
        &self,
        collection_id: &str,
        question_id: &str,
        selected_option_id: &str,
        latency_ms: Option<u32>,
    ) -> Result<AnswerResultDto, ApplicationError> {
        let pending = self.pending_answer(collection_id, question_id)?;
        let (ordinal, sense_uid, dataset_id, collection_type) = pending;
        let (question, detail) = {
            let content = self.content()?;
            let question =
                read_model::build_question(&content, collection_id, ordinal, &sense_uid, 0, 0)
                    .map_err(database_error)?
                    .ok_or_else(|| ApplicationError::NotFound(sense_uid.clone()))?;
            let detail = read_model::sense_detail(&content, &sense_uid)
                .map_err(database_error)?
                .ok_or_else(|| ApplicationError::NotFound(sense_uid.clone()))?;
            (question, detail)
        };
        if !question
            .options
            .iter()
            .any(|option| option.option_id == selected_option_id)
        {
            return Err(ApplicationError::InvalidInput(
                "selected option is not part of the question".to_owned(),
            ));
        }
        let correct = selected_option_id == sense_uid;
        self.record_answer(AnswerWrite {
            collection_id,
            ordinal,
            sense_uid: &sense_uid,
            dataset_id: dataset_id.as_deref(),
            collection_type: &collection_type,
            selected_option_id,
            latency_ms,
            correct,
            options_json: serde_json::to_string(&question.options)
                .map_err(|error| ApplicationError::Infrastructure(error.to_string()))?,
        })?;
        Ok(AnswerResultDto {
            correct,
            correct_sense_uid: sense_uid,
            selected_sense_uid: selected_option_id.to_owned(),
            lemma: detail.lemma,
            ipa: detail.ipa,
            zh_gloss: detail.zh_gloss,
            example_en: detail.example_en,
            example_zh: detail.example_zh,
        })
    }

    fn finish_session(&self, session_id: &str) -> Result<(), ApplicationError> {
        let mut user = self.user()?;
        let updated = schema::in_immediate_transaction(&mut user, |transaction| {
            transaction.execute(
                "UPDATE study_session SET ended_at = ?2 WHERE id = ?1 AND ended_at IS NULL",
                params![session_id, Utc::now().timestamp_millis()],
            )
        })
        .map_err(database_error)?;
        if updated == 0 {
            return Err(ApplicationError::NotFound(session_id.to_owned()));
        }
        Ok(())
    }
}

impl SqliteStore {
    fn resolve_collection(&self, spec: &CollectionSpec) -> Result<Vec<String>, ApplicationError> {
        let candidates = {
            let content = self.content()?;
            match spec.dataset_id() {
                Some(dataset_id) => {
                    read_model::dataset_sense_uids(&content, dataset_id).map_err(database_error)?
                }
                None => match spec {
                    CollectionSpec::Custom { sense_uids } => sense_uids.clone(),
                    _ => read_model::all_sense_uids(&content).map_err(database_error)?,
                },
            }
        };
        let progress = self.load_progress()?;
        let mut resolved: Vec<String> = candidates
            .into_iter()
            .filter(|uid| matches_collection(spec, progress.get(uid)))
            .collect();
        if matches!(
            spec,
            CollectionSpec::Wrong { .. } | CollectionSpec::LastWrong { .. }
        ) {
            resolved.sort_by_key(|uid| {
                let stat = progress.get(uid).cloned().unwrap_or_default();
                Reverse((stat.wrong_count, stat.last_wrong_at.unwrap_or_default()))
            });
        }
        Ok(resolved)
    }

    fn load_progress(&self) -> Result<HashMap<String, WordProgress>, ApplicationError> {
        let user = self.user()?;
        let mut statement = user
            .prepare(
                "SELECT sense_uid, attempt_count, correct_count, wrong_count,
                    last_result, last_wrong_at
                 FROM word_stat",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    WordProgress {
                        attempt_count: row.get(1)?,
                        correct_count: row.get(2)?,
                        wrong_count: row.get(3)?,
                        last_result: row.get(4)?,
                        last_wrong_at: row.get(5)?,
                    },
                ))
            })
            .map_err(database_error)?;
        rows.collect::<rusqlite::Result<_>>()
            .map_err(database_error)
    }

    fn pending_answer(
        &self,
        collection_id: &str,
        question_id: &str,
    ) -> Result<(u32, String, Option<String>, String), ApplicationError> {
        let user = self.user()?;
        let pending = user
            .query_row(
                "SELECT item.ordinal, item.sense_uid, session.dataset_id,
                        session.collection_type
                 FROM session_item item
                 JOIN study_session session ON session.id = item.session_id
                 WHERE item.session_id = ?1 AND item.answered = 0
                 ORDER BY item.ordinal LIMIT 1",
                [collection_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(database_error)?
            .ok_or_else(|| ApplicationError::NotFound(question_id.to_owned()))?;
        let expected_id = format!("{collection_id}:{}", pending.0);
        if question_id != expected_id {
            return Err(ApplicationError::Conflict(
                "question is no longer current".to_owned(),
            ));
        }
        Ok(pending)
    }

    fn record_answer(&self, write: AnswerWrite<'_>) -> Result<(), ApplicationError> {
        let answered_at = Utc::now().timestamp_millis();
        let local_date = Local::now().date_naive().to_string();
        let event_id = Uuid::new_v4().to_string();
        let mut user = self.user()?;
        schema::in_immediate_transaction(&mut user, |transaction| {
            ensure_unanswered(transaction, write.collection_id, write.ordinal)?;
            let is_unique = transaction.query_row(
                "SELECT NOT EXISTS(
                    SELECT 1 FROM review_event
                    WHERE sense_uid = ?1
                      AND date(answered_at / 1000, 'unixepoch', 'localtime') = ?2
                 )",
                params![write.sense_uid, local_date],
                |row| row.get::<_, bool>(0),
            )?;
            insert_review_event(transaction, &write, &event_id, answered_at)?;
            update_word_stat(transaction, &write, answered_at)?;
            update_daily_stat(transaction, &write, &local_date, answered_at, is_unique)?;
            update_session(transaction, &write)?;
            Ok(())
        })
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                ApplicationError::Conflict("question was already answered".to_owned())
            }
            other => database_error(other),
        })
    }
}

struct AnswerWrite<'a> {
    collection_id: &'a str,
    ordinal: u32,
    sense_uid: &'a str,
    dataset_id: Option<&'a str>,
    collection_type: &'a str,
    selected_option_id: &'a str,
    latency_ms: Option<u32>,
    correct: bool,
    options_json: String,
}

fn matches_collection(spec: &CollectionSpec, stat: Option<&WordProgress>) -> bool {
    let stat = stat.cloned().unwrap_or_default();
    match spec {
        CollectionSpec::Dataset { .. } | CollectionSpec::Custom { .. } => true,
        CollectionSpec::Unseen { .. } => stat.attempt_count == 0,
        CollectionSpec::Answered { .. } => stat.attempt_count > 0,
        CollectionSpec::Correct { .. } => stat.correct_count > 0,
        CollectionSpec::Wrong {
            min_wrong_count, ..
        } => stat.wrong_count >= *min_wrong_count,
        CollectionSpec::LastWrong { .. } => stat.last_result.as_deref() == Some("wrong"),
    }
}

fn ensure_unanswered(
    transaction: &rusqlite::Transaction<'_>,
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

fn insert_review_event(
    transaction: &rusqlite::Transaction<'_>,
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
    transaction: &rusqlite::Transaction<'_>,
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
    transaction: &rusqlite::Transaction<'_>,
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
    transaction: &rusqlite::Transaction<'_>,
    write: &AnswerWrite<'_>,
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
            wrong_count = wrong_count + ?3
         WHERE id = ?1",
        params![
            write.collection_id,
            u8::from(write.correct),
            u8::from(!write.correct)
        ],
    )?;
    Ok(())
}
