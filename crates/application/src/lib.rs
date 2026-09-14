use std::sync::Arc;

use polarbear_vocab_domain::{
    AnswerResultDto, AppInfo, ArticleDto, CollectionSession, CollectionSpec, CsvImportPreview,
    CsvImportResult, DatasetImportPlan, DatasetSummary, HomeDto, QuizQuestionDto, SettingsDto,
    SpeakRequest, WrongWordDto,
};
use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ApplicationError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("infrastructure failure: {0}")]
    Infrastructure(String),
}

pub trait HomeQueryPort: Send + Sync {
    fn list_datasets(&self) -> Result<Vec<DatasetSummary>, ApplicationError>;
    fn get_home(&self, dataset_id: &str) -> Result<HomeDto, ApplicationError>;
}

pub trait StudyPort: Send + Sync {
    fn start_collection(
        &self,
        spec: &CollectionSpec,
    ) -> Result<CollectionSession, ApplicationError>;

    fn next_question(
        &self,
        collection_id: &str,
    ) -> Result<Option<QuizQuestionDto>, ApplicationError>;

    fn submit_answer(
        &self,
        collection_id: &str,
        question_id: &str,
        selected_option_id: &str,
        latency_ms: Option<u32>,
    ) -> Result<AnswerResultDto, ApplicationError>;

    fn finish_session(&self, session_id: &str) -> Result<(), ApplicationError>;
}

pub trait MistakeQueryPort: Send + Sync {
    fn list_wrong_words(
        &self,
        dataset_id: Option<&str>,
        min_wrong_count: u32,
        last_wrong_only: bool,
    ) -> Result<Vec<WrongWordDto>, ApplicationError>;
}

pub trait DatasetRepository: Send + Sync {
    fn create_dataset(&self, name: &str) -> Result<DatasetSummary, ApplicationError>;
    fn rename_dataset(&self, dataset_id: &str, name: &str) -> Result<(), ApplicationError>;
    fn delete_dataset(&self, dataset_id: &str) -> Result<(), ApplicationError>;
    fn import_dataset(
        &self,
        dataset_id: &str,
        plan: &DatasetImportPlan,
    ) -> Result<CsvImportResult, ApplicationError>;
}

pub trait CsvImportPort: Send + Sync {
    fn preview(&self, path: &str) -> Result<CsvImportPreview, ApplicationError>;
    fn parse(&self, path: &str) -> Result<DatasetImportPlan, ApplicationError>;
}

pub trait SettingsPort: Send + Sync {
    fn get_settings(&self) -> Result<SettingsDto, ApplicationError>;
    fn update_settings(&self, settings: &SettingsDto) -> Result<(), ApplicationError>;
}

pub trait ArticleRepository: Send + Sync {
    fn list_articles(&self) -> Result<Vec<ArticleDto>, ApplicationError>;
    fn save_article(&self, title: &str, body: &str) -> Result<ArticleDto, ApplicationError>;
    fn delete_article(&self, article_id: &str) -> Result<(), ApplicationError>;
}

pub trait SpeechPort: Send + Sync {
    fn pause(&self) -> Result<(), ApplicationError>;
    fn resume(&self) -> Result<(), ApplicationError>;
    fn speak(&self, request: &SpeakRequest) -> Result<(), ApplicationError>;
    fn stop(&self) -> Result<(), ApplicationError>;
}

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

#[derive(Clone, Debug, Default)]
pub struct AppService;

impl AppService {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub const fn app_info(&self) -> AppInfo {
        AppInfo::current(env!("CARGO_PKG_VERSION"))
    }
}

#[derive(Clone)]
pub struct HomeService {
    repository: Arc<dyn HomeQueryPort>,
}

impl HomeService {
    #[must_use]
    pub fn new(repository: Arc<dyn HomeQueryPort>) -> Self {
        Self { repository }
    }

    pub fn list_datasets(&self) -> Result<Vec<DatasetSummary>, ApplicationError> {
        self.repository.list_datasets()
    }

    pub fn get_home(&self, dataset_id: &str) -> Result<HomeDto, ApplicationError> {
        if dataset_id.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "dataset_id must not be empty".to_owned(),
            ));
        }
        self.repository.get_home(dataset_id)
    }
}

#[derive(Clone)]
pub struct StudyService {
    repository: Arc<dyn StudyPort>,
}

impl StudyService {
    #[must_use]
    pub fn new(repository: Arc<dyn StudyPort>) -> Self {
        Self { repository }
    }

    pub fn start_collection(
        &self,
        spec: &CollectionSpec,
    ) -> Result<CollectionSession, ApplicationError> {
        if matches!(
            spec,
            CollectionSpec::Wrong {
                min_wrong_count: 0,
                ..
            }
        ) {
            return Err(ApplicationError::InvalidInput(
                "min_wrong_count must be at least one".to_owned(),
            ));
        }
        self.repository.start_collection(spec)
    }

    pub fn next_question(
        &self,
        collection_id: &str,
    ) -> Result<Option<QuizQuestionDto>, ApplicationError> {
        self.repository.next_question(collection_id)
    }

    pub fn submit_answer(
        &self,
        collection_id: &str,
        question_id: &str,
        selected_option_id: &str,
        latency_ms: Option<u32>,
    ) -> Result<AnswerResultDto, ApplicationError> {
        self.repository
            .submit_answer(collection_id, question_id, selected_option_id, latency_ms)
    }

    pub fn finish_session(&self, session_id: &str) -> Result<(), ApplicationError> {
        self.repository.finish_session(session_id)
    }
}

#[derive(Clone)]
pub struct MistakeService {
    repository: Arc<dyn MistakeQueryPort>,
}

impl MistakeService {
    #[must_use]
    pub fn new(repository: Arc<dyn MistakeQueryPort>) -> Self {
        Self { repository }
    }

    pub fn list_wrong_words(
        &self,
        dataset_id: Option<&str>,
        min_wrong_count: u32,
        last_wrong_only: bool,
    ) -> Result<Vec<WrongWordDto>, ApplicationError> {
        self.repository
            .list_wrong_words(dataset_id, min_wrong_count.max(1), last_wrong_only)
    }
}

#[derive(Clone)]
pub struct DatasetService {
    repository: Arc<dyn DatasetRepository>,
    csv_import: Arc<dyn CsvImportPort>,
}

impl DatasetService {
    #[must_use]
    pub fn new(repository: Arc<dyn DatasetRepository>, csv_import: Arc<dyn CsvImportPort>) -> Self {
        Self {
            repository,
            csv_import,
        }
    }

    pub fn create(&self, name: &str) -> Result<DatasetSummary, ApplicationError> {
        self.repository.create_dataset(validate_dataset_name(name)?)
    }

    pub fn rename(&self, dataset_id: &str, name: &str) -> Result<(), ApplicationError> {
        self.repository
            .rename_dataset(dataset_id, validate_dataset_name(name)?)
    }

    pub fn delete(&self, dataset_id: &str) -> Result<(), ApplicationError> {
        self.repository.delete_dataset(dataset_id)
    }

    pub fn preview_csv(&self, path: &str) -> Result<CsvImportPreview, ApplicationError> {
        self.csv_import.preview(path)
    }

    pub fn import_csv(
        &self,
        dataset_id: &str,
        path: &str,
    ) -> Result<CsvImportResult, ApplicationError> {
        let plan = self.csv_import.parse(path)?;
        self.repository.import_dataset(dataset_id, &plan)
    }
}

fn validate_dataset_name(name: &str) -> Result<&str, ApplicationError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(ApplicationError::InvalidInput(
            "dataset name must contain between 1 and 80 characters".to_owned(),
        ));
    }
    Ok(name)
}

#[derive(Clone)]
pub struct SettingsService {
    repository: Arc<dyn SettingsPort>,
}

impl SettingsService {
    #[must_use]
    pub fn new(repository: Arc<dyn SettingsPort>) -> Self {
        Self { repository }
    }

    pub fn get(&self) -> Result<SettingsDto, ApplicationError> {
        self.repository.get_settings()
    }

    pub fn update(&self, settings: &SettingsDto) -> Result<(), ApplicationError> {
        if !["system", "en", "zh-CN"].contains(&settings.ui_language.as_str()) {
            return Err(ApplicationError::InvalidInput(
                "unsupported UI language".to_owned(),
            ));
        }
        if !["system", "light", "dark"].contains(&settings.ui_theme.as_str()) {
            return Err(ApplicationError::InvalidInput(
                "unsupported UI theme".to_owned(),
            ));
        }
        if !["en-US", "en-GB"].contains(&settings.speech_locale.as_str()) {
            return Err(ApplicationError::InvalidInput(
                "unsupported speech locale".to_owned(),
            ));
        }
        if ![50, 100, 150, 200].contains(&settings.speech_rate_percent) {
            return Err(ApplicationError::InvalidInput(
                "speech rate must be 50, 100, 150, or 200 percent".to_owned(),
            ));
        }
        if !["male", "female", "indian", "japanese"].contains(&settings.speech_voice.as_str()) {
            return Err(ApplicationError::InvalidInput(
                "unsupported speech voice".to_owned(),
            ));
        }
        self.repository.update_settings(settings)
    }
}

#[derive(Clone)]
pub struct SpeechUseCase {
    speech: Arc<dyn SpeechPort>,
}

impl SpeechUseCase {
    #[must_use]
    pub fn new(speech: Arc<dyn SpeechPort>) -> Self {
        Self { speech }
    }

    pub fn speak(&self, request: &SpeakRequest) -> Result<(), ApplicationError> {
        let text = request.text.trim();
        if text.is_empty() || text.chars().count() > 100_000 {
            return Err(ApplicationError::InvalidInput(
                "speech text must contain between 1 and 100000 characters".to_owned(),
            ));
        }
        if request
            .rate
            .is_some_and(|rate| !(0.2..=2.0).contains(&rate))
        {
            return Err(ApplicationError::InvalidInput(
                "speech rate must be between 0.2 and 2.0".to_owned(),
            ));
        }
        if request
            .locale
            .as_deref()
            .is_some_and(|locale| !["en-US", "en-GB"].contains(&locale))
        {
            return Err(ApplicationError::InvalidInput(
                "unsupported speech locale".to_owned(),
            ));
        }
        if request
            .voice
            .as_deref()
            .is_some_and(|voice| !["male", "female", "indian", "japanese"].contains(&voice))
        {
            return Err(ApplicationError::InvalidInput(
                "unsupported speech voice".to_owned(),
            ));
        }
        self.speech.speak(request)
    }

    pub fn pause(&self) -> Result<(), ApplicationError> {
        self.speech.pause()
    }

    pub fn resume(&self) -> Result<(), ApplicationError> {
        self.speech.resume()
    }

    pub fn stop(&self) -> Result<(), ApplicationError> {
        self.speech.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::AppService;

    #[test]
    fn app_info_is_available_without_infrastructure() {
        let service = AppService::new();

        assert_eq!(service.app_info().name, "Polarbear Vocab");
    }
}
