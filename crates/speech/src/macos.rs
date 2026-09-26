use std::sync::mpsc::{self, Sender};

use objc2::rc::Retained;
#[cfg(target_os = "ios")]
use objc2_avf_audio::{AVAudioSession, AVAudioSessionCategoryPlayback};
use objc2_avf_audio::{
    AVSpeechBoundary, AVSpeechSynthesisVoice, AVSpeechSynthesisVoiceGender, AVSpeechSynthesizer,
    AVSpeechUtterance, AVSpeechUtteranceDefaultSpeechRate, AVSpeechUtteranceMaximumSpeechRate,
    AVSpeechUtteranceMinimumSpeechRate,
};
use objc2_foundation::NSString;
use polarbear_vocab_application::{ApplicationError, SpeechPort};
use polarbear_vocab_domain::SpeakRequest;

#[derive(Clone, Debug)]
pub struct NativeSpeech {
    sender: Sender<SpeechCommand>,
}

#[derive(Clone, Debug)]
enum SpeechCommand {
    Pause,
    Resume,
    Speak(SpeakRequest),
    Stop,
}

impl NativeSpeech {
    pub fn new() -> Result<Self, ApplicationError> {
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("polarbear-vocab-speech".to_owned())
            .spawn(move || {
                #[cfg(target_os = "ios")]
                configure_audio_session();
                let synthesizer = unsafe { AVSpeechSynthesizer::new() };
                while let Ok(command) = receiver.recv() {
                    match command {
                        SpeechCommand::Pause => unsafe {
                            synthesizer.pauseSpeakingAtBoundary(AVSpeechBoundary::Word);
                        },
                        SpeechCommand::Resume => unsafe {
                            synthesizer.continueSpeaking();
                        },
                        SpeechCommand::Speak(request) => {
                            speak(&synthesizer, &request);
                        }
                        SpeechCommand::Stop => unsafe {
                            synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
                        },
                    }
                }
            })
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))?;
        Ok(Self { sender })
    }
}

#[cfg(target_os = "ios")]
fn configure_audio_session() {
    unsafe {
        let session = AVAudioSession::sharedInstance();
        if let Some(category) = AVAudioSessionCategoryPlayback {
            let _ = session.setCategory_error(category);
        }
        let _ = session.setActive_error(true);
    }
}

impl SpeechPort for NativeSpeech {
    fn pause(&self) -> Result<(), ApplicationError> {
        self.sender
            .send(SpeechCommand::Pause)
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))
    }

    fn resume(&self) -> Result<(), ApplicationError> {
        self.sender
            .send(SpeechCommand::Resume)
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))
    }

    fn speak(&self, request: &SpeakRequest) -> Result<(), ApplicationError> {
        self.sender
            .send(SpeechCommand::Speak(request.clone()))
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))
    }

    fn stop(&self) -> Result<(), ApplicationError> {
        self.sender
            .send(SpeechCommand::Stop)
            .map_err(|error| ApplicationError::Infrastructure(error.to_string()))
    }
}

fn speak(synthesizer: &AVSpeechSynthesizer, request: &SpeakRequest) {
    let requested_locale = request.locale.as_deref().unwrap_or("en-US");
    let locale = locale_for_voice(requested_locale, request.voice.as_deref());
    let gender = match request.voice.as_deref() {
        Some("male") => Some(AVSpeechSynthesisVoiceGender::Male),
        Some("female") => Some(AVSpeechSynthesisVoiceGender::Female),
        _ => None,
    };
    unsafe {
        synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
        let voice = best_available_voice(locale, gender);
        let rate = (AVSpeechUtteranceDefaultSpeechRate * request.rate.unwrap_or(1.0)).clamp(
            AVSpeechUtteranceMinimumSpeechRate,
            AVSpeechUtteranceMaximumSpeechRate,
        );
        for segment in request
            .text
            .split("\n\n")
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            let utterance =
                AVSpeechUtterance::speechUtteranceWithString(&NSString::from_str(segment));
            utterance.setVoice(voice.as_deref());
            utterance.setRate(rate);
            utterance.setPostUtteranceDelay(0.18);
            synthesizer.speakUtterance(&utterance);
        }
    }
}

fn best_available_voice(
    locale: &str,
    gender: Option<AVSpeechSynthesisVoiceGender>,
) -> Option<Retained<AVSpeechSynthesisVoice>> {
    unsafe {
        let voices = AVSpeechSynthesisVoice::speechVoices();
        voices
            .to_vec()
            .into_iter()
            .filter(|voice| {
                let language = voice.language().to_string();
                language == locale || (locale.starts_with("en-") && language.starts_with("en-"))
            })
            .max_by_key(|voice| {
                let language = voice.language().to_string();
                let gender_match = gender.is_none_or(|expected| voice.gender() == expected);
                (
                    language == locale && gender_match,
                    language == locale,
                    gender_match,
                    voice.quality(),
                )
            })
            .or_else(|| {
                AVSpeechSynthesisVoice::voiceWithLanguage(Some(&NSString::from_str(locale)))
            })
    }
}

fn locale_for_voice<'a>(requested_locale: &'a str, voice: Option<&str>) -> &'a str {
    match voice {
        Some("american") => "en-US",
        Some("british") => "en-GB",
        Some("hong-kong") => "en-HK",
        Some("indian") => "en-IN",
        Some("japanese") => "ja-JP",
        _ => requested_locale,
    }
}

#[cfg(test)]
mod tests {
    use super::locale_for_voice;

    #[test]
    fn accent_voices_select_their_system_locale() {
        assert_eq!(locale_for_voice("en-GB", Some("american")), "en-US");
        assert_eq!(locale_for_voice("en-US", Some("british")), "en-GB");
        assert_eq!(locale_for_voice("en-US", Some("hong-kong")), "en-HK");
        assert_eq!(locale_for_voice("en-US", Some("indian")), "en-IN");
        assert_eq!(locale_for_voice("en-US", Some("japanese")), "ja-JP");
        assert_eq!(locale_for_voice("en-GB", Some("female")), "en-GB");
    }
}
