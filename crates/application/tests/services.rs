use std::sync::{Arc, Mutex};

use polarbear_vocab_application::{
    ApplicationError, CsvImportPort, DatasetRepository, DatasetService, SettingsPort,
    SettingsService, SpeechPort, SpeechUseCase,
};
use polarbear_vocab_domain::{
    CsvImportPreview, CsvImportResult, DatasetImportPlan, DatasetSummary, SettingsDto, SpeakRequest,
};

#[derive(Default)]
struct DatasetDouble {
    names: Mutex<Vec<String>>,
}

impl DatasetRepository for DatasetDouble {
    fn create_dataset(&self, name: &str) -> Result<DatasetSummary, ApplicationError> {
        self.names.lock().unwrap().push(name.to_owned());
        Ok(DatasetSummary {
            id: "id".to_owned(),
            name: name.to_owned(),
            created_at: 0,
            preloaded: false,
            word_count: 0,
        })
    }

    fn rename_dataset(&self, _: &str, name: &str) -> Result<(), ApplicationError> {
        self.names.lock().unwrap().push(name.to_owned());
        Ok(())
    }

    fn delete_dataset(&self, _: &str) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn import_dataset(
        &self,
        _: &str,
        _: &DatasetImportPlan,
    ) -> Result<CsvImportResult, ApplicationError> {
        Ok(CsvImportResult {
            imported_items: 0,
            inserted_senses: 0,
            updated_senses: 0,
        })
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
        ui_language: "zh-CN".to_owned(),
        ui_theme: "dark".to_owned(),
    };

    service.update(&valid).unwrap();

    assert_eq!(*repository.updates.lock().unwrap(), [valid]);
    let invalid_theme = SettingsDto {
        ui_language: "en".to_owned(),
        ui_theme: "sepia".to_owned(),
    };
    assert!(matches!(
        service.update(&invalid_theme),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[derive(Default)]
struct SpeechDouble {
    requests: Mutex<Vec<SpeakRequest>>,
}

impl SpeechPort for SpeechDouble {
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
    };

    service.speak(&valid).unwrap();

    assert_eq!(*speech.requests.lock().unwrap(), [valid]);
    for rate in [0.1, 2.1] {
        let invalid = SpeakRequest {
            text: "word".to_owned(),
            locale: None,
            rate: Some(rate),
        };
        assert!(matches!(
            service.speak(&invalid),
            Err(ApplicationError::InvalidInput(_))
        ));
    }
    assert!(matches!(
        service.speak(&SpeakRequest {
            text: " ".to_owned(),
            locale: None,
            rate: None
        }),
        Err(ApplicationError::InvalidInput(_))
    ));
}
