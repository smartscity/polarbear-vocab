use polarbear_vocab_domain::{AnswerTotals, MistakeBuckets};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WordStat {
    pub attempt_count: u32,
    pub correct_count: u32,
    pub wrong_count: u32,
    pub last_result: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HistorySummary {
    pub answered: u32,
    pub totals: AnswerTotals,
    pub mistake_buckets: MistakeBuckets,
}

#[must_use]
pub fn summarize(stats: &[&WordStat]) -> HistorySummary {
    let count = |predicate: fn(&WordStat) -> bool| {
        stats.iter().filter(|stat| predicate(stat)).count() as u32
    };
    let answered = count(|stat| stat.attempt_count > 0);
    HistorySummary {
        answered,
        totals: AnswerTotals {
            explored: answered,
            correct: count(|stat| stat.correct_count > 0),
            mistakes: count(|stat| stat.wrong_count > 0),
        },
        mistake_buckets: MistakeBuckets {
            at_least_one: count(|stat| stat.wrong_count >= 1),
            at_least_two: count(|stat| stat.wrong_count >= 2),
            at_least_three: count(|stat| stat.wrong_count >= 3),
            at_least_five: count(|stat| stat.wrong_count >= 5),
            last_wrong: count(|stat| stat.last_result.as_deref() == Some("wrong")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{WordStat, summarize};

    #[test]
    fn totals_and_mistake_buckets_are_history_derived() {
        let unseen = WordStat::default();
        let recovered = WordStat {
            attempt_count: 3,
            correct_count: 2,
            wrong_count: 1,
            last_result: Some("correct".to_owned()),
        };
        let struggling = WordStat {
            attempt_count: 6,
            correct_count: 1,
            wrong_count: 5,
            last_result: Some("wrong".to_owned()),
        };

        let summary = summarize(&[&unseen, &recovered, &struggling]);

        assert_eq!(summary.answered, 2);
        assert_eq!(summary.totals.explored, 2);
        assert_eq!(summary.totals.correct, 2);
        assert_eq!(summary.totals.mistakes, 2);
        assert_eq!(summary.mistake_buckets.at_least_one, 2);
        assert_eq!(summary.mistake_buckets.at_least_five, 1);
        assert_eq!(summary.mistake_buckets.last_wrong, 1);
    }
}
