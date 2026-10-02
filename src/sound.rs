use rodio::{OutputStream, Sink};
use std::{
    io::Cursor,
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
};

pub const BUILTIN_SOUND: &str = ":builtin:";
const ALERT: &[u8] = include_bytes!("../assets/alert.wav");
pub struct SoundPlayer {
    sender: Sender<SoundCommand>,
}
enum SoundCommand {
    Play(String),
    Stop,
}
impl SoundPlayer {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            // Keep the output stream alive; open audio only when an alert fires.
            let mut output = None;
            let mut current_sink: Option<Sink> = None;
            while let Ok(cmd) = receiver.recv() {
                match cmd {
                    SoundCommand::Play(path) => {
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }
                        if output.is_none() {
                            match OutputStream::try_default() {
                                Ok(stream) => output = Some(stream),
                                Err(e) => {
                                    eprintln!("Unable to play alert: {e}");
                                    continue;
                                }
                            }
                        }
                        let bytes = if path == BUILTIN_SOUND {
                            Ok(ALERT.to_vec())
                        } else {
                            std::fs::read(&path)
                        };
                        let bytes = match bytes {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                eprintln!("Unable to read alert: {e}");
                                continue;
                            }
                        };
                        match rodio::Decoder::new(Cursor::new(bytes)) {
                            Ok(source) => match Sink::try_new(&output.as_ref().unwrap().1) {
                                Ok(sink) => {
                                    sink.append(source);
                                    current_sink = Some(sink);
                                }
                                Err(e) => eprintln!("Unable to play alert: {e}"),
                            },
                            Err(e) => eprintln!("Unable to decode alert: {e}"),
                        }
                    }
                    SoundCommand::Stop => {
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }
                    }
                }
            }
        });
        Self { sender }
    }
    pub fn play(&self, path: &str) {
        let _ = self.sender.send(SoundCommand::Play(path.to_string()));
    }
    pub fn stop(&self) {
        let _ = self.sender.send(SoundCommand::Stop);
    }
}
pub fn get_sound_path(name: &str) -> Option<PathBuf> {
    let path = if let Some(tail) = name.strip_prefix("~/") {
        PathBuf::from(std::env::var_os("HOME")?).join(tail)
    } else {
        PathBuf::from(name)
    };
    if path.is_file() {
        return Some(path);
    }
    let dir = crate::config::config_dir()?.join("sounds");
    for ext in ["", ".mp3", ".wav", ".ogg", ".flac"] {
        let path = dir.join(format!("{name}{ext}"));
        if path.is_file() {
            return Some(path);
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    use rodio::Source;
    #[test]
    fn bundled_alert_decodes_without_external_files() {
        let sound = rodio::Decoder::new(Cursor::new(ALERT)).unwrap();
        assert_eq!(sound.channels(), 1);
        assert!(sound.total_duration().unwrap().as_millis() >= 900);
    }
}
