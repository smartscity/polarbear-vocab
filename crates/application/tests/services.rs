use std::sync::{Arc, Mutex};

use polarbear_vocab_application::{
    ApplicationError, ArticleRepository, ArticleService, CsvImportPort, DatasetRepository,
    DatasetService, SettingsPort, SettingsService, SpeechPort, SpeechUseCase,
};
use polarbear_vocab_domain::{
    ArticleDto, CsvImportPreview, CsvImportResult, DatasetImportPlan, DatasetImportStrategy,
    DatasetSummary, SettingsDto, SpeakRequest,
};

#[derive(Default)]
struct ArticleDouble {
    saved: Mutex<Vec<(String, String)>>,
}

impl ArticleRepository for ArticleDouble {
    fn list_articles(&self) -> Result<Vec<ArticleDto>, ApplicationError> {
        Ok(Vec::new())
    }

    fn save_article(&self, title: &str, body: &str) -> Result<ArticleDto, ApplicationError> {
        self.saved
            .lock()
            .unwrap()
            .push((title.to_owned(), body.to_owned()));
        Ok(ArticleDto {
            id: "article".to_owned(),
            title: title.to_owned(),
            body: body.to_owned(),
            created_at: 1,
        })
    }

    fn delete_article(&self, _: &str) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[test]
fn article_service_trims_content_and_rejects_invalid_articles() {
    let repository = Arc::new(ArticleDouble::default());
    let service = ArticleService::new(repository.clone());

    service
        .import("  Listening practice  ", "  A short article.  ")
        .unwrap();

    assert_eq!(
        *repository.saved.lock().unwrap(),
        [(
            "Listening practice".to_owned(),
            "A short article.".to_owned()
        )]
    );
    assert!(matches!(
        service.import("Empty", "  "),
        Err(ApplicationError::InvalidInput(_))
    ));
    assert!(matches!(
        service.import("Long", &"x".repeat(100_001)),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[derive(Default)]
struct DatasetDouble {
    names: Mutex<Vec<String>>,
    orders: Mutex<Vec<Vec<String>>>,
}

impl DatasetRepository for DatasetDouble {
    fn create_dataset(&self, name: &str) -> Result<DatasetSummary, ApplicationError> {
        self.names.lock().unwrap().push(name.to_owned());
        Ok(DatasetSummary {
            id: "id".to_owned(),
            name: name.to_owned(),
            created_at: 0,
            updated_at: 0,
            preloaded: false,
            word_count: 0,
        })
    }

    fn rename_dataset(&self, _: &str, name: &str) -> Result<(), ApplicationError> {
        self.names.lock().unwrap().push(name.to_owned());
        Ok(())
    }

    fn reorder_datasets(&self, dataset_ids: &[String]) -> Result<(), ApplicationError> {
        self.orders.lock().unwrap().push(dataset_ids.to_vec());
        Ok(())
    }

    fn delete_dataset(&self, _: &str) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn import_dataset(
        &self,
        _: &str,
        _: &DatasetImportPlan,
        _: DatasetImportStrategy,
    ) -> Result<CsvImportResult, ApplicationError> {
        Ok(CsvImportResult {
            imported_items: 0,
            inserted_senses: 0,
            updated_senses: 0,
        })
    }

    fn export_dataset(&self, _: &str, _: &str) -> Result<u32, ApplicationError> {
        Ok(0)
    }
}

struct ImportDouble;

impl CsvImportPort for ImportDouble {
    fn preview(&self, _: &str) -> Result<CsvImportPreview, ApplicationError> {
        unreachable!()
    }
    fn parse(&self, _: &str) -> Result<DatasetImportPlan, ApplicationError> {
        unreachable!()
    }
}

#[test]
fn dataset_service_trims_valid_names_and_rejects_invalid_names() {
    let repository = Arc::new(DatasetDouble::default());
    let service = DatasetService::new(repository.clone(), Arc::new(ImportDouble));

    service.create("  Travel  ").unwrap();

    assert_eq!(*repository.names.lock().unwrap(), ["Travel"]);
    assert!(matches!(
        service.create("   "),
        Err(ApplicationError::InvalidInput(_))
    ));
    assert!(matches!(
        service.rename("id", &"x".repeat(81)),
        Err(ApplicationError::InvalidInput(_))
    ));
    assert_eq!(repository.names.lock().unwrap().len(), 1);
}

#[test]
fn dataset_service_validates_reorder_ids() {
    let repository = Arc::new(DatasetDouble::default());
    let service = DatasetService::new(repository.clone(), Arc::new(ImportDouble));
    let requested = vec!["second".to_owned(), "first".to_owned()];

    service.reorder(&requested).unwrap();

    assert_eq!(*repository.orders.lock().unwrap(), [requested]);
    assert!(matches!(
        service.reorder(&["same".to_owned(), "same".to_owned()]),
        Err(ApplicationError::InvalidInput(_))
    ));
    assert!(matches!(
        service.reorder(&[]),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[derive(Default)]
struct SettingsDouble {
    updates: Mutex<Vec<SettingsDto>>,
}

impl SettingsPort for SettingsDouble {
    fn get_settings(&self) -> Result<SettingsDto, ApplicationError> {
        Ok(SettingsDto::default())
    }
    fn update_settings(&self, settings: &SettingsDto) -> Result<(), ApplicationError> {
        self.updates.lock().unwrap().push(settings.clone());
        Ok(())
    }
}

#[test]
fn settings_service_accepts_supported_values_only() {
    let repository = Arc::new(SettingsDouble::default());
    let service = SettingsService::new(repository.clone());
    let valid = SettingsDto {
        speech_locale: "en-GB".to_owned(),
        speech_rate_percent: 150,
        speech_voice: "hong-kong".to_owned(),
        ui_language: "zh-CN".to_owned(),
        ui_theme: "dark".to_owned(),
    };

    service.update(&valid).unwrap();

    assert_eq!(*repository.updates.lock().unwrap(), [valid]);
    let invalid_theme = SettingsDto {
        speech_locale: "en-US".to_owned(),
        speech_rate_percent: 100,
        speech_voice: "female".to_owned(),
        ui_language: "en".to_owned(),
        ui_theme: "sepia".to_owned(),
    };
    assert!(matches!(
        service.update(&invalid_theme),
        Err(ApplicationError::InvalidInput(_))
    ));
    let invalid_speech = SettingsDto {
        speech_locale: "fr-FR".to_owned(),
        ..SettingsDto::default()
    };
    assert!(matches!(
        service.update(&invalid_speech),
        Err(ApplicationError::InvalidInput(_))
    ));
    let invalid_rate = SettingsDto {
        speech_rate_percent: 225,
        ..SettingsDto::default()
    };
    assert!(matches!(
        service.update(&invalid_rate),
        Err(ApplicationError::InvalidInput(_))
    ));
    let invalid_voice = SettingsDto {
        speech_voice: "robot".to_owned(),
        ..SettingsDto::default()
    };
    assert!(matches!(
        service.update(&invalid_voice),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[derive(Default)]
struct SpeechDouble {
    requests: Mutex<Vec<SpeakRequest>>,
}

impl SpeechPort for SpeechDouble {
    fn pause(&self) -> Result<(), ApplicationError> {
        Ok(())
    }
    fn resume(&self) -> Result<(), ApplicationError> {
        Ok(())
    }
    fn speak(&self, request: &SpeakRequest) -> Result<(), ApplicationError> {
        self.requests.lock().unwrap().push(request.clone());
        Ok(())
    }
    fn stop(&self) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[test]
fn speech_service_enforces_text_and_rate_boundaries() {
    let speech = Arc::new(SpeechDouble::default());
    let service = SpeechUseCase::new(speech.clone());
    let valid = SpeakRequest {
        text: "word".to_owned(),
        locale: Some("en-US".to_owned()),
        rate: Some(1.0),
        voice: Some("american".to_owned()),
    };

    service.speak(&valid).unwrap();

    assert_eq!(*speech.requests.lock().unwrap(), [valid]);
    for rate in [0.1, 2.1] {
        let invalid = SpeakRequest {
            text: "word".to_owned(),
            locale: None,
            rate: Some(rate),
            voice: None,
        };
        assert!(matches!(
            service.speak(&invalid),
            Err(ApplicationError::InvalidInput(_))
        ));
    }
    assert!(matches!(
        service.speak(&SpeakRequest {
            text: "word".to_owned(),
            locale: Some("fr-FR".to_owned()),
            rate: None,
            voice: None
        }),
        Err(ApplicationError::InvalidInput(_))
    ));
    assert!(matches!(
        service.speak(&SpeakRequest {
            text: "word".to_owned(),
            locale: None,
            rate: None,
            voice: Some("robot".to_owned())
        }),
        Err(ApplicationError::InvalidInput(_))
    ));
    assert!(matches!(
        service.speak(&SpeakRequest {
            text: " ".to_owned(),
            locale: None,
            rate: None,
            voice: None
        }),
        Err(ApplicationError::InvalidInput(_))
    ));
}
