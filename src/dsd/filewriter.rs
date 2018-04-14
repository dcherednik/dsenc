use std::fs::File;
use std::io::Write;

pub struct WriterCtx {
    file_writer: File,
    channels: usize,
    samples: usize,
    frame_sz: usize,
    byte_buf: u8,
    pos: u8,
    write_buf: Vec<u8>
}

pub trait Writer {
    fn new(file_name: String, channels: usize, samples: usize, frame_sz: usize) -> Self;
    fn write_header(&mut self);
    fn write_bit(&mut self, bit: bool);
}

fn convert_to_bytes(shift: usize, input: usize, buf: &mut Vec<u8>) {
    let mut x = input;
    for i in 0..4 {
        buf[shift + i] = (x & 0xffusize) as u8;
        x >>= 8;
    }
}

impl Drop for WriterCtx {
    fn drop(&mut self) {
        self.file_writer.write_all(&self.write_buf).expect("Unable to write block");
    }
}

impl Writer for WriterCtx {
    fn new(file_name: String, channels: usize, samples: usize, frame_sz: usize) -> WriterCtx {
        WriterCtx {
            file_writer:  File::create(file_name).expect("Unable to open"),
            channels: channels,
            samples: samples,
            frame_sz: frame_sz,
            byte_buf: 0u8,
            pos: 0u8,
            write_buf: Vec::with_capacity(4096)
        }
    }
    fn write_header(&mut self) {
        let mut buf: Vec<u8> = vec![0u8; 28 + 52 + 12]; //28 - DSD header, 52 - fmt header
        buf[0] = 0x44 as u8; //"D"
        buf[1] = 0x53 as u8; //"S"
        buf[2] = 0x44 as u8; //"D"
        buf[3] = 0x20 as u8; //" "
        buf[4] = 0x1c as u8; //28 - size of DSD header

        buf[28] = 0x66 as u8; //"f"
        buf[29] = 0x6d as u8; //"m"
        buf[30] = 0x74 as u8; //"t"
        buf[31] = 0x20 as u8; //" "
        buf[32] = 0x34 as u8; //58 - size of fmt header
        buf[40] = 0x1 as u8; //1 - version of format
        buf[48] = self.channels as u8;
        buf[52] = self.channels as u8;
        buf[56] = 0x0 as u8;
        buf[57] = 0x11 as u8;
        buf[58] = 0x2b as u8;
        buf[59] = 0x00 as u8;
        buf[60] = 0x08 as u8;

        convert_to_bytes(64, self.samples, &mut buf);

        convert_to_bytes(72, self.frame_sz, &mut buf);
        buf[80] = 0x64 as u8; //d
        buf[81] = 0x61 as u8; //a
        buf[82] = 0x74 as u8; //t
        buf[83] = 0x61 as u8; //a

        convert_to_bytes(84, self.samples, &mut buf);

        self.file_writer.write_all(&buf).expect("Unable to write block");

    }
    fn write_bit(&mut self, bit: bool) {

        self.byte_buf = self.byte_buf + bit as u8;

        self.pos = self.pos + 1u8;
        if self.pos == 8u8 {
            self.write_buf.push(self.byte_buf);
            if self.write_buf.len() == 4096 {
                self.file_writer.write_all(&self.write_buf).expect("Unable to write block");
                self.write_buf.clear();
            }
            self.pos = 0u8;
            self.byte_buf = 0u8;
        } else {
            self.byte_buf = self.byte_buf.rotate_left(1);
        }
    }
}
