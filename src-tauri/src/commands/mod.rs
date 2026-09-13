use polarbear_vocab_domain::{
    AnswerResultDto, AppInfo, CollectionSession, CollectionSpec, CsvImportPreview, CsvImportResult,
    DatasetSummary, HomeDto, QuizQuestionDto, SettingsDto, SpeakRequest, WrongWordDto,
};
use tauri::State;

use crate::state::AppRuntime;

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
) -> Result<CollectionSession, String> {
    runtime
        .study
        .start_collection(&spec)
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
) -> Result<CsvImportResult, String> {
    runtime
        .datasets
        .import_csv(&dataset_id, &path)
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
pub fn speak(runtime: State<'_, AppRuntime>, request: SpeakRequest) -> Result<(), String> {
    runtime
        .speech
        .speak(&request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn stop_speech(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    runtime.speech.stop().map_err(|error| error.to_string())
}
