use polarbear_vocab_domain::{
    AnswerResultDto, AppInfo, ArticleDto, BackupStatusDto, CollectionSession, CollectionSpec,
    CsvImportPreview, CsvImportResult, DatasetImportStrategy, DatasetSummary, HomeDto,
    LexiconEntryDto, QuizQuestionDto, RestoreResultDto, SettingsDto, SpeakRequest, WrongWordDto,
};
use tauri::State;

use crate::{local_path, state::AppRuntime};

#[tauri::command]
pub fn get_app_info(runtime: State<'_, AppRuntime>) -> AppInfo {
    runtime.app.app_info()
}

#[tauri::command]
pub fn list_datasets(runtime: State<'_, AppRuntime>) -> Result<Vec<DatasetSummary>, String> {
    runtime
        .home
        .list_datasets()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_home(runtime: State<'_, AppRuntime>, dataset_id: String) -> Result<HomeDto, String> {
    runtime
        .home
        .get_home(&dataset_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn start_collection(
    runtime: State<'_, AppRuntime>,
    spec: CollectionSpec,
    limit: Option<u32>,
) -> Result<CollectionSession, String> {
    runtime
        .study
        .start_collection(&spec, limit)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn next_question(
    runtime: State<'_, AppRuntime>,
    collection_id: String,
) -> Result<Option<QuizQuestionDto>, String> {
    runtime
        .study
        .next_question(&collection_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_resumable_session(
    runtime: State<'_, AppRuntime>,
) -> Result<Option<CollectionSession>, String> {
    runtime
        .study
        .resumable_session()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn submit_answer(
    runtime: State<'_, AppRuntime>,
    collection_id: String,
    question_id: String,
    selected_option_id: String,
    latency_ms: Option<u32>,
) -> Result<AnswerResultDto, String> {
    runtime
        .study
        .submit_answer(
            &collection_id,
            &question_id,
            &selected_option_id,
            latency_ms,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn finish_session(runtime: State<'_, AppRuntime>, session_id: String) -> Result<(), String> {
    runtime
        .study
        .finish_session(&session_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_wrong_words(
    runtime: State<'_, AppRuntime>,
    dataset_id: Option<String>,
    min_wrong_count: Option<u32>,
    last_wrong_only: Option<bool>,
) -> Result<Vec<WrongWordDto>, String> {
    runtime
        .mistakes
        .list_wrong_words(
            dataset_id.as_deref(),
            min_wrong_count.unwrap_or(1),
            last_wrong_only.unwrap_or(false),
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn search_lexicon(
    runtime: State<'_, AppRuntime>,
    query: String,
) -> Result<Vec<LexiconEntryDto>, String> {
    runtime
        .lexicon
        .search(&query)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_to_my_vocabulary(
    runtime: State<'_, AppRuntime>,
    sense_uid: String,
) -> Result<(), String> {
    runtime
        .lexicon
        .add_to_my_vocabulary(&sense_uid)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn remove_from_my_vocabulary(
    runtime: State<'_, AppRuntime>,
    sense_uid: String,
) -> Result<(), String> {
    runtime
        .lexicon
        .remove_from_my_vocabulary(&sense_uid)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_backup_status(runtime: State<'_, AppRuntime>) -> Result<BackupStatusDto, String> {
    runtime.backup.status().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn export_backup(
    runtime: State<'_, AppRuntime>,
    path: String,
) -> Result<BackupStatusDto, String> {
    let path = local_path::string_from_dialog(&path)?;
    runtime
        .backup
        .export(&path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_backup(
    runtime: State<'_, AppRuntime>,
    path: String,
) -> Result<RestoreResultDto, String> {
    let path = local_path::string_from_dialog(&path)?;
    runtime
        .backup
        .import(&path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_dataset(
    runtime: State<'_, AppRuntime>,
    name: String,
) -> Result<DatasetSummary, String> {
    runtime
        .datasets
        .create(&name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rename_dataset(
    runtime: State<'_, AppRuntime>,
    dataset_id: String,
    name: String,
) -> Result<(), String> {
    runtime
        .datasets
        .rename(&dataset_id, &name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn reorder_datasets(
    runtime: State<'_, AppRuntime>,
    dataset_ids: Vec<String>,
) -> Result<(), String> {
    runtime
        .datasets
        .reorder(&dataset_ids)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_dataset(runtime: State<'_, AppRuntime>, dataset_id: String) -> Result<(), String> {
    runtime
        .datasets
        .delete(&dataset_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn preview_dataset_csv(
    runtime: State<'_, AppRuntime>,
    path: String,
) -> Result<CsvImportPreview, String> {
    let path = local_path::string_from_dialog(&path)?;
    runtime
        .datasets
        .preview_csv(&path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_dataset_csv(
    runtime: State<'_, AppRuntime>,
    dataset_id: String,
    path: String,
    strategy: DatasetImportStrategy,
) -> Result<CsvImportResult, String> {
    let path = local_path::string_from_dialog(&path)?;
    runtime
        .datasets
        .import_csv(&dataset_id, &path, strategy)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn export_dataset_csv(
    runtime: State<'_, AppRuntime>,
    dataset_id: String,
    path: String,
) -> Result<u32, String> {
    let path = local_path::string_from_dialog(&path)?;
    runtime
        .datasets
        .export_csv(&dataset_id, &path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_settings(runtime: State<'_, AppRuntime>) -> Result<SettingsDto, String> {
    runtime.settings.get().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_settings(
    runtime: State<'_, AppRuntime>,
    settings: SettingsDto,
) -> Result<(), String> {
    runtime
        .settings
        .update(&settings)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_articles(runtime: State<'_, AppRuntime>) -> Result<Vec<ArticleDto>, String> {
    runtime.articles.list().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_article(runtime: State<'_, AppRuntime>, path: String) -> Result<ArticleDto, String> {
    let normalized = local_path::from_dialog(&path)?;
    let source = normalized.as_path();
    let supported = source
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "txt" | "md"));
    if !supported {
        return Err("invalid input: article must be a UTF-8 .txt or .md file".to_owned());
    }
    let title = source
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Imported article");
    let body =
        std::fs::read_to_string(source).map_err(|error| format!("cannot read article: {error}"))?;
    runtime
        .articles
        .import(title, &body)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_article(runtime: State<'_, AppRuntime>, article_id: String) -> Result<(), String> {
    runtime
        .articles
        .delete(&article_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_article_translation(
    runtime: State<'_, AppRuntime>,
    article_id: String,
    translated_body: String,
) -> Result<(), String> {
    runtime
        .articles
        .save_translation(&article_id, &translated_body)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn speak(runtime: State<'_, AppRuntime>, request: SpeakRequest) -> Result<(), String> {
    runtime
        .speech
        .speak(&request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn pause_speech(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    runtime.speech.pause().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn resume_speech(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    runtime.speech.resume().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn stop_speech(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    runtime.speech.stop().map_err(|error| error.to_string())
}
