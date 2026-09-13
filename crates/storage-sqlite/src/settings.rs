use chrono::Utc;
use polarbear_vocab_application::{ApplicationError, SettingsPort};
use polarbear_vocab_domain::SettingsDto;
use rusqlite::{OptionalExtension, params};

use crate::{SqliteStore, database_error, schema};

impl SettingsPort for SqliteStore {
    fn get_settings(&self) -> Result<SettingsDto, ApplicationError> {
        let user = self.user()?;
        let language = read_setting(&user, "ui.language")?;
        let theme = read_setting(&user, "ui.theme")?;
        Ok(SettingsDto {
            ui_language: language.unwrap_or_else(|| "system".to_owned()),
            ui_theme: theme.unwrap_or_else(|| "system".to_owned()),
        })
    }

    fn update_settings(&self, settings: &SettingsDto) -> Result<(), ApplicationError> {
        let value_json = serde_json::to_string(&settings.ui_language)
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))?;
        let theme_json = serde_json::to_string(&settings.ui_theme)
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))?;
        let updated_at = Utc::now().timestamp_millis();
        let mut user = self.user()?;
        schema::in_immediate_transaction(&mut user, |transaction| {
            transaction.execute(
                "INSERT INTO setting(key, value_json, updated_at)
                 VALUES ('ui.language', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET
                    value_json = excluded.value_json,
                    updated_at = excluded.updated_at",
                params![value_json, updated_at],
            )?;
            transaction.execute(
                "INSERT INTO setting(key, value_json, updated_at)
                 VALUES ('ui.theme', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET
                    value_json = excluded.value_json,
                    updated_at = excluded.updated_at",
                params![theme_json, updated_at],
            )?;
            Ok(())
        })
        .map_err(database_error)
    }
}

fn read_setting(
    connection: &rusqlite::Connection,
    key: &str,
) -> Result<Option<String>, ApplicationError> {
    connection
        .query_row(
            "SELECT json_extract(value_json, '$') FROM setting WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .map_err(database_error)
}
