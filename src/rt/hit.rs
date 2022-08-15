use glam::Vec3A;

use super::shape::Shape;

pub struct Hit<'a> {
  pub shape: Option<&'a dyn Shape>,
  pub p: Vec3A,
  pub ng: Vec3A,
  pub ns: Vec3A,
  pub u: f32,
  pub v: f32,
  pub t: f32,
  pub front: bool,
}

impl<'a> Default for Hit<'a> {
  fn default() -> Self {
    Self {
      shape: None,
      p: Vec3A::ZERO,
      ng: Vec3A::ZERO,
      ns: Vec3A::ZERO,
      u: 0.0,
      v: 0.0,
      t: f32::INFINITY,
      front: false,
    }
  }
}
