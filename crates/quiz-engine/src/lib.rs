#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuizOption {
    pub sense_uid: String,
    pub lemma: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuizQuestion {
    pub question_id: String,
    pub prompt_zh: String,
    pub correct_sense_uid: String,
    pub options: [QuizOption; 4],
}

impl QuizQuestion {
    #[must_use]
    pub fn has_valid_options(&self) -> bool {
        let mut sense_uids = self.options.iter().map(|option| &option.sense_uid);
        let Some(first) = sense_uids.next() else {
            return false;
        };

        self.options
            .iter()
            .filter(|option| option.sense_uid == self.correct_sense_uid)
            .count()
            == 1
            && sense_uids.all(|sense_uid| sense_uid != first)
            && self.options.iter().enumerate().all(|(index, option)| {
                self.options[..index]
                    .iter()
                    .all(|previous| previous.sense_uid != option.sense_uid)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{QuizOption, QuizQuestion};

    fn option(uid: &str, lemma: &str) -> QuizOption {
        QuizOption {
            sense_uid: uid.to_owned(),
            lemma: lemma.to_owned(),
        }
    }

    #[test]
    fn valid_question_has_four_unique_options_and_one_answer() {
        let question = QuizQuestion {
            question_id: "question-1".to_owned(),
            prompt_zh: "获得（通过努力赢得）".to_owned(),
            correct_sense_uid: "earn.v.02".to_owned(),
            options: [
                option("borrow.v.01", "borrow"),
                option("earn.v.02", "earn"),
                option("escape.v.01", "escape"),
                option("reduce.v.01", "reduce"),
            ],
        };

        assert!(question.has_valid_options());
    }
}
