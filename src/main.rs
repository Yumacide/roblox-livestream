use anyhow::Result;
use mtpng::{
    encoder::{Encoder, Options},
    Header,
};
use screenshots::Screen;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread, time,
};

use resize::Pixel::RGBA8;
use resize::Type::Lanczos3;
use rgb::FromSlice;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 144;

// FIX: make screenshots library output rgb instead of rgba

fn main() -> Result<()> {
    let screen = Screen::all().unwrap()[0];
    let current_img: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));
    {
        let current_img = Arc::clone(&current_img);
        thread::spawn(move || -> Result<()> {
            loop {
                let start = time::Instant::now();
                let img = screen.capture().unwrap();
                let img = resize(
                    img.buffer(),
                    img.width() as usize,
                    img.height() as usize,
                    WIDTH as usize,
                    HEIGHT as usize,
                );

                let img_rgb = to_rgb(img?);
                current_img.lock().unwrap().clear();
                png_encode(&img_rgb, &mut *current_img.lock().unwrap())?;

                if start.elapsed().as_secs_f64() < 0.05 {
                    thread::sleep(time::Duration::from_secs_f64(
                        0.05 - start.elapsed().as_secs_f64(),
                    ));
                }
            }
        });
    }

    let listener = TcpListener::bind("127.0.0.1:7331")?;

    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream, &current_img.lock().unwrap())?;
    }

    Ok(())
}

fn to_rgb(img: Vec<u8>) -> Vec<u8> {
    let mut rgb = vec![0; (WIDTH * HEIGHT * 3) as usize];
    for (output, chunk) in rgb.chunks_exact_mut(3).zip(img.chunks_exact(4)) {
        output.copy_from_slice(&chunk[0..3]);
    }
    rgb
}

fn png_encode<W: Write>(img: &[u8], dst: W) -> Result<()> {
    let mut header = Header::new();
    header.set_size(WIDTH, HEIGHT)?;
    header.set_color(mtpng::ColorType::Truecolor, 8)?;

    let mut options = Options::new();
    options.set_compression_level(mtpng::CompressionLevel::Fast)?;

    let mut encoder = Encoder::new(dst, &options);

    encoder.write_header(&header)?;
    encoder.write_image_rows(img)?;
    encoder.finish()?;

    Ok(())
}

fn resize(
    src_image: &[u8],
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize,
) -> Result<Vec<u8>> {
    let mut dst_image = vec![0; dst_width * dst_height * 4];
    let mut resizer = resize::new(
        src_width, src_height, dst_width, dst_height, RGBA8, Lanczos3,
    )?;
    resizer.resize(src_image.as_rgba(), dst_image.as_rgba_mut())?;
    Ok(dst_image)
}

fn handle_connection(mut stream: TcpStream, img: &[u8]) -> Result<()> {
    let mut buf = [0];
    let mut buf_reader = BufReader::new(&mut stream);
    buf_reader.read_exact(&mut buf)?;

    stream.write_all("HTTP/1.1 200 OK \r\n\r\n".as_bytes())?;
    let mut buf = BufWriter::new(stream);
    buf.write_all(img)?;
    Ok(())
}
