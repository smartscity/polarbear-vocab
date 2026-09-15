use std::collections::HashMap;

use polarbear_lexicon::SenseDetail;
use polarbear_vocab_domain::{DatasetSummary, QuestionOption, QuizQuestionDto};
use rusqlite::{Connection, OptionalExtension};

pub fn list_datasets(connection: &Connection) -> rusqlite::Result<Vec<DatasetSummary>> {
    let mut statement = connection.prepare(
        "SELECT d.id, d.name, d.created_at, d.updated_at, d.preloaded, COUNT(di.sense_uid)
         FROM dataset d
         LEFT JOIN dataset_item di ON di.dataset_id = d.id
         GROUP BY d.id
         ORDER BY d.preloaded DESC, d.created_at, d.name",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(DatasetSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            preloaded: row.get(4)?,
            word_count: row.get::<_, u32>(5)?,
        })
    })?;
    rows.collect()
}

pub fn dataset_summary(
    connection: &Connection,
    dataset_id: &str,
) -> rusqlite::Result<Option<DatasetSummary>> {
    connection
        .query_row(
            "SELECT d.id, d.name, d.created_at, d.updated_at, d.preloaded, COUNT(di.sense_uid)
             FROM dataset d
             LEFT JOIN dataset_item di ON di.dataset_id = d.id
             WHERE d.id = ?1
             GROUP BY d.id",
            [dataset_id],
            |row| {
                Ok(DatasetSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    preloaded: row.get(4)?,
                    word_count: row.get(5)?,
                })
            },
        )
        .optional()
}

pub fn dataset_sense_uids(
    connection: &Connection,
    dataset_id: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT s.uid
         FROM dataset_item di
         JOIN sense s ON s.uid = di.sense_uid
         WHERE di.dataset_id = ?1
         ORDER BY di.sequence, s.uid",
    )?;
    let rows = statement.query_map([dataset_id], |row| row.get(0))?;
    rows.collect()
}

pub fn all_sense_uids(connection: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare("SELECT uid FROM sense ORDER BY uid")?;
    let rows = statement.query_map([], |row| row.get(0))?;
    rows.collect()
}

pub fn all_sense_details(
    connection: &Connection,
) -> rusqlite::Result<HashMap<String, SenseDetail>> {
    let mut statement = connection.prepare(
        "SELECT s.uid, w.lemma, s.pos, s.quiz_prompt_zh, s.zh_gloss,
                COALESCE(p.ipa, ''), e.sentence_en, e.sentence_zh
         FROM sense s
         JOIN word w ON w.id = s.word_id
         LEFT JOIN pronunciation p ON p.sense_id = s.id AND p.accent = 'en-US'
         LEFT JOIN example e ON e.sense_id = s.id AND e.is_primary = 1",
    )?;
    let rows = statement.query_map([], map_sense_detail)?;
    let mut details = HashMap::new();
    for row in rows {
        let detail = row?;
        details.insert(detail.sense_uid.clone(), detail);
    }
    Ok(details)
}

pub fn sense_detail(
    connection: &Connection,
    sense_uid: &str,
) -> rusqlite::Result<Option<SenseDetail>> {
    connection
        .query_row(
            "SELECT s.uid, w.lemma, s.pos, s.quiz_prompt_zh, s.zh_gloss,
                    COALESCE(p.ipa, ''), e.sentence_en, e.sentence_zh
             FROM sense s
             JOIN word w ON w.id = s.word_id
             LEFT JOIN pronunciation p ON p.sense_id = s.id AND p.accent = 'en-US'
             LEFT JOIN example e ON e.sense_id = s.id AND e.is_primary = 1
             WHERE s.uid = ?1",
            [sense_uid],
            map_sense_detail,
        )
        .optional()
}

pub fn build_question(
    connection: &Connection,
    session_id: &str,
    ordinal: u32,
    sense_uid: &str,
    answered_count: u32,
    total_count: u32,
) -> rusqlite::Result<Option<QuizQuestionDto>> {
    let Some(detail) = sense_detail(connection, sense_uid)? else {
        return Ok(None);
    };
    let mut options = vec![QuestionOption {
        option_id: sense_uid.to_owned(),
        sense_uid: sense_uid.to_owned(),
        lemma: detail.lemma,
    }];
    let mut statement = connection.prepare(
        "SELECT candidate.uid, word.lemma
         FROM distractor_edge edge
         JOIN sense candidate ON candidate.uid = edge.candidate_sense_uid
         JOIN word ON word.id = candidate.word_id
         WHERE edge.prompt_sense_uid = ?1
         ORDER BY edge.score DESC, candidate.uid
         LIMIT 3",
    )?;
    let candidates = statement.query_map([sense_uid], |row| {
        let candidate_uid: String = row.get(0)?;
        Ok(QuestionOption {
            option_id: candidate_uid.clone(),
            sense_uid: candidate_uid,
            lemma: row.get(1)?,
        })
    })?;
    for candidate in candidates {
        options.push(candidate?);
    }
    if options.len() != 4 {
        return Ok(None);
    }
    options.rotate_left((ordinal as usize) % 4);
    Ok(Some(QuizQuestionDto {
        question_id: format!("{session_id}:{ordinal}"),
        sense_uid: sense_uid.to_owned(),
        prompt_zh: detail.prompt_zh,
        options,
        answered_count,
        total_count,
    }))
}

fn map_sense_detail(row: &rusqlite::Row<'_>) -> rusqlite::Result<SenseDetail> {
    Ok(SenseDetail {
        sense_uid: row.get(0)?,
        lemma: row.get(1)?,
        part_of_speech: row.get(2)?,
        prompt_zh: row.get(3)?,
        zh_gloss: row.get(4)?,
        ipa: row.get(5)?,
        example_en: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        example_zh: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
    })
}
