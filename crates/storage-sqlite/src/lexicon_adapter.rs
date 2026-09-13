use polarbear_lexicon::{LexiconReader, SenseSummary};
use polarbear_vocab_application::ApplicationError;

use crate::{SqliteStore, database_error, read_model};

impl LexiconReader for SqliteStore {
    type Error = ApplicationError;

    fn sense(&self, sense_uid: &str) -> Result<Option<SenseSummary>, Self::Error> {
        let content = self.content()?;
        Ok(read_model::sense_detail(&content, sense_uid)
            .map_err(database_error)?
            .map(|detail| SenseSummary {
                sense_uid: detail.sense_uid,
                lemma: detail.lemma,
                part_of_speech: detail.part_of_speech,
                quiz_prompt_zh: detail.prompt_zh,
            }))
    }
}
