use std::collections::HashMap;

use chrono::Utc;
use polarbear_vocab_application::ApplicationError;
use polarbear_vocab_domain::DatasetSummary;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{database_error, schema};

const DATASET_ORDER_KEY: &str = "dataset.order";

pub fn apply(datasets: &mut [DatasetSummary], order: &[String]) {
    let positions: HashMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect();
    datasets.sort_by_key(|dataset| {
        positions
            .get(dataset.id.as_str())
            .copied()
            .unwrap_or(usize::MAX)
    });
}

pub fn read(connection: &Connection) -> Result<Vec<String>, ApplicationError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value_json FROM setting WHERE key = ?1",
            [DATASET_ORDER_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(database_error)?;
    value
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map(|order| order.unwrap_or_default())
        .map_err(|error| ApplicationError::Infrastructure(error.to_string()))
}

pub fn write(connection: &mut Connection, order: &[String]) -> Result<(), ApplicationError> {
    let value = serde_json::to_string(order)
        .map_err(|error| ApplicationError::Infrastructure(error.to_string()))?;
    schema::in_immediate_transaction(connection, |transaction| {
        transaction.execute(
            "INSERT INTO setting(key, value_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json, updated_at = excluded.updated_at",
            params![DATASET_ORDER_KEY, value, Utc::now().timestamp_millis()],
        )?;
        Ok(())
    })
    .map_err(database_error)
}
