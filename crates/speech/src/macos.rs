use std::sync::mpsc::{self, Sender};

use objc2_avf_audio::{
    AVSpeechBoundary, AVSpeechSynthesisVoice, AVSpeechSynthesisVoiceGender, AVSpeechSynthesizer,
    AVSpeechUtterance,
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
    let text = NSString::from_str(request.text.trim());
    let requested_locale = request.locale.as_deref().unwrap_or("en-US");
    let (locale, gender) = match request.voice.as_deref() {
        Some("indian") => ("en-IN", None),
        Some("japanese") => ("ja-JP", None),
        Some("male") => (requested_locale, Some(AVSpeechSynthesisVoiceGender::Male)),
        Some("female") => (requested_locale, Some(AVSpeechSynthesisVoiceGender::Female)),
        _ => (requested_locale, None),
    };
    let locale = NSString::from_str(locale);
    unsafe {
        synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
        let utterance = AVSpeechUtterance::speechUtteranceWithString(&text);
        let fallback = || AVSpeechSynthesisVoice::voiceWithLanguage(Some(&locale));
        let voice = gender
            .and_then(|expected| {
                let voices = AVSpeechSynthesisVoice::speechVoices().to_vec();
                voices
                    .iter()
                    .find(|voice| {
                        voice.gender() == expected
                            && voice.language().to_string() == requested_locale
                    })
                    .cloned()
                    .or_else(|| {
                        voices.into_iter().find(|voice| {
                            voice.gender() == expected
                                && voice.language().to_string().starts_with("en-")
                        })
                    })
            })
            .or_else(fallback);
        utterance.setVoice(voice.as_deref());
        utterance.setRate(request.rate.unwrap_or(1.0) * 0.5);
        synthesizer.speakUtterance(&utterance);
    }
}
