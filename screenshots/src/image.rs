use png::EncodingError;

pub struct Image {
  width: u32,
  height: u32,
  buffer: Vec<u8>,
}

impl Image {
  pub fn new(width: u32, height: u32, buffer: Vec<u8>) -> Self {
    Image {
      width,
      height,
      buffer,
    }
  }

  // edited - don't know if this'll break anything
  pub fn from_bgra(width: u32, height: u32, mut bgra: Vec<u8>) -> Result<Self, EncodingError> {
    // BGRA 转换为 RGBA
    for i in (0..bgra.len()).step_by(4) {
      let b = bgra[i];
      let r = bgra[i + 2];

      bgra[i] = r;
      bgra[i + 2] = b;
      bgra[i + 3] = 255;
    }

    Ok(Image::new(width, height, bgra))
  }

  pub fn width(&self) -> u32 {
    self.width
  }

  pub fn height(&self) -> u32 {
    self.height
  }

  pub fn buffer(&self) -> &Vec<u8> {
    &self.buffer
  }
}
