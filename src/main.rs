extern crate pixelflut;
#[macro_use]
extern crate clap;
extern crate image;
extern crate rand;

use clap::App;
use image::GenericImageView;
use image::gif::{Decoder, Encoder};
use image::{ImageDecoder, AnimationDecoder};
use rand::prelude::*;

use std::net::SocketAddr;
use std::fs::File;
use std::iter::Iterator;

fn main() -> Result<(), Box<std::error::Error>> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let host: SocketAddr = matches.value_of("server").unwrap_or("127.0.0.1:1337").parse().unwrap();
    let file = matches.value_of("FILE").unwrap();

    let mut rng = rand::thread_rng();

    print!("opening {} ...", file);
    let file_in = File::open(file)?;
    let mut decoder = Decoder::new(file_in)?;
    let frames = decoder.into_frames();
    let frames = frames.collect_frames()?;
    let img = image::open(file).unwrap();
    println!("finished");

    print!("connecting to {} ...", host);
    let mut client = pixelflut::sync::Client::connect(host).unwrap();
    println!("finished");

    //let (width, height) = img.dimensions();

    let images: Vec<_> = frames.iter().map(|frame| {
        let delay: u32 = frame.delay().to_integer().into();
        let (left, top)  = (frame.left(), frame.top());

        let img: image::DynamicImage = image::DynamicImage::ImageRgba8(frame.clone().into_buffer());
        (delay, left, top, img)
    }).collect();

    loop {
        for (delay, left, top, img) in &images {
            let start = std::time::Instant::now();

            let mut pixels: Vec<_> = img.pixels().collect();

            if matches.is_present("random") {
                rng.shuffle(&mut pixels);
            }

            for (x, y, pixel) in pixels {
                let [r, g, b, a] = pixel.data;
                client.set(pixelflut::Pixel::new((x+left, y+top), (r, g, b, a)))?;
            }

            let mut wait: i128 = *delay as i128 - start.elapsed().as_millis() as i128;
            if wait < 0 {
                wait = 0;
            }

            println!("Delay: {}/{}", wait, delay);
            std::thread::sleep_ms(wait as u32);

            //println!("dimensions: {:?}", img.dimensions());
        }
    }


    Ok(())
}
