use chrono::Utc;
use polarbear_vocab_application::{ApplicationError, ArticleRepository};
use polarbear_vocab_domain::ArticleDto;
use rusqlite::params;
use uuid::Uuid;

use crate::{SqliteStore, database_error};

impl ArticleRepository for SqliteStore {
    fn list_articles(&self) -> Result<Vec<ArticleDto>, ApplicationError> {
        let user = self.user()?;
        let mut statement = user
            .prepare(
                "SELECT id, title, body, translated_body, created_at
                 FROM article ORDER BY created_at DESC, id DESC",
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
        };
        self.user()?
            .execute(
                "INSERT INTO article(id, title, body, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![article.id, article.title, article.body, article.created_at],
            )
            .map_err(database_error)?;
        Ok(article)
    }

    fn save_article_translation(
        &self,
        article_id: &str,
        translated_body: &str,
    ) -> Result<(), ApplicationError> {
        let changed = self
            .user()?
            .execute(
                "UPDATE article SET translated_body = ?1 WHERE id = ?2",
                params![translated_body, article_id],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(ApplicationError::NotFound("article".to_owned()));
        }
        Ok(())
    }

    fn delete_article(&self, article_id: &str) -> Result<(), ApplicationError> {
        let changed = self
            .user()?
            .execute("DELETE FROM article WHERE id = ?1", [article_id])
            .map_err(database_error)?;
        if changed == 0 {
            return Err(ApplicationError::NotFound("article".to_owned()));
        }
        Ok(())
    }
}
