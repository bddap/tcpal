use cpal::traits::{DeviceTrait as _, EventLoopTrait, HostTrait as _};
use std::sync::Mutex;
use std::thread;

mod plot;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MICSOUND: Mutex<Vec<f32>> = Mutex::new(Vec::new());
    static ref PLAY_NEXT: Mutex<Vec<f32>> = Mutex::new(Vec::new());
}

const TARGET_SAMPLES: usize = 512;
fn main() {
    thread::spawn(sound_thread);
    crate::plot::plot();
}

fn sound_thread() {
    let host = cpal::default_host();
    let event_loop = host.event_loop();

    let mic = host.default_input_device().unwrap();
    let format = mic.default_input_format().unwrap();
    let mic_stream_id = event_loop.build_input_stream(&mic, &format).unwrap();
    event_loop.play_stream(mic_stream_id.clone()).unwrap();

    // let speaker = host.default_output_device().unwrap();
    // let format = speaker.default_output_format().unwrap();
    // let stream_id = event_loop.build_output_stream(&speaker, &format).unwrap();
    // event_loop.play_stream(stream_id);

    event_loop.run(move |_stream_id, stream_result| match stream_result {
        Err(e) => eprintln!("{}", e),
        Ok(data) => match data {
            cpal::StreamData::Input { buffer } => {
                on_audio_input(buffer);
            }
            cpal::StreamData::Output { buffer } => {
                on_audio_output(buffer);
            }
        },
    });
}

fn on_audio_output(_buf: cpal::UnknownTypeOutputBuffer) {
    unimplemented!();
    // let mut out = PLAY_NEXT.lock().unwrap();
    // from_f32(&mut out, buf);
    // out.clear();
}

fn on_audio_input(buf: cpal::UnknownTypeInputBuffer) {
    // read all sound into vec
    let mut got: Vec<f32> = vec![0.0; buf.len()];
    buf.write_as_f32(&mut got);

    // post new sound to micsound
    let mut mc = MICSOUND.lock().unwrap();
    mc.extend_from_slice(&got);
    // trim micsound to not be longer than TARGET_SAMPLES
    if mc.len() > TARGET_SAMPLES {
        let len = mc.len();
        drop(mc.drain(..(len - TARGET_SAMPLES)));
        debug_assert_eq!(mc.len(), TARGET_SAMPLES);
    }

    // post new sound to to_play
    // PLAY_NEXT.lock().unwrap().append(&mut got);
}

trait Sample {
    fn to_f(self) -> f32;
    fn from_f(f: f32) -> Self;
}

impl Sample for f32 {
    fn to_f(self) -> f32 {
        self
    }

    fn from_f(f: f32) -> Self {
        f
    }
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
                for (dest, src) in out.iter_mut().zip(buffer.as_ref()) {
                    *dest = src.to_f();
                }
            }
        }
    }
}
