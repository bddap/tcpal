use crate::downsample::downsample;
use rustfft::num_complex::Complex;

// Frequency domain output will be downsampled to this resolution so output will be consistent
// accross various audio hardware.
const FFT_BINS: usize = 256;

/// Output of an fft, resampled to a fixed size..
pub struct Freqs(pub Box<[f32; FFT_BINS]>);

impl Default for Freqs {
    fn default() -> Self {
        Self(Box::new([Default::default(); FFT_BINS]))
    }
}

impl Freqs {
    /// Accepts raw audio, converts to frequency domain, resamples to `FFT_BINS` frequency
    /// measurements returns a reference to the spectrum measurement. The spectrum is stored in
    /// freq_domain.
    ///
    /// freq_domain does not need to be the same length as time_domain
    ///
    /// # Panics
    ///
    /// panics if time_domain.len() == 0
    pub fn fft(&mut self, time_domain: &[f32]) {
        if time_domain.len() == 0 {
            panic!("You probably dont want to perform an fft on zero length data.");
        }

        // do fft
        let complex = fft(time_domain);
        let intermediate_freqs: Vec<f32> = complex.iter().map(|c| c.norm()).collect();

        // resample output into freq domain
        downsample(
            &intermediate_freqs[0..(intermediate_freqs.len() / 4)],
            &mut *self.0,
        );
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
            freqs.fft(&samples);
            samples.push(len as f32);
        }
    }
}
