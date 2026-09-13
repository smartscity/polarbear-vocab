#[cfg(not(target_os = "macos"))]
use polarbear_vocab_application::{ApplicationError, SpeechPort};
#[cfg(not(target_os = "macos"))]
use polarbear_vocab_domain::SpeakRequest;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::NativeSpeech;

#[cfg(not(target_os = "macos"))]
#[derive(Clone, Debug, Default)]
pub struct NativeSpeech;

#[cfg(not(target_os = "macos"))]
impl NativeSpeech {
    pub fn new() -> Result<Self, ApplicationError> {
        Ok(Self)
    }
}

#[cfg(not(target_os = "macos"))]
impl SpeechPort for NativeSpeech {
    fn speak(&self, _request: &SpeakRequest) -> Result<(), ApplicationError> {
        Err(ApplicationError::Infrastructure(
            "native speech is not implemented on this platform".to_owned(),
        ))
    }

    fn stop(&self) -> Result<(), ApplicationError> {
        Ok(())
    }
}
