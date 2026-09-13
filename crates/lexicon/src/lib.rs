#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SenseSummary {
    pub sense_uid: String,
    pub lemma: String,
    pub part_of_speech: String,
    pub quiz_prompt_zh: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SenseDetail {
    pub sense_uid: String,
    pub lemma: String,
    pub part_of_speech: String,
    pub prompt_zh: String,
    pub zh_gloss: String,
    pub ipa: String,
    pub example_en: String,
    pub example_zh: String,
}

pub trait LexiconReader {
    type Error;

    fn sense(&self, sense_uid: &str) -> Result<Option<SenseSummary>, Self::Error>;
}
