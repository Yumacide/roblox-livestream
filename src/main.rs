use fast_image_resize::{self as fr, Image, PixelType};
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use screenshots::Screen;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
    num::NonZeroU32,
    path::Path,
    thread, time,
};

fn main() {
    println!("Hello, world!");
    let screen = Screen::all().unwrap()[0];
    let mut current_img: Vec<u8> = vec![];
    thread::spawn(move || loop {
        let start = time::Instant::now();
        let img = screen.capture().unwrap();

        println!("captured screen in {}ms", start.elapsed().as_millis());

        let mut img = Image::from_vec_u8(
            NonZeroU32::new(img.width()).unwrap(),
            NonZeroU32::new(img.height()).unwrap(),
            img.buffer().to_vec(),
            PixelType::U8x4,
        )
        .unwrap();

        let dst_width = NonZeroU32::new(854).unwrap();
        let dst_height = NonZeroU32::new(480).unwrap();
        let img = resize(&mut img, dst_width, dst_height);
        println!("resized in {}ms", start.elapsed().as_millis());
        current_img = img.buffer().to_vec();

        // Write destination image as PNG-file
        let path = Path::new(r"target/image.png");
        let file = File::create(path).unwrap();
        let mut result_buf = BufWriter::new(file);
        PngEncoder::new(&mut result_buf)
            .write_image(
                img.buffer(),
                dst_width.get(),
                dst_height.get(),
                ColorType::Rgba8,
            )
            .unwrap();

        thread::sleep(time::Duration::from_millis(5000));
    });

    let listener = TcpListener::bind("127.0.0.1:7331").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
        println!("Connection established!");
    }
}

fn resize<'a>(
    src_image: &'a mut Image<'a>,
    dst_width: NonZeroU32,
    dst_height: NonZeroU32,
) -> Image<'a> {
    // Multiple RGB channels of source image by alpha channel
    // (not required for the Nearest algorithm)
    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .unwrap();

    // Create container for data of destination image
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    // Get mutable view of destination image data
    let mut dst_view = dst_image.view_mut();

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
    resizer.resize(&src_image.view(), &mut dst_view).unwrap();

    // Divide RGB channels of destination image by alpha
    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();
    dst_image
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
    println!("Request: {:#?}", http_request);
}
