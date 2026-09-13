use std::collections::HashMap;

use chrono::Utc;
use polarbear_vocab_application::{ApplicationError, StudyPort};
use polarbear_vocab_collection_engine::{WordProgress, resolve_collection};
use polarbear_vocab_domain::{AnswerResultDto, CollectionSession, CollectionSpec, QuizQuestionDto};
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use crate::{SqliteStore, answer_history, database_error, read_model, schema};

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
        answer_history::record_answer(
            self,
            answer_history::AnswerWrite {
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
            },
        )?;
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
        Ok(resolve_collection(spec, candidates, &progress))
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
}
