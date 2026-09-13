use std::sync::mpsc::{self, Sender};

use objc2_avf_audio::{
    AVSpeechBoundary, AVSpeechSynthesisVoice, AVSpeechSynthesizer, AVSpeechUtterance,
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
    let locale = NSString::from_str(request.locale.as_deref().unwrap_or("en-US"));
    unsafe {
        synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
        let utterance = AVSpeechUtterance::speechUtteranceWithString(&text);
        let voice = AVSpeechSynthesisVoice::voiceWithLanguage(Some(&locale));
        utterance.setVoice(voice.as_deref());
        utterance.setRate(request.rate.unwrap_or(1.0) * 0.5);
        synthesizer.speakUtterance(&utterance);
    }
}
