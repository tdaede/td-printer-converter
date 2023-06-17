use clap::Parser;
use image::RgbImage;
use std::fs::File;
use std::io::BufReader;

mod printer;

use printer::Cz8pc4;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    input: String,

    #[arg()]
    output: String,
}

fn main() {
    let args = Args::parse();

    let input_file = File::open(args.input).unwrap();
    let mut input = BufReader::new(input_file);

    let mut img: RgbImage = Cz8pc4::create_image();

    Cz8pc4::decode(&mut input, &mut img);

    img.save(args.output).unwrap();
}
