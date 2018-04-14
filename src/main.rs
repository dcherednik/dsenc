extern crate getopts;
use getopts::Options;
use std::env;
use std::io::Write;
use std::io::stdout;
extern crate hound;



mod dsd;
use dsd::Dsdenc;
use dsd::filewriter::Writer;
//struct Settings {
//    frame_size: usize
//}

const FRAMESZ: usize = 512;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} -i FILE -o FILE", program);
    print!("{}", opts.usage(&brief));
}

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


    let mut ref_samples = reader.samples::<i16>();

    //DSD64
    let oversample = 64 as usize; // 64 as u32;
    let scale = (1 << (spec.bits_per_sample - 1)) as f64;
    let mut new_spec = spec.clone();
    new_spec.sample_rate = spec.sample_rate * oversample as u32;

    let mut dsd_writer: dsd::filewriter::WriterCtx = dsd::filewriter::Writer::new(output_file, channels, oversample * duration, oversample * FRAMESZ / 8);
    dsd_writer.write_header();

    let mut possition = 0usize;

    let mut buffers: Vec<Vec<f64>> =
        (0..channels).map(|_| vec![0f64; FRAMESZ]).collect();


    let mut encoders: Vec<dsd::DsdencCtx> =
        (0..channels).map(|_| dsd::Dsdenc::new(FRAMESZ, spec.sample_rate as usize, oversample)).collect();

    // encode one extra frame (dummy) per channel to compensate encoding delay
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

        let mut out_buffers: Vec<Vec<i8>> = Vec::new();
        let chs = buffers.len();
        for ch in 0..chs {
            let encoder = &mut encoders[ch];
            let buf = &mut buffers[ch];
            let out = encoder.encode_frame(buf.to_vec());

            out_buffers.push(out);

        }

        // ignore result of encoding first frame - do add extra silence
        if possition == 0 {
            possition = possition + FRAMESZ;
            continue;
        }

        let chs = buffers.len();
        block_pos = 0usize;

        for ch in 0..chs {
            let buf = &mut out_buffers[ch];
            while block_pos < FRAMESZ * oversample && possition * oversample + block_pos < (duration + FRAMESZ) * oversample {
                let sample = buf[block_pos];
                if sample > 0i8 {
                    dsd_writer.write_bit(true);
                } else {
                    dsd_writer.write_bit(false);
                }
                block_pos = block_pos + 1;
            }
            if ch < chs - 1 {
                block_pos = 0usize;
            }
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
//    opts.optopt("f", "", "set frame fize", "");
    opts.optflag("h", "", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.opt_present("i") && matches.opt_present("o") {
        let input_file = matches.opt_str("i").unwrap();
        let output_file = matches.opt_str("o").unwrap();
        do_work(input_file, output_file);
        return;
    }
}
