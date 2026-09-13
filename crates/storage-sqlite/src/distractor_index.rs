use rusqlite::{Transaction, params};

#[derive(Clone, Debug)]
struct Candidate {
    sense_uid: String,
    lemma: String,
    part_of_speech: String,
    prompt_zh: String,
    gloss_zh: String,
}

pub fn refresh_for(
    transaction: &Transaction<'_>,
    prompt_sense_uids: &[String],
) -> rusqlite::Result<()> {
    let candidates = load_candidates(transaction)?;
    for prompt_uid in prompt_sense_uids {
        let Some(prompt) = candidates.iter().find(|item| item.sense_uid == *prompt_uid) else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        };
        let mut ranked: Vec<(&Candidate, (u8, usize, &str))> = candidates
            .iter()
            .filter(|candidate| eligible(prompt, candidate))
            .map(|candidate| {
                let different_pos = u8::from(candidate.part_of_speech != prompt.part_of_speech);
                let length_delta = candidate.lemma.len().abs_diff(prompt.lemma.len());
                (
                    candidate,
                    (different_pos, length_delta, candidate.sense_uid.as_str()),
                )
            })
            .collect();
        ranked.sort_by_key(|(_, rank)| *rank);
        if ranked.len() < 3 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        transaction.execute(
            "DELETE FROM distractor_edge WHERE prompt_sense_uid = ?1",
            [prompt_uid],
        )?;
        for (index, (candidate, _)) in ranked.into_iter().take(3).enumerate() {
            transaction.execute(
                "INSERT INTO distractor_edge(
                    prompt_sense_uid, candidate_sense_uid, score, reason
                 ) VALUES (?1, ?2, ?3, 'lexical-shape-v1')",
                params![prompt_uid, candidate.sense_uid, 1.0 - index as f64 / 10.0],
            )?;
        }
    }
    Ok(())
}

fn load_candidates(transaction: &Transaction<'_>) -> rusqlite::Result<Vec<Candidate>> {
    let mut statement = transaction.prepare(
        "SELECT s.uid, w.lemma, s.pos, s.quiz_prompt_zh, s.zh_gloss
         FROM sense s JOIN word w ON w.id = s.word_id",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(Candidate {
            sense_uid: row.get(0)?,
            lemma: row.get(1)?,
            part_of_speech: row.get(2)?,
            prompt_zh: row.get(3)?,
            gloss_zh: row.get(4)?,
        })
    })?;
    rows.collect()
}

fn eligible(prompt: &Candidate, candidate: &Candidate) -> bool {
    prompt.sense_uid != candidate.sense_uid
        && !prompt.lemma.eq_ignore_ascii_case(&candidate.lemma)
        && prompt.prompt_zh != candidate.prompt_zh
        && !meaning_overlaps(&prompt.gloss_zh, &candidate.gloss_zh)
}

fn meaning_overlaps(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    !left.is_empty()
        && !right.is_empty()
        && (left == right || left.contains(right) || right.contains(left))
}
