use std::sync::Arc;

use polarbear_vocab_application::{
    AppService, ArticleRepository, ArticleService, CsvImportPort, DatasetRepository,
    DatasetService, HomeQueryPort, HomeService, MistakeQueryPort, MistakeService, SettingsPort,
    SettingsService, SpeechPort, SpeechUseCase, StudyPort, StudyService,
};
use polarbear_vocab_dataset_import::CsvDatasetImporter;
use polarbear_vocab_speech::NativeSpeech;
use polarbear_vocab_storage_sqlite::{DatabasePaths, SqliteStore};

pub struct AppRuntime {
    pub app: AppService,
    pub articles: ArticleService,
    pub home: HomeService,
    pub study: StudyService,
    pub mistakes: MistakeService,
    pub datasets: DatasetService,
    pub settings: SettingsService,
    pub speech: SpeechUseCase,
}

impl AppRuntime {
    pub fn open(paths: &DatabasePaths) -> Result<Self, Box<dyn std::error::Error>> {
        let store = Arc::new(SqliteStore::open(paths)?);
        let home_repository: Arc<dyn HomeQueryPort> = store.clone();
        let study_repository: Arc<dyn StudyPort> = store.clone();
        let mistake_repository: Arc<dyn MistakeQueryPort> = store.clone();
        let dataset_repository: Arc<dyn DatasetRepository> = store.clone();
        let settings_repository: Arc<dyn SettingsPort> = store.clone();
        let article_repository: Arc<dyn ArticleRepository> = store.clone();
        let csv_import: Arc<dyn CsvImportPort> = Arc::new(CsvDatasetImporter);
        let speech: Arc<dyn SpeechPort> = Arc::new(NativeSpeech::new()?);
        Ok(Self {
            app: AppService::new(),
            articles: ArticleService::new(article_repository),
            home: HomeService::new(home_repository),
            study: StudyService::new(study_repository),
            mistakes: MistakeService::new(mistake_repository),
            datasets: DatasetService::new(dataset_repository, csv_import),
            settings: SettingsService::new(settings_repository),
            speech: SpeechUseCase::new(speech),
        })
    }
}
