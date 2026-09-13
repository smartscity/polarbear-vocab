use chrono::Utc;
use polarbear_vocab_application::{ApplicationError, SettingsPort};
use polarbear_vocab_domain::SettingsDto;
use rusqlite::{OptionalExtension, params};

use crate::{SqliteStore, database_error, schema};

impl SettingsPort for SqliteStore {
    fn get_settings(&self) -> Result<SettingsDto, ApplicationError> {
        let user = self.user()?;
        let language: Option<String> = user
            .query_row(
                "SELECT json_extract(value_json, '$') FROM setting WHERE key = 'ui.language'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(database_error)?;
        Ok(SettingsDto {
            ui_language: language.unwrap_or_else(|| "system".to_owned()),
        })
    }

    fn update_settings(&self, settings: &SettingsDto) -> Result<(), ApplicationError> {
        let value_json = serde_json::to_string(&settings.ui_language)
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))?;
        let mut user = self.user()?;
        schema::in_immediate_transaction(&mut user, |transaction| {
            transaction.execute(
                "INSERT INTO setting(key, value_json, updated_at)
                 VALUES ('ui.language', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET
                    value_json = excluded.value_json,
                    updated_at = excluded.updated_at",
                params![value_json, Utc::now().timestamp_millis()],
            )?;
            Ok(())
        })
        .map_err(database_error)
    }
}
