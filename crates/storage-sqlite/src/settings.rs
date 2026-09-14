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
        let speech_locale = read_setting(&user, "speech.locale")?;
        let speech_rate_percent = read_setting(&user, "speech.rate_percent")?;
        Ok(SettingsDto {
            speech_locale: speech_locale.unwrap_or_else(|| "en-US".to_owned()),
            speech_rate_percent: speech_rate_percent.unwrap_or(100),
            ui_language: language.unwrap_or_else(|| "system".to_owned()),
            ui_theme: theme.unwrap_or_else(|| "system".to_owned()),
        })
    }

    fn update_settings(&self, settings: &SettingsDto) -> Result<(), ApplicationError> {
        let values = [
            ("ui.language", serde_json::to_string(&settings.ui_language)),
            ("ui.theme", serde_json::to_string(&settings.ui_theme)),
            (
                "speech.locale",
                serde_json::to_string(&settings.speech_locale),
            ),
            (
                "speech.rate_percent",
                serde_json::to_string(&settings.speech_rate_percent),
            ),
        ]
        .map(|(key, value)| {
            value
                .map(|json| (key, json))
                .map_err(|error| ApplicationError::Infrastructure(error.to_string()))
        });
        let values: Result<Vec<_>, _> = values.into_iter().collect();
        let values = values?;
        let updated_at = Utc::now().timestamp_millis();
        let mut user = self.user()?;
        schema::in_immediate_transaction(&mut user, |transaction| {
            for (key, value_json) in &values {
                transaction.execute(
                    "INSERT INTO setting(key, value_json, updated_at)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(key) DO UPDATE SET
                        value_json = excluded.value_json,
                        updated_at = excluded.updated_at",
                    params![key, value_json, updated_at],
                )?;
            }
            Ok(())
        })
        .map_err(database_error)
    }
}

fn read_setting<T: serde::de::DeserializeOwned>(
    connection: &rusqlite::Connection,
    key: &str,
) -> Result<Option<T>, ApplicationError> {
    let value_json: Option<String> = connection
        .query_row(
            "SELECT value_json FROM setting WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .map_err(database_error)?;
    value_json
        .map(|json| {
            serde_json::from_str(&json)
                .map_err(|error| ApplicationError::Infrastructure(error.to_string()))
        })
        .transpose()
}
