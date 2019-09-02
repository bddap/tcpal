//! Keeps a sample buffer populated with TARGET_SECONDS worth of the most recent samples.

use cpal::traits::{DeviceTrait as _, EventLoopTrait, HostTrait as _};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub const TARGET_SECONDS: f32 = 0.1;

/// Start microphone input thread.
pub fn start_sound_thread(output: Arc<Mutex<Vec<f32>>>) {
    thread::spawn(move || sound_thread(output));
}

fn sound_thread(output: Arc<Mutex<Vec<f32>>>) {
    let host = cpal::default_host();
    let event_loop = host.event_loop();

    let mic = host.default_input_device().unwrap();
    let format = mic.default_input_format().unwrap();
    dbg!(&format);
    output.lock().unwrap().resize(
        num_samples_to_save(TARGET_SECONDS, &format),
        Default::default(),
    );
    let mic_stream_id = event_loop.build_input_stream(&mic, &format).unwrap();
    event_loop.play_stream(mic_stream_id.clone()).unwrap();

    event_loop.run(move |_stream_id, stream_result| match stream_result {
        Err(e) => eprintln!("{}", e),
        Ok(data) => match data {
            cpal::StreamData::Input { buffer } => {
                on_audio_input(&format, buffer, &mut output.lock().unwrap());
            }
            cpal::StreamData::Output { .. } => panic!(),
        },
    });
}

/// # Panics
///
/// Panics if num_channels is 0
fn on_audio_input(
    input_format: &cpal::Format,
    buf: cpal::UnknownTypeInputBuffer,
    output: &mut Vec<f32>,
) {
    assert_ne!(input_format.channels, 0);

    debug_assert_eq!(input_format.sample_rate.0 as usize, buf.len());

    let target_samples = num_samples_to_save(TARGET_SECONDS, input_format);

    // read all sound into vec
    let mut got: Vec<f32> = vec![0.0; buf.len()];
    buf.write_as_f32(&mut got);

    // post new sound to micsound
    for sample in got.iter().step_by(input_format.channels as usize) {
        // must step by number of channels because cpal implicitly intersperses audio samples from
        // multiple channels
        output.push(*sample);
    }
    // trim micsound to not be longer than target_samples
    if output.len() > target_samples {
        let len = output.len();
        drop(output.drain(..(len - target_samples)));
        debug_assert_eq!(output.len(), target_samples);
    }
}

trait Sample {
    fn to_f(self) -> f32;
    fn from_f(f: f32) -> Self;
}

impl Sample for u16 {
    fn to_f(self) -> f32 {
        (self as f32) / (Self::max_value() as f32) * 2f32 - 1f32
    }

    fn from_f(f: f32) -> Self {
        ((f + 1.0) * (Self::max_value() as f32) / 2.0) as u16
    }
}

impl Sample for i16 {
    fn to_f(self) -> f32 {
        (self as f32) / (Self::max_value() as f32) * 2f32 - 1f32
    }

    fn from_f(f: f32) -> Self {
        (f * (Self::max_value() as f32)) as i16
    }
}

trait Samples {
    fn len(&self) -> usize;

    /// panics if self len != out len
    fn write_as_f32(self, out: &mut [f32]);
}

impl Samples for cpal::UnknownTypeInputBuffer<'_> {
    fn len(&self) -> usize {
        match self {
            cpal::UnknownTypeInputBuffer::U16(buffer) => buffer.len(),
            cpal::UnknownTypeInputBuffer::I16(buffer) => buffer.len(),
            cpal::UnknownTypeInputBuffer::F32(buffer) => buffer.len(),
        }
    }

    fn write_as_f32(self, out: &mut [f32]) {
        assert_eq!(out.len(), self.len());
        match self {
            cpal::UnknownTypeInputBuffer::U16(buffer) => {
                for (dest, src) in out.iter_mut().zip(buffer.as_ref()) {
                    *dest = src.to_f();
                }
            }
            cpal::UnknownTypeInputBuffer::I16(buffer) => {
                for (dest, src) in out.iter_mut().zip(buffer.as_ref()) {
                    *dest = src.to_f();
                }
            }
            cpal::UnknownTypeInputBuffer::F32(buffer) => {
                out.copy_from_slice(&buffer);
            }
        }
    }
}

fn num_samples_to_save(target_seconds: f32, format: &cpal::Format) -> usize {
    let sample_rate_per_channel = format.sample_rate.0 as usize / format.channels as usize;
    (target_seconds * (sample_rate_per_channel as f32)).round() as usize
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn samples_a() {
        let format = cpal::Format {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            data_type: cpal::SampleFormat::F32,
        };
        assert_eq!(num_samples_to_save(0.1, &format), 1600);
        assert_eq!(num_samples_to_save(1.0, &format), 16000);
    }

    #[test]
    fn samples_b() {
        let format = cpal::Format {
            channels: 2,
            sample_rate: cpal::SampleRate(44100),
            data_type: cpal::SampleFormat::F32,
        };
        assert_eq!(num_samples_to_save(0.1, &format), 2205);
        assert_eq!(num_samples_to_save(1.0, &format), 22050);
    }
}
