extern crate num;
extern crate rustfft;

mod fftresample;
mod modulator;
pub mod filewriter;

use self::fftresample::Resampler;
use self::fftresample::ResampleCtx;
use self::modulator::Modulator;
use self::modulator::ModulatorCtx;

pub struct DsdencCtx {
    resampler: ResampleCtx,
    modulator: ModulatorCtx
}

pub trait Dsdenc {
    fn new(sz: usize, input_samplerate: usize, oversample: usize) -> Self;
    fn encode_frame(&mut self, input: Vec<f64>) -> Vec<i8>;
}

impl Dsdenc for DsdencCtx {
    fn new(sz: usize, _input_samplerate: usize, oversample: usize) -> DsdencCtx {

        DsdencCtx {
                    resampler: Resampler::new(sz, oversample),
                    modulator: Modulator::new()
        }
    }

    fn encode_frame(&mut self, input: Vec<f64>) -> Vec<i8> {
        return self.modulator.process_frame(self.resampler.resample_frame(input));
        //return self.resampler.resample_frame(input);
    }
}
