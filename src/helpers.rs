use std::io::stdout;
use std::io::Write;

pub fn show_progress(pos: usize, duration: usize, cont: bool) {
    let percent = pos * 100usize / duration;
    if cont {
        print!("{} % \r", percent);
    } else {
        print!("{} % done\n", percent);
    }
    stdout().flush().expect("Unable to flush stdout");
}

