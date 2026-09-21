mod builtin_listening;
mod commands;
mod desktop_integration;
mod local_path;
mod state;

use std::path::PathBuf;

use polarbear_vocab_storage_sqlite::{DatabasePaths, ensure_writable_content};
use tauri::Manager;

use crate::state::AppRuntime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let content = content_database_path(app)?;
            let user = app.path().app_data_dir()?.join("user.db");
            let articles = builtin_listening::load()?;
            let runtime = AppRuntime::open(&DatabasePaths { content, user }, &articles)?;
            if let Err(error) = runtime.backup.ensure_automatic() {
                eprintln!("automatic backup failed: {error}");
            }
            app.manage(runtime);
            desktop_integration::setup(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::list_articles,
            commands::import_article,
            commands::save_article_translation,
            commands::delete_article,
            commands::list_datasets,
            commands::get_home,
            commands::search_lexicon,
            commands::add_to_my_vocabulary,
            commands::remove_from_my_vocabulary,
            commands::get_backup_status,
            commands::ensure_automatic_backup,
            commands::list_backup_versions,
            commands::export_backup,
            commands::import_backup,
            commands::restore_backup_version,
            commands::get_sync_status,
            commands::export_sync,
            commands::import_sync,
            commands::start_collection,
            commands::next_question,
            commands::get_resumable_session,
            commands::submit_answer,
            commands::finish_session,
            commands::list_wrong_words,
            commands::create_dataset,
            commands::create_vocabulary_dataset,
            commands::rename_dataset,
            commands::reorder_datasets,
            commands::delete_dataset,
            commands::preview_dataset_csv,
            commands::import_dataset_csv,
            commands::export_dataset_csv,
            commands::get_settings,
            commands::update_settings,
            commands::speak,
            commands::pause_speech,
            commands::resume_speech,
            commands::stop_speech,
            commands::take_macos_service_requests,
        ])
        .on_window_event(desktop_integration::handle_window_event)
        .run(tauri::generate_context!())
        .expect("failed to run Polarbear Vocab");
}

fn content_database_path(app: &tauri::App) -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(mobile)]
    let seed = app
        .path()
        .resolve("content.db", tauri::path::BaseDirectory::Resource)?;

    #[cfg(all(not(mobile), debug_assertions))]
    let seed = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/generated/content.db");

    #[cfg(all(not(mobile), not(debug_assertions)))]
    let seed = app
        .path()
        .resolve("content.db", tauri::path::BaseDirectory::Resource)?;

    let destination = app.path().app_data_dir()?.join("content.db");

    Ok(ensure_writable_content(&seed, &destination)?)
}
