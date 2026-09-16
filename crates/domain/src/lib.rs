use serde::{Deserialize, Serialize};

pub const PRODUCT_NAME: &str = "Polarbear Vocab";
pub const KNOWLEDGE_MODULE_NAME: &str = "Polarbear Lexicon";
pub const SPEECH_VOICES: [&str; 7] = [
    "male",
    "female",
    "american",
    "british",
    "hong-kong",
    "indian",
    "japanese",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub knowledge_module: &'static str,
    pub version: &'static str,
}

impl AppInfo {
    #[must_use]
    pub const fn current(version: &'static str) -> Self {
        Self {
            name: PRODUCT_NAME,
            knowledge_module: KNOWLEDGE_MODULE_NAME,
            version,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CollectionSpec {
    Dataset {
        dataset_id: String,
    },
    Unseen {
        dataset_id: String,
    },
    Answered {
        dataset_id: Option<String>,
    },
    Correct {
        dataset_id: Option<String>,
    },
    Wrong {
        dataset_id: Option<String>,
        min_wrong_count: u32,
    },
    LastWrong {
        dataset_id: Option<String>,
    },
    Custom {
        sense_uids: Vec<String>,
    },
    MyVocabulary,
}

impl CollectionSpec {
    #[must_use]
    pub fn dataset_id(&self) -> Option<&str> {
        match self {
            Self::Dataset { dataset_id } | Self::Unseen { dataset_id } => Some(dataset_id),
            Self::Answered { dataset_id }
            | Self::Correct { dataset_id }
            | Self::Wrong { dataset_id, .. }
            | Self::LastWrong { dataset_id } => dataset_id.as_deref(),
            Self::Custom { .. } | Self::MyVocabulary => None,
        }
    }

    #[must_use]
    pub const fn collection_type(&self) -> &'static str {
        match self {
            Self::Dataset { .. } => "dataset",
            Self::Unseen { .. } => "unseen",
            Self::Answered { .. } => "answered",
            Self::Correct { .. } => "correct",
            Self::Wrong { .. } => "wrong",
            Self::LastWrong { .. } => "last_wrong",
            Self::Custom { .. } => "custom",
            Self::MyVocabulary => "my_vocabulary",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetSummary {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub preloaded: bool,
    pub word_count: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetProgress {
    pub total: u32,
    pub answered: u32,
    pub unseen: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerTotals {
    pub explored: u32,
    pub correct: u32,
    pub mistakes: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MistakeBuckets {
    pub at_least_one: u32,
    pub at_least_two: u32,
    pub at_least_three: u32,
    pub at_least_five: u32,
    pub last_wrong: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyActivity {
    pub local_date: String,
    pub attempt_count: u32,
    pub correct_count: u32,
    pub wrong_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeDto {
    pub dataset: DatasetSummary,
    pub progress: DatasetProgress,
    pub totals: AnswerTotals,
    pub daily_activity: Vec<DailyActivity>,
    pub mistake_buckets: MistakeBuckets,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSession {
    pub collection_id: String,
    pub session_id: String,
    pub dataset_id: Option<String>,
    pub collection_type: String,
    pub total_count: u32,
    pub answered_count: u32,
    pub correct_count: u32,
    pub wrong_count: u32,
    pub new_word_count: u32,
    pub wrong_sense_uids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionOption {
    pub option_id: String,
    pub sense_uid: String,
    pub lemma: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizQuestionDto {
    pub question_id: String,
    pub sense_uid: String,
    pub prompt_zh: String,
    pub options: Vec<QuestionOption>,
    pub answered_count: u32,
    pub total_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerResultDto {
    pub correct: bool,
    pub was_new: bool,
    pub correct_sense_uid: String,
    pub selected_sense_uid: String,
    pub lemma: String,
    pub ipa: String,
    pub zh_gloss: String,
    pub example_en: String,
    pub example_zh: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrongWordDto {
    pub sense_uid: String,
    pub lemma: String,
    pub ipa: String,
    pub zh_gloss: String,
    pub wrong_count: u32,
    pub correct_count: u32,
    pub last_result: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LexiconEntryDto {
    pub sense_uid: String,
    pub lemma: String,
    pub ipa: String,
    pub part_of_speech: String,
    pub zh_gloss: String,
    pub example_en: String,
    pub dataset_names: Vec<String>,
    pub attempt_count: u32,
    pub correct_count: u32,
    pub wrong_count: u32,
    pub in_my_vocabulary: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct BackupManifest {
    pub format: String,
    pub schema_version: u32,
    pub app_version: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatusDto {
    pub last_backup_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResultDto {
    pub automatic_backup_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleDto {
    pub id: String,
    pub title: String,
    pub body: String,
    pub translated_body: Option<String>,
    pub created_at: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakRequest {
    pub text: String,
    pub locale: Option<String>,
    pub rate: Option<f32>,
    pub voice: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportIssue {
    pub row: u32,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportSample {
    pub sense_uid: String,
    pub lemma: String,
    pub part_of_speech: String,
    pub quiz_prompt_zh: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportPreview {
    pub file_name: String,
    pub total_rows: u32,
    pub valid_rows: u32,
    pub issues: Vec<CsvImportIssue>,
    pub sample: Vec<CsvImportSample>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportResult {
    pub imported_items: u32,
    pub inserted_senses: u32,
    pub updated_senses: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatasetImportStrategy {
    AddOnly,
    UpdateExisting,
    ReplaceDataset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetImportPlan {
    pub entries: Vec<ImportedSense>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedSense {
    pub sense_uid: String,
    pub lemma: String,
    pub part_of_speech: String,
    pub quiz_prompt_zh: String,
    pub gloss_zh: String,
    pub ipa_us: String,
    pub ipa_uk: String,
    pub example_en: String,
    pub example_zh: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub speech_locale: String,
    pub speech_rate_percent: u16,
    pub speech_voice: String,
    pub ui_language: String,
    pub ui_theme: String,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            speech_locale: "en-US".to_owned(),
            speech_rate_percent: 100,
            speech_voice: "female".to_owned(),
            ui_language: "system".to_owned(),
            ui_theme: "system".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppInfo, CollectionSpec, KNOWLEDGE_MODULE_NAME, PRODUCT_NAME, SettingsDto};

    #[test]
    fn app_info_uses_canonical_product_names() {
        let info = AppInfo::current("0.1.0");

        assert_eq!(info.name, PRODUCT_NAME);
        assert_eq!(info.knowledge_module, KNOWLEDGE_MODULE_NAME);
        assert_eq!(info.version, "0.1.0");
    }

    #[test]
    fn collection_contract_serializes_camel_case_dataset_id() {
        let json = serde_json::to_value(CollectionSpec::Wrong {
            dataset_id: Some("core".to_owned()),
            min_wrong_count: 3,
        })
        .unwrap();

        assert_eq!(json["type"], "wrong");
        assert_eq!(json["datasetId"], "core");
        assert_eq!(json["minWrongCount"], 3);
        assert!(json.get("dataset_uid").is_none());
    }

    #[test]
    fn settings_default_to_system_preferences() {
        assert_eq!(
            SettingsDto::default(),
            SettingsDto {
                speech_locale: "en-US".to_owned(),
                speech_rate_percent: 100,
                speech_voice: "female".to_owned(),
                ui_language: "system".to_owned(),
                ui_theme: "system".to_owned(),
            }
        );
    }
}
