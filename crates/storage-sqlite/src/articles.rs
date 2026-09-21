use chrono::Utc;
use polarbear_vocab_application::{ApplicationError, ArticleRepository};
use polarbear_vocab_domain::{ArticleDto, BuiltinArticle};
use rusqlite::params;
use uuid::Uuid;

use crate::{SqliteStore, database_error, schema};

impl ArticleRepository for SqliteStore {
    fn list_articles(&self) -> Result<Vec<ArticleDto>, ApplicationError> {
        let user = self.user()?;
        let mut statement = user
            .prepare(
                "SELECT id, title, body, translated_body, created_at,
                    id LIKE 'builtin.%'
                 FROM article
                 ORDER BY id LIKE 'builtin.%' DESC, created_at DESC, id",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(ArticleDto {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    body: row.get(2)?,
                    translated_body: row.get(3)?,
                    created_at: row.get(4)?,
                    builtin: row.get(5)?,
                })
            })
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn save_article(&self, title: &str, body: &str) -> Result<ArticleDto, ApplicationError> {
        let article = ArticleDto {
            id: Uuid::new_v4().to_string(),
            title: title.to_owned(),
            body: body.to_owned(),
            translated_body: None,
            created_at: Utc::now().timestamp_millis(),
            builtin: false,
        };
        let mut user = self.user()?;
        schema::in_immediate_transaction(&mut user, |transaction| {
            transaction.execute(
                "INSERT INTO article(id, title, body, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                params![article.id, article.title, article.body, article.created_at],
            )?;
            schema::record_sync_change(
                transaction,
                "article",
                &article.id,
                "upsert",
                article.created_at,
            )?;
            Ok(())
        })
        .map_err(database_error)?;
        Ok(article)
    }

    fn save_article_translation(
        &self,
        article_id: &str,
        translated_body: &str,
    ) -> Result<(), ApplicationError> {
        if article_id.starts_with("builtin.") {
            return Err(ApplicationError::Conflict(
                "built-in listening packs cannot be changed".to_owned(),
            ));
        }
        let changed_at = Utc::now().timestamp_millis();
        let mut user = self.user()?;
        let changed = schema::in_immediate_transaction(&mut user, |transaction| {
            let changed = transaction.execute(
                "UPDATE article SET translated_body = ?1, updated_at = ?3 WHERE id = ?2",
                params![translated_body, article_id, changed_at],
            )?;
            if changed > 0 {
                schema::record_sync_change(
                    transaction,
                    "article",
                    article_id,
                    "upsert",
                    changed_at,
                )?;
            }
            Ok(changed)
        })
        .map_err(database_error)?;
        if changed == 0 {
            return Err(ApplicationError::NotFound("article".to_owned()));
        }
        Ok(())
    }

    fn delete_article(&self, article_id: &str) -> Result<(), ApplicationError> {
        if article_id.starts_with("builtin.") {
            return Err(ApplicationError::Conflict(
                "built-in listening packs cannot be deleted".to_owned(),
            ));
        }
        let changed_at = Utc::now().timestamp_millis();
        let mut user = self.user()?;
        let changed = schema::in_immediate_transaction(&mut user, |transaction| {
            let changed = transaction.execute("DELETE FROM article WHERE id = ?1", [article_id])?;
            if changed > 0 {
                schema::record_sync_change(
                    transaction,
                    "article",
                    article_id,
                    "delete",
                    changed_at,
                )?;
            }
            Ok(changed)
        })
        .map_err(database_error)?;
        if changed == 0 {
            return Err(ApplicationError::NotFound("article".to_owned()));
        }
        Ok(())
    }
}

impl SqliteStore {
    pub fn ensure_builtin_articles(
        &self,
        articles: &[BuiltinArticle],
    ) -> Result<(), ApplicationError> {
        let mut user = self.user()?;
        schema::in_immediate_transaction(&mut user, |transaction| {
            for article in articles {
                transaction.execute(
                    "INSERT INTO article(
                       id, title, body, translated_body, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, 0, 0)
                     ON CONFLICT(id) DO UPDATE SET title = excluded.title,
                       body = excluded.body, translated_body = excluded.translated_body
                     WHERE article.id LIKE 'builtin.%'",
                    params![
                        article.id,
                        article.title,
                        article.body,
                        article.translated_body
                    ],
                )?;
            }
            Ok(())
        })
        .map_err(database_error)
    }
}
