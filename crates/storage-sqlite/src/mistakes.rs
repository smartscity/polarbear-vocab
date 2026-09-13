use std::cmp::Reverse;
use std::collections::HashSet;

use polarbear_vocab_application::{ApplicationError, MistakeQueryPort};
use polarbear_vocab_domain::WrongWordDto;

use crate::{SqliteStore, database_error, read_model};

#[derive(Clone, Debug)]
struct MistakeStat {
    sense_uid: String,
    wrong_count: u32,
    correct_count: u32,
    last_result: String,
    last_wrong_at: i64,
}

impl MistakeQueryPort for SqliteStore {
    fn list_wrong_words(
        &self,
        dataset_id: Option<&str>,
        min_wrong_count: u32,
        last_wrong_only: bool,
    ) -> Result<Vec<WrongWordDto>, ApplicationError> {
        let mut stats = self.load_mistake_stats(min_wrong_count, last_wrong_only)?;
        let (allowed, details) = {
            let content = self.content()?;
            let allowed = dataset_id
                .map(|uid| read_model::dataset_sense_uids(&content, uid))
                .transpose()
                .map_err(database_error)?
                .map(|uids| uids.into_iter().collect::<HashSet<_>>());
            let details = read_model::all_sense_details(&content).map_err(database_error)?;
            (allowed, details)
        };
        if let Some(allowed) = allowed {
            stats.retain(|stat| allowed.contains(&stat.sense_uid));
        }
        stats.sort_by_key(|stat| Reverse((stat.wrong_count, stat.last_wrong_at)));
        Ok(stats
            .into_iter()
            .filter_map(|stat| {
                let detail = details.get(&stat.sense_uid)?;
                Some(WrongWordDto {
                    sense_uid: stat.sense_uid,
                    lemma: detail.lemma.clone(),
                    ipa: detail.ipa.clone(),
                    zh_gloss: detail.zh_gloss.clone(),
                    wrong_count: stat.wrong_count,
                    correct_count: stat.correct_count,
                    last_result: stat.last_result,
                })
            })
            .collect())
    }
}

impl SqliteStore {
    fn load_mistake_stats(
        &self,
        min_wrong_count: u32,
        last_wrong_only: bool,
    ) -> Result<Vec<MistakeStat>, ApplicationError> {
        let user = self.user()?;
        let mut statement = user
            .prepare(
                "SELECT sense_uid, wrong_count, correct_count, last_result, last_wrong_at
                 FROM word_stat
                 WHERE wrong_count >= ?1
                   AND (?2 = 0 OR last_result = 'wrong')",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(rusqlite::params![min_wrong_count, last_wrong_only], |row| {
                Ok(MistakeStat {
                    sense_uid: row.get(0)?,
                    wrong_count: row.get(1)?,
                    correct_count: row.get(2)?,
                    last_result: row.get(3)?,
                    last_wrong_at: row.get(4)?,
                })
            })
            .map_err(database_error)?;
        rows.collect::<rusqlite::Result<_>>()
            .map_err(database_error)
    }
}
