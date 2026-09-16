mod commands;
mod state;

use std::path::PathBuf;

use polarbear_vocab_storage_sqlite::{DatabasePaths, ensure_writable_content};
use tauri::{Manager, path::BaseDirectory};

use crate::state::AppRuntime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let content = content_database_path(app)?;
            let user = app.path().app_data_dir()?.join("user.db");
            app.manage(AppRuntime::open(&DatabasePaths { content, user })?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::list_articles,
            commands::import_article,
            commands::delete_article,
            commands::list_datasets,
            commands::get_home,
            commands::search_lexicon,
            commands::add_to_my_vocabulary,
            commands::remove_from_my_vocabulary,
            commands::get_backup_status,
            commands::export_backup,
            commands::import_backup,
            commands::start_collection,
            commands::next_question,
            commands::get_resumable_session,
            commands::submit_answer,
            commands::finish_session,
            commands::list_wrong_words,
            commands::create_dataset,
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
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Polarbear Vocab");
}

fn content_database_path(
    app: &tauri::App,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(mobile)]
    let seed = app
        .path()
        .resolve("content.db", BaseDirectory::Resource)?;

    #[cfg(all(not(mobile), debug_assertions))]
    let seed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../data/generated/content.db");

    #[cfg(all(not(mobile), not(debug_assertions)))]
    let seed = app
        .path()
        .resolve("content.db", BaseDirectory::Resource)?;

    let destination = app.path().app_data_dir()?.join("content.db");

    Ok(ensure_writable_content(&seed, &destination)?)
}