use polarbear_vocab_domain::ImportedSense;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncManifest {
    pub format: String,
    pub schema_version: u32,
    pub app_version: String,
    pub package_id: String,
    pub source_device_id: String,
    pub created_at: String,
    pub content_cursor: i64,
    pub user_cursor: i64,
    pub acknowledgements: Vec<SyncAcknowledgement>,
    pub content_sha256: String,
    pub user_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAcknowledgement {
    pub device_id: String,
    pub content_cursor: i64,
    pub user_cursor: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSyncPayload {
    pub datasets: Vec<SyncDataset>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDataset {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: i64,
    pub deleted: bool,
    pub entries: Vec<ImportedSense>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSyncPayload {
    pub articles: Vec<SyncArticle>,
    pub review_events: Vec<SyncReviewEvent>,
    pub vocabulary: Vec<SyncVocabulary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncArticle {
    pub id: String,
    pub title: String,
    pub body: String,
    pub translated_body: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: i64,
    pub deleted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReviewEvent {
    pub id: String,
    pub session_id: String,
    pub dataset_id: Option<String>,
    pub sense_uid: String,
    pub collection_type: String,
    pub answered_at: i64,
    pub correct: bool,
    pub selected_sense_uid: Option<String>,
    pub latency_ms: Option<u32>,
    pub options_json: String,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncVocabulary {
    pub sense_uid: String,
    pub added_at: i64,
    pub changed_at: i64,
    pub revision: i64,
    pub deleted: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SyncCursor {
    pub content: i64,
    pub user: i64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ApplyCounts {
    pub datasets: u32,
    pub articles: u32,
    pub review_events: u32,
    pub conflicts: u32,
}
