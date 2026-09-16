use std::collections::{HashMap, HashSet};

use chrono::{Duration, Local};
use polarbear_vocab_application::{ApplicationError, HomeQueryPort};
use polarbear_vocab_domain::{DailyActivity, DatasetProgress, DatasetSummary, HomeDto};
use polarbear_vocab_statistics_engine::{WordStat, summarize};

use crate::{SqliteStore, database_error, dataset_order, read_model};

impl HomeQueryPort for SqliteStore {
    fn list_datasets(&self) -> Result<Vec<DatasetSummary>, ApplicationError> {
        let mut datasets = {
            let content = self.content()?;
            read_model::list_datasets(&content).map_err(database_error)?
        };
        let user = self.user()?;
        let order = dataset_order::read(&user)?;
        dataset_order::apply(&mut datasets, &order);
        Ok(datasets)
    }

    fn get_home(&self, dataset_id: &str) -> Result<HomeDto, ApplicationError> {
        let (dataset, sense_uids) = {
            let content = self.content()?;
            let dataset = read_model::dataset_summary(&content, dataset_id)
                .map_err(database_error)?
                .ok_or_else(|| ApplicationError::NotFound(dataset_id.to_owned()))?;
            let sense_uids =
                read_model::dataset_sense_uids(&content, dataset_id).map_err(database_error)?;
            (dataset, sense_uids)
        };
        let user = self.user()?;
        let stats = load_word_stats(&user)?;
        let daily_activity = load_daily_activity(&user)?;
        Ok(build_home(dataset, &sense_uids, &stats, daily_activity))
    }
}

fn load_word_stats(
    connection: &rusqlite::Connection,
) -> Result<HashMap<String, WordStat>, ApplicationError> {
    let mut statement = connection
        .prepare(
            "SELECT sense_uid, attempt_count, correct_count, wrong_count, last_result
             FROM word_stat",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                WordStat {
                    attempt_count: row.get(1)?,
                    correct_count: row.get(2)?,
                    wrong_count: row.get(3)?,
                    last_result: row.get(4)?,
                },
            ))
        })
        .map_err(database_error)?;
    rows.collect::<rusqlite::Result<_>>()
        .map_err(database_error)
}

fn load_daily_activity(
    connection: &rusqlite::Connection,
) -> Result<Vec<DailyActivity>, ApplicationError> {
    let mut statement = connection
        .prepare(
            "SELECT local_date, attempt_count, correct_count, wrong_count
             FROM daily_stat
             WHERE local_date >= date('now', '-29 days', 'localtime')",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(DailyActivity {
                local_date: row.get(0)?,
                attempt_count: row.get(1)?,
                correct_count: row.get(2)?,
                wrong_count: row.get(3)?,
            })
        })
        .map_err(database_error)?;
    let by_date: HashMap<String, DailyActivity> = rows
        .map(|row| row.map(|activity| (activity.local_date.clone(), activity)))
        .collect::<rusqlite::Result<_>>()
        .map_err(database_error)?;
    let today = Local::now().date_naive();
    Ok((0..30)
        .rev()
        .map(|days_ago| {
            let local_date = (today - Duration::days(days_ago)).to_string();
            by_date.get(&local_date).cloned().unwrap_or(DailyActivity {
                local_date,
                attempt_count: 0,
                correct_count: 0,
                wrong_count: 0,
            })
        })
        .collect())
}

fn build_home(
    dataset: DatasetSummary,
    sense_uids: &[String],
    stats: &HashMap<String, WordStat>,
    daily_activity: Vec<DailyActivity>,
) -> HomeDto {
    let included: HashSet<&str> = sense_uids.iter().map(String::as_str).collect();
    let relevant: Vec<&WordStat> = stats
        .iter()
        .filter(|(uid, _)| included.contains(uid.as_str()))
        .map(|(_, stat)| stat)
        .collect();
    let summary = summarize(&relevant);
    HomeDto {
        progress: DatasetProgress {
            total: sense_uids.len() as u32,
            answered: summary.answered,
            unseen: sense_uids.len() as u32 - summary.answered,
        },
        totals: summary.totals,
        mistake_buckets: summary.mistake_buckets,
        dataset,
        daily_activity,
    }
}
