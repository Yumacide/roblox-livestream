use fast_image_resize::{self as fr, Image, PixelType};
use screenshots::Screen;
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    num::NonZeroU32,
    sync::{Arc, Mutex},
    thread, time,
};

fn main() {
    let screen = Screen::all().unwrap()[0];
    let current_img: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));
    {
        let current_img = Arc::clone(&current_img);
        thread::spawn(move || loop {
            let _img = screen.capture().unwrap();

            let mut img = Image::from_vec_u8(
                NonZeroU32::new(_img.width()).unwrap(),
                NonZeroU32::new(_img.height()).unwrap(),
                _img.buffer().to_vec(),
                PixelType::U8x4,
            )
            .unwrap();

            let dst_width = NonZeroU32::new(192).unwrap();
            let dst_height = NonZeroU32::new(108).unwrap();
            let img = resize(&mut img, dst_width, dst_height);

            *current_img.lock().unwrap() = img.buffer().to_vec();

            thread::sleep(time::Duration::from_millis(100));
        });
    }

    let listener = TcpListener::bind("127.0.0.1:7331").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, current_img.lock().unwrap().to_vec());
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

fn handle_connection(mut stream: TcpStream, img: Vec<u8>) {
    let mut str = String::new();
    let mut i = 0;
    for n in img.iter() {
        i += 1;
        if i % 4 == 0 {
            continue;
        }
        str.push(*n as char);
    }

    let response = format!("HTTP/1.1 200 OK \r\n\r\n{}", str);
    stream.write_all(response.as_bytes()).unwrap();
    println!("{} pixels", str.len() / 3);
}
