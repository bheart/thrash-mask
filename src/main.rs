#![feature(step_by)]

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;

extern crate getopts;

use getopts::Options;

extern crate image;

use image::{DynamicImage, GenericImage, ImageBuffer, ImageResult, Pixel};

fn encode_layer(input: &DynamicImage, quality: u8) -> io::Result<Vec<u8>> {
    use image::jpeg::JPEGEncoder;

    let mut buf: Vec<u8> = vec![];

    {
        let mut enc = JPEGEncoder::new_with_quality(&mut buf, quality);

        let (width, height) = input.dimensions();

        try!(enc.encode(&input.raw_pixels(), width, height, input.color()));
    }

    Ok(buf)
}

fn decode_layer(buf: Vec<u8>) -> ImageResult<DynamicImage> {
    use std::io::Cursor;

    use image::ImageFormat;

    image::load(Cursor::new(buf), ImageFormat::JPEG)
}

fn generate_layer(input: &DynamicImage, quality: u8) -> Result<DynamicImage, Box<Error>> {
    let buf = try!(encode_layer(input, quality).map_err(Box::new));

    Ok(decode_layer(buf).unwrap()) // FIXME
}

fn generate_layers(input: &DynamicImage, count: usize) -> Result<Vec<DynamicImage>, Box<Error>> {
    (0..100).step_by(100 / count).map(|quality| {
        generate_layer(input, quality as u8)
    }).collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("i", "input", "set input file", "INPUT");
    opts.optopt("m", "mask", "set mask file", "MASK");
    opts.optopt("o", "output", "set output file", "OUTPUT");
    opts.optopt("l", "layers", "set layer count", "LAYERS");

    let matches = opts.parse(&args[1..]).expect("failed to parse args");

    let input = matches.opt_str("i").expect("no input specified");
    let mask = matches.opt_str("m").expect("no mask specified");
    let output = matches.opt_str("o").expect("no output specified");
    let count = matches.opt_str("l").expect("no layer count specified");
    let count: usize = count.parse().expect("invalid layer count");

    if count > 100 {
        panic!("layer count cannot exceed 100");
    }

    let input = image::open(input).expect("failed to open input image");
    let mask = image::open(mask).expect("failed to open mask image");

    if input.dimensions() != mask.dimensions() {
        panic!("invalid mask/input dimensions");
    }

    let layers = generate_layers(&input, count).expect("failed to generate layers");

    let (width, height) = input.dimensions();
    let mut buf = ImageBuffer::new(width, height);

    for (x, y, pixel) in buf.enumerate_pixels_mut() {
        let weight = mask.get_pixel(x, y).to_luma().channels()[0] as usize;

        let idx = weight * (count - 1) / 255;

        *pixel = layers[idx].get_pixel(x, y);
    }

    let ref mut fout = File::create(output).expect("failed to create output");

    let _ = image::ImageRgba8(buf).save(fout, image::JPEG);

    println!("ok!");
}
