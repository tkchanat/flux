use glam::{Vec3A, Vec2, Affine3A};

use super::shape::Shape;

pub struct Hit<'a> {
  pub shape: Option<&'a dyn Shape>,
  pub p: Vec3A,
  pub ng: Vec3A,
  pub ns: Vec3A,
  pub uv: Vec2,
  pub t: f32,
  pub frame: Affine3A,
  pub front: bool,
}

impl<'a> Default for Hit<'a> {
  fn default() -> Self {
    Self {
      shape: None,
      p: Vec3A::ZERO,
      ng: Vec3A::ZERO,
      ns: Vec3A::ZERO,
      uv: Vec2::ZERO,
      t: f32::INFINITY,
      frame: Affine3A::IDENTITY,
      front: false,
    }
  }
}
