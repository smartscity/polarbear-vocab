use std::collections::{HashMap, HashSet};

use anyhow::ensure;
use rusqlite::{Connection, Transaction, params};

use crate::preloaded_rows::{Batch, Row};

pub fn append(connection: &mut Connection, batch: &Batch) -> anyhow::Result<()> {
    let transaction = connection.transaction()?;
    transaction.execute(
        "INSERT INTO source(name, license, url, attribution) VALUES (?1, ?2, ?3, ?4)",
        params![
            "KyleBing/english-vocabulary derived local CSVs",
            "UNVERIFIED-NOT-FOR-REDISTRIBUTION",
            "https://github.com/KyleBing/english-vocabulary",
            "Local-only dataset build; upstream redistribution rights must be verified"
        ],
    )?;
    let source_id = transaction.last_insert_rowid();
    for row in &batch.rows {
        insert_sense(&transaction, source_id, row)?;
    }
    let mut sequence_offsets = HashMap::new();
    for item in &batch.memberships {
        let offset = if let Some(offset) = sequence_offsets.get(&item.dataset_id) {
            *offset
        } else {
            let offset: u32 = transaction.query_row(
                "SELECT COALESCE(MAX(sequence) + 1, 0) FROM dataset_item WHERE dataset_id = ?1",
                [&item.dataset_id],
                |row| row.get(0),
            )?;
            sequence_offsets.insert(item.dataset_id.clone(), offset);
            offset
        };
        transaction
            .prepare_cached(
                "INSERT INTO dataset_item(dataset_id, sense_uid, sequence) VALUES (?1, ?2, ?3)",
            )?
            .execute(params![
                item.dataset_id,
                item.sense_uid,
                offset + item.sequence
            ])?;
    }
    insert_distractors(&transaction, &batch.rows)?;
    let violations: i64 =
        transaction.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    ensure!(violations == 0, "preloaded data has foreign-key violations");
    transaction.commit()?;
    Ok(())
}

fn insert_sense(transaction: &Transaction<'_>, source_id: i64, row: &Row) -> anyhow::Result<()> {
    let word_uid = format!("{}.word", row.lemma.to_lowercase());
    transaction
        .prepare_cached(
            "INSERT INTO word(uid, lemma, frequency_rank) VALUES (?1, ?2, NULL)
         ON CONFLICT(uid) DO NOTHING",
        )?
        .execute(params![word_uid, row.lemma])?;
    let word_id: i64 = transaction.query_row(
        "SELECT id FROM word WHERE uid = ?1",
        [&word_uid],
        |record| record.get(0),
    )?;
    transaction
        .prepare_cached(
            "INSERT INTO sense(uid, word_id, source_id, pos, quiz_prompt_zh, zh_gloss,
            en_definition, cefr) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', '')",
        )?
        .execute(params![
            row.uid, word_id, source_id, row.pos, row.prompt, row.gloss
        ])?;
    let sense_id = transaction.last_insert_rowid();
    for (accent, ipa) in [("en-US", &row.ipa_us), ("en-GB", &row.ipa_uk)] {
        if !ipa.is_empty() {
            transaction
                .prepare_cached(
                    "INSERT INTO pronunciation(sense_id, accent, ipa) VALUES (?1, ?2, ?3)",
                )?
                .execute(params![sense_id, accent, ipa])?;
        }
    }
    if !row.example_en.is_empty() {
        transaction
            .prepare_cached(
                "INSERT INTO example(sense_id, sentence_en, sentence_zh, is_primary)
             VALUES (?1, ?2, ?3, 1)",
            )?
            .execute(params![sense_id, row.example_en, row.example_zh])?;
    }
    Ok(())
}

fn insert_distractors(transaction: &Transaction<'_>, rows: &[Row]) -> anyhow::Result<()> {
    let offsets = [17_usize, 41, 73, 101, 149, 211, 307, 431, 613, 997];
    for (index, prompt) in rows.iter().enumerate() {
        let mut chosen_uids = HashSet::new();
        let mut chosen_lemmas = HashSet::new();
        for offset in offsets {
            let candidate = &rows[(index + offset) % rows.len()];
            if candidate.uid == prompt.uid
                || candidate.lemma.eq_ignore_ascii_case(&prompt.lemma)
                || candidate.prompt == prompt.prompt
                || meaning_overlaps(&candidate.gloss, &prompt.gloss)
                || chosen_uids.contains(candidate.uid.as_str())
                || chosen_lemmas.contains(&candidate.lemma.to_lowercase())
            {
                continue;
            }
            chosen_uids.insert(candidate.uid.as_str());
            chosen_lemmas.insert(candidate.lemma.to_lowercase());
            transaction.prepare_cached(
                "INSERT INTO distractor_edge(prompt_sense_uid, candidate_sense_uid, score, reason)
                 VALUES (?1, ?2, ?3, 'preloaded-distance-v1')",
            )?.execute(params![prompt.uid, candidate.uid, 1.0 - chosen_uids.len() as f64 / 10.0])?;
            if chosen_uids.len() == 3 {
                break;
            }
        }
        ensure!(
            chosen_uids.len() == 3,
            "cannot find three distractors for {}",
            prompt.uid
        );
    }
    Ok(())
}

fn meaning_overlaps(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    !left.is_empty()
        && !right.is_empty()
        && (left == right || left.contains(right) || right.contains(left))
}

#[cfg(test)]
mod tests {
    use super::append;
    use crate::build::create_schema;
    use crate::preloaded_rows::{Batch, Membership, Row};
    use rusqlite::Connection;

    #[test]
    fn appended_memberships_follow_starter_rows_and_options_are_distinct() {
        let mut connection = Connection::open_in_memory().expect("open database");
        create_schema(&connection).expect("create content schema");
        connection
            .execute_batch(
                "INSERT INTO dataset(id, name, created_at, preloaded)
                 VALUES ('cet4', 'CET-4', 0, 1);
                 INSERT INTO source(name, license, attribution)
                 VALUES ('starter', 'CC0', 'test');
                 INSERT INTO word(uid, lemma) VALUES ('starter.word', 'starter');
                 INSERT INTO sense(uid, word_id, source_id, pos, quiz_prompt_zh,
                   zh_gloss, en_definition, cefr)
                 VALUES ('starter.n.01', 1, 1, 'noun', '起始', '起始', '', '');
                 INSERT INTO dataset_item(dataset_id, sense_uid, sequence)
                 VALUES ('cet4', 'starter.n.01', 0);",
            )
            .expect("insert starter row");
        let rows: Vec<Row> = (0..12)
            .map(|index| Row {
                uid: format!("test.{index}.01"),
                lemma: format!("word{index}"),
                pos: "noun".to_owned(),
                prompt: format!("释义{index}"),
                gloss: format!("释义{index}"),
                ipa_us: String::new(),
                ipa_uk: String::new(),
                example_en: String::new(),
                example_zh: String::new(),
            })
            .collect();
        let memberships = rows
            .iter()
            .enumerate()
            .map(|(sequence, row)| Membership {
                dataset_id: "cet4".to_owned(),
                sense_uid: row.uid.clone(),
                sequence: sequence as u32,
            })
            .collect();
        append(&mut connection, &Batch { rows, memberships }).expect("append full dataset");
        let counts: (i64, i64) = connection
            .query_row(
                "SELECT COUNT(*), COUNT(DISTINCT sequence) FROM dataset_item",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("count memberships");
        assert_eq!(counts, (13, 13));
        let edge_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM distractor_edge", [], |row| row.get(0))
            .expect("count quiz options");
        assert_eq!(edge_count, 36);
        let ambiguous: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM (
                   SELECT edge.prompt_sense_uid
                   FROM distractor_edge edge
                   JOIN sense ON edge.candidate_sense_uid = sense.uid
                   JOIN word ON sense.word_id = word.id
                   GROUP BY edge.prompt_sense_uid
                   HAVING COUNT(DISTINCT lower(word.lemma)) != 3
                 )",
                [],
                |row| row.get(0),
            )
            .expect("check distractor lemmas");
        assert_eq!(ambiguous, 0);
    }
}
