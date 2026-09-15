use std::sync::Arc;

use polarbear_vocab_domain::{ArticleDto, BackupStatusDto, LexiconEntryDto, RestoreResultDto};

use crate::{ApplicationError, ArticleRepository, BackupRepository, LexiconRepository};

#[derive(Clone)]
pub struct ArticleService {
    repository: Arc<dyn ArticleRepository>,
}

impl ArticleService {
    #[must_use]
    pub fn new(repository: Arc<dyn ArticleRepository>) -> Self {
        Self { repository }
    }

    pub fn list(&self) -> Result<Vec<ArticleDto>, ApplicationError> {
        self.repository.list_articles()
    }

    pub fn import(&self, title: &str, body: &str) -> Result<ArticleDto, ApplicationError> {
        let title = title.trim();
        let body = body.trim();
        if title.is_empty() || title.chars().count() > 160 {
            return Err(ApplicationError::InvalidInput(
                "article title must contain between 1 and 160 characters".to_owned(),
            ));
        }
        if body.is_empty() || body.chars().count() > 100_000 {
            return Err(ApplicationError::InvalidInput(
                "article body must contain between 1 and 100000 characters".to_owned(),
            ));
        }
        self.repository.save_article(title, body)
    }

    pub fn delete(&self, article_id: &str) -> Result<(), ApplicationError> {
        if article_id.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "article id must not be empty".to_owned(),
            ));
        }
        self.repository.delete_article(article_id)
    }
}

#[derive(Clone)]
pub struct LexiconService {
    repository: Arc<dyn LexiconRepository>,
}

impl LexiconService {
    #[must_use]
    pub fn new(repository: Arc<dyn LexiconRepository>) -> Self {
        Self { repository }
    }

    pub fn search(&self, query: &str) -> Result<Vec<LexiconEntryDto>, ApplicationError> {
        let query = query.trim();
        if query.is_empty() || query.chars().count() > 80 {
            return Err(ApplicationError::InvalidInput(
                "search query must contain between 1 and 80 characters".to_owned(),
            ));
        }
        self.repository.search_lexicon(query, 50)
    }

    pub fn add_to_my_vocabulary(&self, sense_uid: &str) -> Result<(), ApplicationError> {
        self.repository.add_to_my_vocabulary(valid_uid(sense_uid)?)
    }

    pub fn remove_from_my_vocabulary(&self, sense_uid: &str) -> Result<(), ApplicationError> {
        self.repository
            .remove_from_my_vocabulary(valid_uid(sense_uid)?)
    }
}

fn valid_uid(value: &str) -> Result<&str, ApplicationError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 {
        return Err(ApplicationError::InvalidInput(
            "sense_uid must contain between 1 and 200 characters".to_owned(),
        ));
    }
    Ok(value)
}

#[derive(Clone)]
pub struct BackupService {
    repository: Arc<dyn BackupRepository>,
}

impl BackupService {
    #[must_use]
    pub fn new(repository: Arc<dyn BackupRepository>) -> Self {
        Self { repository }
    }

    pub fn status(&self) -> Result<BackupStatusDto, ApplicationError> {
        self.repository.backup_status()
    }

    pub fn export(&self, path: &str) -> Result<BackupStatusDto, ApplicationError> {
        validate_backup_path(path)?;
        self.repository
            .export_backup(path, env!("CARGO_PKG_VERSION"))
    }

    pub fn import(&self, path: &str) -> Result<RestoreResultDto, ApplicationError> {
        validate_backup_path(path)?;
        self.repository
            .import_backup(path, env!("CARGO_PKG_VERSION"))
    }
}

fn validate_backup_path(path: &str) -> Result<(), ApplicationError> {
    let path = path.trim();
    if path.is_empty() || !path.ends_with(".polarbear-vocab-backup") {
        return Err(ApplicationError::InvalidInput(
            "backup path must end with .polarbear-vocab-backup".to_owned(),
        ));
    }
    Ok(())
}
