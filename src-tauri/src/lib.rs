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
            commands::list_datasets,
            commands::get_home,
            commands::start_collection,
            commands::next_question,
            commands::submit_answer,
            commands::finish_session,
            commands::list_wrong_words,
            commands::create_dataset,
            commands::rename_dataset,
            commands::delete_dataset,
            commands::preview_dataset_csv,
            commands::import_dataset_csv,
            commands::get_settings,
            commands::update_settings,
            commands::speak,
            commands::stop_speech,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Polarbear Vocab");
}

fn content_database_path(app: &tauri::App) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let seed = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/generated/content.db")
    } else {
        app.path().resolve("content.db", BaseDirectory::Resource)?
    };
    let destination = app.path().app_data_dir()?.join("content.db");
    Ok(ensure_writable_content(&seed, &destination)?)
}
