use std::ops::{Add, Div, Mul, Sub};

#[derive(Copy, Clone, Debug, Default)]
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

impl PartialEq for Color {
  fn eq(&self, other: &Self) -> bool {
    self.r == other.r && self.g == other.g && self.b == other.b
  }
}

impl Add for Color {
  type Output = Color;
  fn add(self, rhs: Self) -> Self::Output {
    Self::Output::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
  }
}

impl Sub for Color {
  type Output = Color;
  fn sub(self, rhs: Self) -> Self::Output {
    Self::Output::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b)
  }
}

impl Mul<Color> for Color {
  type Output = Color;
  fn mul(self, rhs: Self) -> Self::Output {
    Self::Output::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
  }
}

impl Mul<f32> for Color {
  type Output = Color;
  fn mul(self, scalar: f32) -> Self::Output {
    Self::Output::new(self.r * scalar, self.g * scalar, self.b * scalar)
  }
}

impl Div<f32> for Color {
  type Output = Color;
  fn div(self, scalar: f32) -> Self::Output {
    Self::Output::new(self.r / scalar, self.g / scalar, self.b / scalar)
  }
}

impl From<glam::Vec3A> for Color {
  fn from(v: glam::Vec3A) -> Self {
    Self::new(v.x, v.y, v.z)
  }
}
