use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    pub source: Source,
    pub datasets: Vec<Dataset>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Source {
    pub name: String,
    pub license: String,
    pub url: Option<String>,
    pub attribution: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Dataset {
    #[serde(alias = "uid")]
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Entry {
    pub sense_uid: String,
    pub lemma: String,
    pub pos: String,
    pub quiz_prompt_zh: String,
    pub zh_gloss: String,
    pub definition_en: String,
    pub cefr: String,
    pub ipa_us: String,
    pub ipa_uk: String,
    pub example_en: String,
    pub example_zh: String,
    pub datasets: String,
}

impl Entry {
    pub fn dataset_ids(&self) -> impl Iterator<Item = &str> {
        self.datasets
            .split('|')
            .map(str::trim)
            .filter(|uid| !uid.is_empty())
    }
}
