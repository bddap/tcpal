use std::sync::{Arc, Mutex};

mod audio_input;
mod audio_processing;
mod downsample;
mod plot;

fn main() {
    let recent_samples = Arc::new(Mutex::new(Vec::new()));
    audio_input::start_sound_thread(recent_samples.clone());
    crate::plot::plot(recent_samples);
}
