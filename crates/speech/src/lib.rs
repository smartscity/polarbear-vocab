#[cfg(not(any(target_os = "macos", target_os = "ios")))]
use polarbear_vocab_application::{ApplicationError, SpeechPort};
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
use polarbear_vocab_domain::SpeakRequest;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod macos;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use macos::NativeSpeech;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
#[derive(Clone, Debug, Default)]
pub struct NativeSpeech;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
impl NativeSpeech {
    pub fn new() -> Result<Self, ApplicationError> {
        Ok(Self)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
impl SpeechPort for NativeSpeech {
    fn pause(&self) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn resume(&self) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn speak(&self, _request: &SpeakRequest) -> Result<(), ApplicationError> {
        Err(ApplicationError::Infrastructure(
            "native speech is not implemented on this platform".to_owned(),
        ))
    }

    fn stop(&self) -> Result<(), ApplicationError> {
        Ok(())
    }
}
