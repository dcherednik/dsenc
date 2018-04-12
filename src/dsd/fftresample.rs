extern crate num;
extern crate rustfft;

use self::rustfft::algorithm::Radix4;
use self::num::complex::Complex;
use self::rustfft::FFT;
use self::rustfft::num_traits::Zero;

use std::f64::consts::PI;
use std::boxed::Box;

pub struct ResampleCtx {
    frame_size: usize,
    oversample: usize,
    input_half_win: Vec<f64>,
    output_half_win: Vec<f64>,
    input_buffer: Vec<Complex<f64>>,
    output_buffer: Vec<f64>,
    fft: Box<FFT<f64>>,
    ifft: Box<FFT<f64>>
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
        s = (sz * 2) as f64;
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
                      fft: Box::new(Radix4::new(sz * 2, false)),
                      ifft: Box::new(Radix4::new((sz * oversample * 2), true))
        }
    }

    fn resample_frame(&mut self, input: Vec<f64>) -> Vec<f64> {
        let mut oversample_buf: Vec<f64> = vec![Zero::zero(); self.frame_size * self.oversample];

        let buf_midle = self.input_buffer.len() / 2;

        let input_end = input.len();

        // Apply window and copy input to the second part of buffer
        for i in 0..input_end {
            self.input_buffer[buf_midle + i].re = input[i] * self.input_half_win[input_end - 1 - i];
            self.input_buffer[buf_midle + i].im = Zero::zero();
        }


        let mut spectrum: Vec<Complex<f64>> = vec![Zero::zero(); self.frame_size * 2];
        self.fft.process(&mut self.input_buffer, &mut spectrum);

        // Apply window and copy to the first part of buffer (overlap)
        for i in 0..input.len() {
            self.input_buffer[i].re = input[i] * self.input_half_win[i];
            self.input_buffer[i].im = Zero::zero();
        }

        let mut new_spectrum: Vec<Complex<f64>> = Vec::with_capacity(self.frame_size * 2 * self.oversample);
        let middle = spectrum.len() / 2;

        // Create new spectrum
        for i in 0..middle {
            new_spectrum.push(spectrum[i]);
        }

        for _j in 0..((self.frame_size * (self.oversample - 1))* 2) {
            new_spectrum.push(Complex::new(0f64, 0f64));
        }

        new_spectrum[self.frame_size * self.oversample] = spectrum[middle];
        spectrum[middle] = Complex::new(0f64, 0f64);

        for i in middle..spectrum.len() {
            new_spectrum.push(spectrum[i]);
        }

        let mut result:   Vec<Complex<f64>> = vec![Zero::zero(); (self.frame_size * self.oversample * 2)];
        self.ifft.process(&mut new_spectrum, &mut result);

        let half = result.len()/2;
        for i in 0..half {
            oversample_buf[i] = result[i].re * self.output_half_win[i] + self.output_buffer[i];
            self.output_buffer[i] = result[half + i].re * self.output_half_win[half - 1 - i];
        }

        return oversample_buf;
    }
}
