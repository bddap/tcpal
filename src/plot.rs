use crate::audio_processing::Freqs;
use piston_window::{EventLoop, PistonWindow, WindowSettings};
use plotters::prelude::*;
use std::sync::{Arc, Mutex};

const MAX_FPS: u64 = 30;

pub fn plot(input: Arc<Mutex<Vec<f32>>>) {
    let mut window: PistonWindow = WindowSettings::new("Realtime CPU Usage", [450, 300])
        .samples(4)
        .build()
        .unwrap();
    window.set_max_fps(MAX_FPS);

    let mut samples: Vec<f32> = Vec::new();
    let mut freqs: Freqs = Default::default();

    while let Some(_) = draw_piston_window(&mut window, |b| {
        let input = input.lock().unwrap();
        samples.resize(input.len(), Default::default());
        samples.copy_from_slice(input.as_slice());
        drop(input); // release lock
        freqs.fft(&samples);
        draw(b, &*freqs.0).map_err(Into::into)
    }) {}
}

fn draw<DB: DrawingBackend>(b: DB, xs: &[f32]) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    let root = b.into_drawing_area();
    root.fill(&White)?;

    let mut cc = ChartBuilder::on(&root)
        .margin(10)
        .x_label_area_size(40)
        .build_ranged(0..xs.len() as u32, 0f32..20f32)?;

    cc.configure_mesh()
        .x_labels(8)
        .y_labels(4)
        .axis_desc_style(("Arial", 15).into_font())
        .draw()?;

    let step = (xs.len() / 512 / 2).max(1);
    cc.draw_series(LineSeries::new(
        (0..).zip(xs.iter()).step_by(step).map(|(a, b)| (a, *b)),
        &Palette99::pick(1),
    ))?;

    Ok(())
}
