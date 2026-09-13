use std::cmp::Reverse;
use std::collections::HashMap;

use polarbear_vocab_domain::CollectionSpec;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WordProgress {
    pub attempt_count: u32,
    pub correct_count: u32,
    pub wrong_count: u32,
    pub last_result: Option<String>,
    pub last_wrong_at: Option<i64>,
}

#[must_use]
pub fn resolve_collection(
    spec: &CollectionSpec,
    candidates: Vec<String>,
    progress: &HashMap<String, WordProgress>,
) -> Vec<String> {
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
    resolved
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use polarbear_vocab_domain::CollectionSpec;

    use super::{WordProgress, resolve_collection};

    fn progress() -> HashMap<String, WordProgress> {
        HashMap::from([
            ("new".to_owned(), WordProgress::default()),
            (
                "mixed".to_owned(),
                WordProgress {
                    attempt_count: 4,
                    correct_count: 2,
                    wrong_count: 2,
                    last_result: Some("correct".to_owned()),
                    last_wrong_at: Some(10),
                },
            ),
            (
                "wrong".to_owned(),
                WordProgress {
                    attempt_count: 6,
                    correct_count: 1,
                    wrong_count: 5,
                    last_result: Some("wrong".to_owned()),
                    last_wrong_at: Some(20),
                },
            ),
        ])
    }

    fn candidates() -> Vec<String> {
        ["new", "mixed", "wrong"].map(str::to_owned).to_vec()
    }

    #[test]
    fn resolves_history_derived_collections() {
        let progress = progress();

        assert_eq!(
            resolve_collection(
                &CollectionSpec::Unseen {
                    dataset_id: "dataset".to_owned(),
                },
                candidates(),
                &progress,
            ),
            ["new"]
        );
        assert_eq!(
            resolve_collection(
                &CollectionSpec::Correct { dataset_id: None },
                candidates(),
                &progress,
            ),
            ["mixed", "wrong"]
        );
        assert_eq!(
            resolve_collection(
                &CollectionSpec::LastWrong { dataset_id: None },
                candidates(),
                &progress,
            ),
            ["wrong"]
        );
    }

    #[test]
    fn wrong_collection_applies_threshold_and_priority_order() {
        let mut progress = progress();
        progress.get_mut("mixed").unwrap().last_wrong_at = Some(30);

        assert_eq!(
            resolve_collection(
                &CollectionSpec::Wrong {
                    dataset_id: None,
                    min_wrong_count: 2,
                },
                candidates(),
                &progress,
            ),
            ["wrong", "mixed"]
        );
    }
}
