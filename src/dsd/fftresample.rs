extern crate num;
extern crate rustfft;
extern crate rfft;

use self::num::complex::Complex;
use self::rustfft::num_traits::Zero;
use self::rfft::{RFFT, RFFTImpl, RIFFT, RIFFTImpl};

use std::f64::consts::PI;
use std::boxed::Box;

pub struct ResampleCtx {
    frame_size: usize,
    oversample: usize,
    input_half_win: Vec<f64>,
    output_half_win: Vec<f64>,
    input_buffer: Vec<f64>,
    output_buffer: Vec<f64>,
    fft: Box<RFFT<f64>>,
    ifft: Box<RIFFT<f64>>
}

pub trait Resampler {
    fn new(sz: usize, oversample: usize) -> Self;
    fn resample_frame(&mut self, input: Vec<f64>) -> Vec<f64>;
}

fn create_half_win (sz: usize, scale: bool) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::with_capacity(sz);
    let n = sz * 2; //full size
    // sine window
    let s: f64;
    if scale {
        s = sz as f64;
    } else {
        s = 1f64;
    }
    for i in 0..sz {
        result.push((PI * (i as f64) / (n - 1) as f64).sin() / s);
    }
    return result;
}

impl Resampler for ResampleCtx {
    fn new(sz: usize, oversample: usize) -> ResampleCtx {
        ResampleCtx { frame_size: sz,
                      oversample: oversample,
                      input_half_win: create_half_win(sz, true),
                      output_half_win: create_half_win(sz * oversample, false),
                      input_buffer: vec![Zero::zero(); sz * 2],
                      output_buffer: vec![Zero::zero(); sz * oversample],
                      fft: Box::new(RFFTImpl::new(sz * 2)),
                      ifft: Box::new(RIFFTImpl::new(sz * oversample * 2))
        }
    }

    fn resample_frame(&mut self, input: Vec<f64>) -> Vec<f64> {
        let mut oversample_buf: Vec<f64> = vec![Zero::zero(); self.frame_size * self.oversample];

        let buf_midle = self.input_buffer.len() / 2;

        let input_end = input.len();

        // Apply window and copy input to the second part of buffer
        for i in 0..input_end {
            self.input_buffer[buf_midle + i] = input[i] * self.input_half_win[input_end - 1 - i];
        }

        let mut spectrum: Vec<Complex<f64>> = vec![Zero::zero(); self.frame_size];
        self.fft.process(&mut self.input_buffer, &mut spectrum);

        // Apply window and copy to the first part of buffer (overlap)
        for i in 0..input.len() {
            self.input_buffer[i] = input[i] * self.input_half_win[i];
        }

        let mut new_spectrum: Vec<Complex<f64>> = Vec::with_capacity(self.frame_size * self.oversample);
        let end = spectrum.len();

        // Create new spectrum
        for i in 0..end {
            new_spectrum.push(spectrum[i]);
        }

        for _j in 0..(self.frame_size * (self.oversample - 1)) {
            new_spectrum.push(Complex::new(0f64, 0f64));
        }

        let mut result: Vec<f64> = vec![Zero::zero(); (self.frame_size * self.oversample * 2)];
        self.ifft.process(&mut new_spectrum, &mut result);

        let half = result.len()/2;
        for i in 0..half {
            oversample_buf[i] = result[i] * self.output_half_win[i] + self.output_buffer[i];
            self.output_buffer[i] = result[half + i] * self.output_half_win[half - 1 - i];
        }

        return oversample_buf;
    }
}
