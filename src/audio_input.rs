//! Keeps MICSOUND populated with TARGET_SAMPLES most recent audio samples

use cpal::traits::{DeviceTrait as _, EventLoopTrait, HostTrait as _};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

const TARGET_SAMPLES: usize = 512 * 32 * 4;
// const TARGET_MICROSECONDS: usize = 512 * 32;

/// Start microphone input thread.
pub fn start_sound_thread(output: Arc<Mutex<Vec<f32>>>) {
    thread::spawn(move || sound_thread(output));
}

fn sound_thread(output: Arc<Mutex<Vec<f32>>>) {
    output
        .lock()
        .unwrap()
        .resize(TARGET_SAMPLES, Default::default());

    let host = cpal::default_host();
    let event_loop = host.event_loop();

    let mic = host.default_input_device().unwrap();
    let format = mic.default_input_format().unwrap();
    dbg!(&format);
    let mic_stream_id = event_loop.build_input_stream(&mic, &format).unwrap();
    event_loop.play_stream(mic_stream_id.clone()).unwrap();

    event_loop.run(move |_stream_id, stream_result| match stream_result {
        Err(e) => eprintln!("{}", e),
        Ok(data) => match data {
            cpal::StreamData::Input { buffer } => {
                on_audio_input(buffer, &mut output.lock().unwrap());
            }
            cpal::StreamData::Output { .. } => panic!(),
        },
    });
}

fn on_audio_input(buf: cpal::UnknownTypeInputBuffer, output: &mut Vec<f32>) {
    // read all sound into vec
    let mut got: Vec<f32> = vec![0.0; buf.len()];
    buf.write_as_f32(&mut got);

    // post new sound to micsound
    output.extend_from_slice(&got);
    // trim micsound to not be longer than TARGET_SAMPLES
    if output.len() > TARGET_SAMPLES {
        let len = output.len();
        drop(output.drain(..(len - TARGET_SAMPLES)));
        debug_assert_eq!(output.len(), TARGET_SAMPLES);
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
