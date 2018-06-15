extern crate getopts;
use getopts::Options;
use std::env;
use std::io::Write;
use std::io::stdout;
extern crate hound;

#[path = "../../dsd/fftresample.rs"]
mod fftresample;
use self::fftresample::Resampler;
use self::fftresample::ResampleCtx;

const FRAMESZ: usize = 512;

fn show_progress(pos: usize, duration: usize, cont: bool) {
    let percent = pos * 100usize / duration;
    if cont {
        print!("{} % \r", percent);
    } else {
        print!("{} % done\n", percent);
    }
    stdout().flush().expect("Unable to flush stdout");
}

fn do_work(input_file: String, output_file: String) {
    let mut reader = hound::WavReader::open(input_file).unwrap();
    let spec = reader.spec();
    let duration = reader.duration() as usize;
    let channels = spec.channels as usize;


    let mut ref_samples = reader.samples::<i32>();

    let oversample = 64 as usize;
    let scale = (1 << (spec.bits_per_sample - 1)) as f64;
    let mut new_spec = spec.clone();
    new_spec.sample_rate = spec.sample_rate * oversample as u32;

    let mut writer = hound::WavWriter::create(output_file, new_spec).unwrap();

    let mut possition = 0usize;

    let mut buffers: Vec<Vec<f64>> =
        (0..channels).map(|_| vec![0f64; FRAMESZ]).collect();

    let mut resamplers: Vec<fftresample::ResampleCtx> =
        (0..channels).map(|_| fftresample::Resampler::new(FRAMESZ, oversample)).collect();

    while possition < duration + FRAMESZ {
        let mut block_pos = 0u32 as usize;
        while block_pos < FRAMESZ && possition + block_pos < duration {
            let chs = buffers.len();
            for ch in 0..chs {
                let buf = &mut buffers[ch];
                let sample = ref_samples.next().map(|r| r.ok().unwrap());
                buf[block_pos] = sample.unwrap() as f64 / scale;
            }
            block_pos = block_pos + 1;
        }

        // zero end of frame if real lengh less than frame fize
        while block_pos < FRAMESZ {
            let chs = buffers.len();
            for ch in 0..chs {
                let buf = &mut buffers[ch];
                buf[block_pos] = 0f64;
            }
            block_pos = block_pos + 1;
        }

        let mut out_buffers: Vec<Vec<f64>> = Vec::new();
        let chs = buffers.len();
        for ch in 0..chs {
            let resampler = &mut resamplers[ch];
            let buf = &mut buffers[ch];
            let out = resampler.resample_frame(buf.to_vec());
            out_buffers.push(out);
        }

        // ignore result of encoding first frame - do add extra silence
        if possition == 0 {
            possition = possition + FRAMESZ;
            continue;
        }

        let chs = buffers.len();
        block_pos = 0usize;

        while block_pos < FRAMESZ * oversample && possition * oversample + block_pos < (duration + FRAMESZ) * oversample {
            for ch in 0..chs {
                let buf = &mut out_buffers[ch];
                let sample = buf[block_pos];
                writer.write_sample((sample * scale) as i32).unwrap();
            }
            block_pos = block_pos + 1;
        }

        possition = possition + block_pos / oversample;

        show_progress(possition, duration, true);
    }

    show_progress(possition, duration, false);

}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("i", "", "set input file name", "NAME");
    opts.optopt("o", "", "set output file name", "NAME");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("i") && matches.opt_present("o") {
        let input_file = matches.opt_str("i").unwrap();
        let output_file = matches.opt_str("o").unwrap();
        do_work(input_file, output_file);
        return;
    }

}
