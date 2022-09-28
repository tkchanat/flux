use crate::math::{self, Color};

pub struct Film {
  dimension: (u32, u32),
  data: Vec<[u8; 4]>,
}

impl Film {
  pub fn new(width: u32, height: u32) -> Self {
    Self {
      dimension: (width, height),
      data: vec![[0; 4]; (width * height) as usize],
    }
  }

  pub fn width(&self) -> u32 {
    self.dimension.0
  }

  pub fn height(&self) -> u32 {
    self.dimension.1
  }

  pub fn x_stride(&self) -> usize {
    std::mem::size_of::<[u8; 4]>()
  }

  pub fn y_stride(&self) -> usize {
    self.x_stride() * self.dimension.0 as usize
  }

  pub fn data(&self) -> &[u8] {
    unsafe {
      std::slice::from_raw_parts(
        self.data.as_ptr() as *const u8,
        self.data.len() * self.x_stride(),
      )
    }
  }

  pub fn pixel(&self, x: u32, y: u32) -> Color {
    let data = self.data[(y * self.dimension.0 + x) as usize];
    Color::new(
      data[0] as f32 / 255.0,
      data[1] as f32 / 255.0,
      data[2] as f32 / 255.0,
    )
  }

  pub fn write_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
    self.data[(y * self.dimension.0 + x) as usize] = color;
  }
}
