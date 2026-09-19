use polarbear_vocab_domain::QuestionOption;
use sha2::{Digest, Sha256};

pub(super) fn shuffle_session_items(items: &mut [String], session_id: &str) {
    items.sort_by_cached_key(|sense_uid| rank(session_id, sense_uid));
}

pub(super) fn shuffle_question_options(options: &mut [QuestionOption], question_id: &str) {
    options.sort_by_cached_key(|option| rank(question_id, &option.sense_uid));
}

fn rank(seed: &str, value: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(seed.as_bytes());
    digest.update([0]);
    digest.update(value.as_bytes());
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use polarbear_vocab_domain::QuestionOption;

    use super::{shuffle_question_options, shuffle_session_items};

    #[test]
    fn session_shuffle_is_stable_for_one_session_and_varies_between_sessions() {
        let source = ["alpha", "bravo", "charlie", "delta", "echo"]
            .map(str::to_owned)
            .to_vec();
        let mut first = source.clone();
        let mut repeated = source.clone();
        let mut second = source;

        shuffle_session_items(&mut first, "session-a");
        shuffle_session_items(&mut repeated, "session-a");
        shuffle_session_items(&mut second, "session-b");

        assert_eq!(first, repeated);
        assert_ne!(first, second);
    }

    #[test]
    fn option_shuffle_is_stable_for_rebuilt_questions() {
        let mut options: Vec<QuestionOption> = ["a", "b", "c", "d"]
            .map(|uid| QuestionOption {
                option_id: uid.to_owned(),
                sense_uid: uid.to_owned(),
                lemma: uid.to_owned(),
            })
            .to_vec();
        let mut repeated = options.clone();

        shuffle_question_options(&mut options, "session:3");
        shuffle_question_options(&mut repeated, "session:3");

        assert_eq!(options, repeated);
        assert_ne!(options[0].sense_uid, "a");
    }
}
