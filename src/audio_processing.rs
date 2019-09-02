use crate::downsample::downsample;
use rustfft::num_complex::Complex;

/// Frequency domain output will be downsampled to this resolution so output will be consistent
/// accross various audio hardware.
const FFT_BINS: usize = 256;

// const FREQ_BOTTOM: usize = 0;
// const FREQ_TOP: usize = 100_000;

/// Output of an fft, resampled to a fixed size..
pub struct Freqs(pub Box<[f32; FFT_BINS]>);

impl Default for Freqs {
    fn default() -> Self {
        Self(Box::new([Default::default(); FFT_BINS]))
    }
}

impl Freqs {
    pub fn fft(&mut self, time_domain: &[f32]) {
        // TODO: this yields different looking spectra for different microphones needs fixing

        if time_domain.len() == 0 {
            panic!("You probably dont want to perform an fft on zero length data.");
        }

        // do fft
        let complex = fft(time_domain);
        let intermediate_freqs: Vec<f32> = complex.iter().map(|c| c.norm()).collect();

        // for some reason, fft writes the spectrum twice, mirroring the right side. We chop the
        // spectrum in half to remove the redundant data.
        let half_freqs = &intermediate_freqs[0..(intermediate_freqs.len() / 2)];

        // now we slice to only show desired frequencies
        // for now we just guess a good slicing point
        let selected_freqs = &half_freqs[0..(half_freqs.len() / 4)];

        // resample output into freq domain
        downsample(&selected_freqs, &mut *self.0);
    }
}

/// the first fourth of freq_domain is the useful part
fn fft(time_domain: &[f32]) -> Vec<Complex<f32>> {
    debug_assert_ne!(time_domain.len(), 0);

    use rustfft::num_traits::Zero;
    use rustfft::FFTplanner;

    let mut input: Vec<Complex<f32>> = time_domain.iter().map(|f| Complex::new(*f, 0.0)).collect();
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); time_domain.len()];

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(time_domain.len());
    fft.process(&mut input, &mut output);

    output
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn freqs_fft() {
        let mut freqs: Freqs = Default::default();
        let mut samples: Vec<f32> = vec![0.0, 1.0];
        for len in (2..1000).step_by(8) {
            freqs.fft(&samples, 1.0);
            samples.push(len as f32);
        }
    }

    #[test]
    fn proper_peak() {
        let desired_peak = 2.0f32;

        let num_samps = 100_000;
        let samples: Vec<f32> = (0..num_samps)
            .map(|i| ((i as f32) / (num_samps as f32) * desired_peak).sin())
            .collect();
        let mut freqs = Freqs::default();
        freqs.fft(&samples, 1.0);
        // assert there is a peak at desired_peak
        unimplemented!();
    }
}
