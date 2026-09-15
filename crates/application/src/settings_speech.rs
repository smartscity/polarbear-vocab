use std::sync::Arc;

use polarbear_vocab_domain::{SPEECH_VOICES, SettingsDto, SpeakRequest};

use crate::{ApplicationError, SettingsPort, SpeechPort};

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
        validate_settings(settings)?;
        self.repository.update_settings(settings)
    }
}

fn validate_settings(settings: &SettingsDto) -> Result<(), ApplicationError> {
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
    if !SPEECH_VOICES.contains(&settings.speech_voice.as_str()) {
        return Err(ApplicationError::InvalidInput(
            "unsupported speech voice".to_owned(),
        ));
    }
    Ok(())
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
        validate_speech(request)?;
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

fn validate_speech(request: &SpeakRequest) -> Result<(), ApplicationError> {
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
        .is_some_and(|voice| !SPEECH_VOICES.contains(&voice))
    {
        return Err(ApplicationError::InvalidInput(
            "unsupported speech voice".to_owned(),
        ));
    }
    Ok(())
}
