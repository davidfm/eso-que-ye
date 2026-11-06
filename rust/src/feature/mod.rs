// Modified by David Fernández, 2026. See git history for details.

pub mod autocorrelate;
pub mod interpolate;
pub mod normalise;
pub mod signal;
pub mod types;
pub mod window;

use crate::ff_config;
use interpolate::compute_interp_inds;
use rustfft::{algorithm::Radix4, FftDirection};
use signal::validate_sample_rate;
use std::convert::TryInto;
use types::{Features, InterpInds, Spectrum, Window};
use window::gen_blackman_window;

pub struct FeatureExtractor {
    pub sample_rate: u32,
    pub window_function: Window,
    pub fft: Radix4<f32>,
    pub interp_inds: InterpInds,
    pub spectra: Vec<Spectrum>,
}

impl FeatureExtractor {
    pub fn new(sample_rate: u32) -> Result<FeatureExtractor, signal::SampleRateError> {
        if !validate_sample_rate(&sample_rate) {
            return Err(signal::SampleRateError);
        }

        Ok(FeatureExtractor {
            sample_rate: sample_rate,
            window_function: gen_blackman_window(),
            fft: Radix4::new(ff_config::SPEC_WINDOW_SIZE, FftDirection::Forward),
            interp_inds: compute_interp_inds(&sample_rate),
            spectra: Vec::new(),
        })
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        // Need to know the sample rate to undertake the linear resampling step
        validate_sample_rate(&sample_rate);
        self.interp_inds = compute_interp_inds(&sample_rate);
    }

    pub fn feed_signal(&mut self, signal: Vec<f32>) {
        // Implicit floor division
        let num_windows = signal.len() / ff_config::SPEC_WINDOW_SIZE;
        let last_ind = num_windows * ff_config::SPEC_WINDOW_SIZE;
        for i in (0..last_ind).step_by(ff_config::SPEC_WINDOW_SIZE) {
            self.feed_window(
                signal[i..i + ff_config::SPEC_WINDOW_SIZE]
                    .try_into()
                    .expect("Invalid window length"),
            );
        }
    }
    pub fn feed_window(&mut self, window: [f32; ff_config::SPEC_WINDOW_SIZE]) {
        let spectrum: Spectrum = autocorrelate::modified_autocorrelation(
            window,
            &self.window_function,
            &self.fft,
            &self.interp_inds,
        );
        self.spectra.push(spectrum);
    }

    pub fn features(&self) -> Features {
        // One feature per MIDI pitch per frame, with the pitch boundaries
        //  moved to suit the instrument's tuning if it is far from A = 440 Hz.
        let cents = self.tuning_offset_cents();
        let bin_offset = if cents <= -ff_config::TUNING_CORRECTION_THRESHOLD_CENTS {
            -1
        } else if cents >= ff_config::TUNING_CORRECTION_THRESHOLD_CENTS {
            1
        } else {
            0
        };
        self.spectra
            .iter()
            .map(|spectrum| autocorrelate::frame_from_spectrum(spectrum, bin_offset))
            .collect()
    }

    pub fn tuning_offset_cents(&self) -> f32 {
        // Estimate how far the instrument is from A = 440 Hz equal temperament,
        //  in [-50, 50] cents. Take the strongest peak of each frame, refine
        //  its position between bins, and average its distance from the nearest
        //  semitone, weighted by energy. The average is circular, so that +45
        //  and -45 cents (which are 10 cents apart) don't cancel out.
        let bins_per_midi = ff_config::SPEC_BINS_PER_MIDI as f32;
        let (mut sum_sin, mut sum_cos) = (0f32, 0f32);
        for spectrum in &self.spectra {
            let (peak, &energy) = spectrum[1..spectrum.len() - 1]
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap();
            let peak = peak + 1;
            if energy <= 0. {
                continue;
            }
            let (a, b, c) = (spectrum[peak - 1], energy, spectrum[peak + 1]);
            let curvature = a - 2. * b + c;
            let delta = if curvature < 0. { 0.5 * (a - c) / curvature } else { 0. };
            // Bin 1 of each group of 3 is exactly on the MIDI pitch
            let semitones = (peak as f32 + delta - (bins_per_midi - 1.) / 2.) / bins_per_midi;
            let angle = 2. * std::f32::consts::PI * (semitones - semitones.round());
            sum_sin += energy * angle.sin();
            sum_cos += energy * angle.cos();
        }
        if sum_sin == 0. && sum_cos == 0. {
            return 0.;
        }
        100. * sum_sin.atan2(sum_cos) / (2. * std::f32::consts::PI)
    }

    pub fn flush(&mut self) {
        self.spectra = Vec::new();
    }
}
