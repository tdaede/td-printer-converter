use clap::Parser;
use image::RgbImage;
use image::imageops;
use ipp::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::time::Duration;
use std::sync::Mutex;
use turbojpeg;

mod printer;
mod gui;

use printer::Cz8pc4;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    input: Option<String>,

    #[arg(long, short)]
    output: Option<String>,

    #[arg(long)]
    serial: Option<String>,

    #[arg(long)]
    print: Option<String>,

    #[arg(long)]
    gui: bool,
}

fn ipp_print(print: &String, img_cropped: RgbImage) {
    let uri: Uri = print.clone().parse().unwrap();
    let client = IppClient::new(uri.clone());
    let jpeg_data = turbojpeg::compress_image(&img_cropped, 100, turbojpeg::Subsamp::None).unwrap();
    let jpeg_data_read = jpeg_data.to_vec();
    let ipp_payload = IppPayload::new(Cursor::new(jpeg_data_read));
    let print_operation = IppOperationBuilder::print_job(uri, ipp_payload)
        .attribute(IppAttribute::new("document-format", IppValue::MimeMediaType("image/jpeg".to_string())))
        .attribute(IppAttribute::new("print-scaling", IppValue::Keyword("fit".to_string())))
        .attribute(IppAttribute::new("media-col", IppValue::Collection(vec!(
            IppValue::MemberAttrName("media-bottom-margin".to_string()), IppValue::Integer(0),
            IppValue::MemberAttrName("media-left-margin".to_string()), IppValue::Integer(0),
            IppValue::MemberAttrName("media-right-margin".to_string()), IppValue::Integer(0),
            IppValue::MemberAttrName("media-top-margin".to_string()), IppValue::Integer(0),
        ))))
        .build();
    let resp = client.send(print_operation).unwrap();
    if resp.header().status_code().is_success() {
        eprintln!("Sent to printer!");
    } else {
        eprintln!("Failed to send to printer!");
        dbg!(resp.attributes());
    }
}

fn main() {
    let args = Args::parse();

    let output = |img: &mut RgbImage, covered_x, covered_y| {
        if covered_x == 0 || covered_y == 0 {
            eprintln!("Page is blank, not printing!");
            return;
        }
        let img_cropped = imageops::crop(img, 0, 0, covered_x, covered_y).to_image();
        if let Some(ref print) = args.print {
            ipp_print(print, img_cropped);
        } else {
            img_cropped.save(args.output.as_ref().expect("Output filename not provided")).unwrap();
        }
    };

    if args.gui {
        gui::gui_main();
    } else if let Some(serial_port_name) = args.serial {
        // serial mode
        let mut serial_port = serialport::new(&serial_port_name, 500_000).timeout(Duration::from_secs(60)).open().expect("Failed to open port");
        eprintln!("Serial port opened on {}", serial_port_name);
        loop {
            let mut print_job_bytes = Vec::new();
            let mut buf = [0; 128];
            while let Ok(bytes_read) = serial_port.read(&mut buf) {
                print_job_bytes.extend_from_slice(&mut buf[0..bytes_read]);
                eprint!("Print job in progress, read {} bytes...\r", print_job_bytes.len());
            };
            if print_job_bytes.len() == 0 {
                continue;
            }
            eprintln!("Print job of {} bytes complete            ", print_job_bytes.len());
            let img = Cz8pc4::create_image();
            let img_mutex = Mutex::new(img);
            let (covered_x, covered_y) = Cz8pc4::decode(&mut print_job_bytes.as_slice(), &img_mutex);
            let mut img_ = img_mutex.lock().unwrap();
            output(&mut img_, covered_x, covered_y);
            drop(img_);
        }
    } else {
        // file mode
        let input_file = File::open(args.input.unwrap()).unwrap();
        let mut input = BufReader::new(input_file);

        let img = Cz8pc4::create_image();
        let img_mutex = Mutex::new(img);

        let (covered_x, covered_y) = Cz8pc4::decode(&mut input, &img_mutex);

        let mut img_ = img_mutex.lock().unwrap();
        output(&mut img_, covered_x, covered_y);
    }
}
