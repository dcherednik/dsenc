//extern crate rand;

//use self::rand::{Rng, thread_rng};
//use self::rand::distributions::{Normal};
//use self::rand::distributions::Sample;

pub struct ModulatorCtx {
    z: i8,
    z1: f64,
    z2: f64
}

pub trait Modulator {
    fn new() -> Self;
    fn process_frame(&mut self, input: Vec<f64>) -> Vec<i8>;
}

fn tobit(input: f64) -> i8 {
    let res: i8;
    if input < 0f64 {
        res = -1i8;
    } else {
        res = 1i8;
    }
    return res;
}

impl Modulator for ModulatorCtx {
    fn new() -> ModulatorCtx {
        ModulatorCtx {
            z: 0i8,
            z1: 0f64,
            z2: 0f64
        }
    }
    fn process_frame(&mut self, input: Vec<f64>) -> Vec<i8> {
        //let mut normal = Normal::new(0.0, 4096.0);
        let mut out: Vec<i8> = vec![0i8; input.len()];
        //let mut rng = thread_rng();
        let mut z = self.z;
        let mut z1 = self.z1;
        let mut z2 = self.z2;
        for i in 0..input.len() {
            z2 = z2 + input[i] - z as f64;
            z1 = z1 + z2 - z as f64;
            z = tobit(z1);
            out[i] = z;
        }
        self.z = z;
        self.z1 = z1;
        self.z2 = z2;
        return out;
    }
}
