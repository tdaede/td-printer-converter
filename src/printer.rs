use std::io::Read;
use image::RgbImage;

pub struct Cz8pc4 {

}

impl Cz8pc4 {
    pub fn create_image() -> RgbImage {
        let page_width = 2000;
        let page_height = 2000;
        RgbImage::new(page_width, page_height)
    }

    pub fn decode(input: &mut dyn Read, img: &mut RgbImage) {
        let page_width = 2000;
        let page_height = 2000;
        let mut head_y = 0;
        let mut head_x = 0;
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
                            for x in 0..0x800 {
                                let mut p: [u8; 6] = [0; 6];
                                if !input.read_exact(&mut p).is_ok() {
                                    continue;
                                }
                                for (i, p_byte) in p.iter().enumerate() {
                                    for y in 0..8 {
                                        let pixel = match p_byte >> (7-y) & 1 {
                                            0 => image::Rgb([255,255,255]),
                                            1 => image::Rgb([0,0,0]),
                                            _ => unreachable!(),
                                        };
                                        let pixel_x = x*4 + head_x;
                                        let pixel_y = y + head_y + i as u32 * 8;
                                        if pixel_x < page_width && pixel_y < page_height {
                                            img.put_pixel(pixel_x, pixel_y, pixel);
                                        }
                                    }
                                }
                            }
                            head_x += 0x800;
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
    }
}
