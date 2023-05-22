use std::io::Read;
use image::RgbImage;

pub struct Cz8pc4 {

}

impl Cz8pc4 {
    pub fn create_image() -> RgbImage {
        let page_width = 3000;
        let page_height = 2000;
        RgbImage::from_pixel(page_width, page_height, image::Rgb([255,255,255]))
    }

    pub fn decode(input: &mut dyn Read, img: &mut RgbImage) -> (u32, u32) {
        let mut head_y = 0;
        let mut head_x = 0;
        let mut covered_x = 0;
        let mut covered_y = 0;
        let mut color = 0; // 0 = black, 1 = yellow, 2 = magenta, 3 = cyan
        let page_width = img.width();
        let page_height = img.height();
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
                        0x23 => { // unknown 1 byte
                            let mut b: [u8; 1] = [0; 1];
                            input.read_exact(&mut b).unwrap();
                        },
                        0x25 => { // line spacing
                            let mut asdf: [u8; 2] = [0; 2];
                            input.read_exact(&mut asdf).unwrap();
                            // discard line spacing for now
                        },
                        0x19 => {
                            color = 1;
                        },
                        0x4c => { // unknown
                            let mut b: [u8; 3] = [0; 3];
                            input.read_exact(&mut b).unwrap();
                        }
                        0x4d => { // 48 dot
                            let mut col_count_bytes: [u8; 2] = [0; 2];
                            input.read_exact(&mut col_count_bytes).unwrap();
                            let col_count = 3000.min(u16::from_be_bytes(col_count_bytes) as u32);
                            for x in 0..col_count {
                                let mut p: [u8; 6] = [0; 6];
                                if !input.read_exact(&mut p).is_ok() {
                                    continue;
                                }
                                for (i, p_byte) in p.iter().enumerate() {
                                    for y in 0..8 {
                                        if color == 0 {
                                            let pixel = match p_byte >> (7-y) & 1 {
                                                0 => image::Rgb([255,255,255]),
                                                1 => image::Rgb([0,0,0]),
                                                _ => unreachable!(),
                                            };
                                            let pixel_x = x + head_x;
                                            let pixel_y = y + head_y + i as u32 * 8;
                                            if pixel_x < page_width && pixel_y < page_height {
                                                img.put_pixel(pixel_x, pixel_y, pixel);
                                                covered_y = covered_y.max(pixel_y);
                                            }
                                        } else {
                                            if p_byte >> (7-y) & 1 != 0 {
                                                let pixel_x = x + head_x;
                                                let pixel_y = y + head_y + i as u32 * 8;
                                                if pixel_x < page_width && pixel_y < page_height {
                                                    let mut pixel = img.get_pixel(pixel_x, pixel_y).clone();
                                                    match color {
                                                        1 => {
                                                            pixel[2] = 0;
                                                        },
                                                        2 => {
                                                            pixel[1] = 0;
                                                        },
                                                        3 => {
                                                            pixel[0] = 0;
                                                        },
                                                        _ => {
                                                            unreachable!();
                                                        }
                                                    }
                                                    img.put_pixel(pixel_x, pixel_y, pixel);
                                                    covered_y = covered_y.max(pixel_y);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            head_x += col_count;
                            covered_x = covered_x.max(head_x);
                        },
                        _ => {
                            eprintln!("Warning: unsupported escape code {:x}", b[0]);
                        },
                    }
                },
                // line feed
                0x0a => {
                    head_x = 0;
                    head_y += 48;
                },
                // carriage return / color change
                0x0d => {
                    head_x = 0;
                    if color > 0 {
                        color += 1;
                        if color > 3 {
                            color = 1;
                        }
                    }
                }
                _ => {}
            };
        }
        (covered_x, covered_y)
    }
}
