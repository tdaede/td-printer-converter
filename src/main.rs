use clap::Parser;
use image::ImageEncoder;
use image::RgbImage;
use image::ImageBuffer;
use image::Rgb;
use std::fs::File;
use std::io::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();

    let mut input = File::open(args.input).unwrap();

    let mut img: RgbImage = ImageBuffer::new(1056, 1056);

    let mut head_y = 0;
    let mut head_x = 0;

    let x_doubling = 2;

    loop {
        let mut c: [u8; 1] = [0; 1];
        match input.read_exact(&mut c) {
            Ok(()) => {},
            Err(_) => break,
        };
        match c[0] {
            // escape
            0x1b => {
                let mut b: [u8; 1] = [0; 1];
                input.read_exact(&mut b).unwrap();
                match b[0] {
                    0x63 => {},
                    0x25 => {},
                    0x4d => {
                        let mut col_count: [u8; 2] = [0; 2];
                        input.read_exact(&mut col_count).unwrap();
                        for x in 0..0x419/6 {
                            let mut p: [u8; 6] = [0; 6];
                            input.read_exact(&mut p).unwrap();
                            for (i, p_byte) in p.iter().enumerate() {
                                for y in 0..8 {
                                    let pixel = match p_byte >> (7-y) & 1 {
                                        0 => image::Rgb([255,255,255]),
                                        1 => image::Rgb([0,0,0]),
                                        _ => unreachable!(),
                                    };
                                    img.put_pixel(x*4 + head_x, y + head_y + i as u32 * 8, pixel);
                                }
                            }
                        }
                        head_x += 1;
                    },
                    _ => {},
                }
            },
            // line feed
            0x10 => {
                head_x = 0;
                head_y += 48;
            }
            _ => {}
        };
    }

    img.save(args.output).unwrap();
}
