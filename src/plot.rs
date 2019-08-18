use piston_window::{EventLoop, PistonWindow, WindowSettings};
use plotters::prelude::*;

const FPS: u32 = 10;
const LENGTH: u32 = 20;
const N_DATA_POINTS: usize = (FPS * LENGTH) as usize;

pub fn plot() {
    let mut window: PistonWindow = WindowSettings::new("Realtime CPU Usage", [450, 300])
        .samples(4)
        .build()
        .unwrap();

    window.set_max_fps(FPS as u64);

    while let Some(_) = draw_piston_window(&mut window, |b| {
        let root = b.into_drawing_area();
        root.fill(&White)?;

        let mut cc = ChartBuilder::on(&root)
            .margin(10)
            .caption("fft", ("Arial", 30).into_font())
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_ranged(0..N_DATA_POINTS as u32, 0f32..1f32)?;

        cc.configure_mesh()
            .x_label_formatter(&|x| format!("{}", -(LENGTH as f32) + (*x as f32 / FPS as f32)))
            .y_label_formatter(&|y| format!("{}%", (*y * 100.0) as u32))
            .x_labels(15)
            .y_labels(5)
            .x_desc("X")
            .y_desc("Y")
            .axis_desc_style(("Arial", 15).into_font())
            .draw()?;

        let xs = get_fft();
        // let xs = crate::MICSOUND.lock().unwrap().clone();
        cc.draw_series(LineSeries::new(
            (0..).zip(xs.iter()).map(|(a, b)| (a, *b)),
            &Palette99::pick(1),
        ))?;

        cc.configure_series_labels()
            .background_style(&White.mix(0.8))
            .border_style(&Black)
            .draw()?;

        Ok(())
    }) {}
}

fn get_fft() -> Vec<f32> {
    // perform fft and post result to freqs, could be done on another thread
    let mc = crate::MICSOUND.lock().unwrap().clone();
    let mut freqd: Vec<f32> = vec![0.0; mc.len()];
    fft(&mc, &mut freqd);
    freqd
}

/// panics if time_domain.len() != freq_domain.len()
fn fft(time_domain: &[f32], freq_domain: &mut [f32]) {
    assert_eq!(time_domain.len(), freq_domain.len());

    use rustfft::num_complex::Complex;
    use rustfft::num_traits::Zero;
    use rustfft::FFTplanner;

    let mut input: Vec<Complex<f32>> = time_domain.iter().map(|f| Complex::new(*f, 0.0)).collect();
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); time_domain.len()];

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(time_domain.len());
    fft.process(&mut input, &mut output);

    for (complex, out) in output.iter().zip(freq_domain) {
        *out = complex.re;
    }
}
