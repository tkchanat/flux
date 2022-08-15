#[derive(Debug, Default)]
pub struct Color {
  pub r: f32,
  pub g: f32,
  pub b: f32,
}

impl Color {
  pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
  };
  pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
  };
  pub fn new(r: f32, g: f32, b: f32) -> Self {
    Self { r, g, b }
  }
}

impl Into<[u8; 4]> for Color {
  fn into(self) -> [u8; 4] {
    [
      (self.r * 255.0).clamp(0.0, 255.0) as u8,
      (self.g * 255.0).clamp(0.0, 255.0) as u8,
      (self.b * 255.0).clamp(0.0, 255.0) as u8,
      255u8,
    ]
  }
}
