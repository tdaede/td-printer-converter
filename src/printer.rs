use std::io::Read;
use std::sync::Mutex;
use image::RgbImage;


pub trait Printer {
    fn create_image(&self) -> RgbImage;
    fn decode(&mut self, input: &mut dyn Read, img_mutex: &Mutex<RgbImage>) -> (u32, u32);
}

#[derive(Default)]
pub struct Cz8pc4 {
    head_y: u32,
    covered_x: u32,
    covered_y: u32,
    color: u32, // 0 = black, 1 = yellow, 2 = magenta, 3 = cyan
}

impl Cz8pc4 {
    pub const PAGE_WIDTH: u32 = 2988;
    pub const PAGE_HEIGHT: u32 = 2000;
}

impl Printer for Cz8pc4 {
    fn create_image(&self) -> RgbImage {
        RgbImage::from_pixel(Cz8pc4::PAGE_WIDTH, Cz8pc4::PAGE_HEIGHT, image::Rgb([255,255,255]))
    }

    fn decode(&mut self, input: &mut dyn Read, img_mutex: &Mutex<RgbImage>) -> (u32, u32) {
        let head_y = &mut self.head_y;
        let mut head_x = 0;
        let covered_x = &mut self.covered_x;
        let covered_y = &mut self.covered_y;
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
                            self.color = 1;
                        },
                        0x4c => { // unknown
                            let mut b: [u8; 3] = [0; 3];
                            input.read_exact(&mut b).unwrap();
                        }
                        0x4d => { // 48 dot
                            let mut col_count_bytes: [u8; 2] = [0; 2];
                            input.read_exact(&mut col_count_bytes).unwrap();
                            let col_count = Cz8pc4::PAGE_WIDTH.min(u16::from_be_bytes(col_count_bytes) as u32);
                            for x in 0..col_count {
                                let mut p: [u8; 6] = [0; 6];
                                if !input.read_exact(&mut p).is_ok() {
                                    continue;
                                }
                                let mut img = img_mutex.lock().unwrap();
                                let page_width = img.width();
                                let page_height = img.height();
                                for (i, p_byte) in p.iter().enumerate() {
                                    for y in 0..8 {
                                        if self.color == 0 {
                                            let pixel = match p_byte >> (7-y) & 1 {
                                                0 => image::Rgb([255,255,255]),
                                                1 => image::Rgb([0,0,0]),
                                                _ => unreachable!(),
                                            };
                                            let pixel_x = x + head_x;
                                            let pixel_y = y + *head_y + i as u32 * 8;
                                            if pixel_x < page_width && pixel_y < page_height {
                                                img.put_pixel(pixel_x, pixel_y, pixel);
                                                *covered_y = (*covered_y).max(pixel_y);
                                            }
                                        } else {
                                            if p_byte >> (7-y) & 1 != 0 {
                                                let pixel_x = x + head_x;
                                                let pixel_y = y + *head_y + i as u32 * 8;
                                                if pixel_x < page_width && pixel_y < page_height {
                                                    let mut pixel = img.get_pixel(pixel_x, pixel_y).clone();
                                                    match self.color {
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
                                                    *covered_y = (*covered_y).max(pixel_y);
                                                }
                                            }
                                        }
                                    }
                                }
                                drop(img);
                            }
                            head_x += col_count;
                            *covered_x = (*covered_x).max(head_x);
                        },
                        _ => {
                            eprintln!("Warning: unsupported escape code {:x}", b[0]);
                        },
                    }
                },
                // line feed
                0x0a => {
                    head_x = 0;
                    *head_y += 48;
                },
                // carriage return / color change
                0x0d => {
                    head_x = 0;
                    if self.color > 0 {
                        self.color += 1;
                        if self.color > 3 {
                            self.color = 1;
                        }
                    }
                }
                _ => {}
            };
        }
        (*covered_x, *covered_y)
    }
}

#[derive(Default)]
pub struct Cz6pv1 {

}

impl Cz6pv1 {
    const PAGE_WIDTH: u32 = 0x200;
    const PAGE_HEIGHT: u32 = 992;
    const Y_MIN: u16 = 0x80;
    const Y_MAX: u16 = 0xBF;
    const M_MIN: u16 = 0x40;
    const M_MAX: u16 = 0x7E;
    const C_MIN: u16 = 0x00;
    const C_MAX: u16 = 0x3E;
}

impl Printer for Cz6pv1 {
    fn create_image(&self) -> RgbImage {
        RgbImage::from_pixel(Cz6pv1::PAGE_WIDTH, Cz6pv1::PAGE_HEIGHT, image::Rgb([255,255,255]))
    }

    fn decode(&mut self, input: &mut dyn Read, img_mutex: &Mutex<RgbImage>) -> (u32, u32) {
        let mut c: [u8; 1] = [0; 1];
        match input.read_exact(&mut c) {
            Ok(()) => {},
            Err(_) => return (0, 0),
        };
        match c[0] {
            0xC0 => {
                for plane in 0..3 {
                    for y in 0..992 {
                        let mut line = [0; 0x200];
                        if !input.read_exact(&mut line).is_ok() {
                            eprintln!("cz-6pv1: not enough data for image");
                            return (0, 0)
                        }
                        let mut img = img_mutex.lock().unwrap();
                        for (x, val) in line.into_iter().enumerate() {
                            let mut pixel = img.get_pixel(x as u32, y as u32).clone();
                            let half_offset = if y > 511 { 7 } else { 0 };
                            let val_scaled = if y < 512 {
                                (val & 0b00111111) * 4
                            } else {
                                ((((val & 0b00111111) - 7) as f32) * 4.5) as u8
                            };
                            match plane {
                                0 => {
                                    pixel[2] = val_scaled;
                                },
                                1 => {
                                    pixel[1] = val_scaled;
                                },
                                2 => {
                                    pixel[0] = val_scaled;
                                }
                                _ => unreachable!()
                            }
                            img.put_pixel(x as u32, y as u32, pixel);
                        }
                        drop(img)
                    }
                }
            }
            _ => {
                eprintln!("cz-6pv1: unknown command");
                return (0, 0)
            }
        }
        (Cz6pv1::PAGE_WIDTH, Cz6pv1::PAGE_HEIGHT)
    }
}
