use glam::{Affine3A, Mat3A, Vec2, Vec3A};

use super::shape::Shape;

pub struct Hit<'a> {
  pub shape: Option<&'a dyn Shape>,
  pub p: Vec3A,
  pub ng: Vec3A,
  pub ns: Vec3A,
  pub uv: Vec2,
  pub t: f32,
  pub dpdu: Vec3A,
  pub dpdv: Vec3A,
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
      dpdu: Vec3A::ZERO,
      dpdv: Vec3A::ZERO,
      front: false,
    }
  }
}

impl<'a> Hit<'a> {
  pub fn local_to_world(&self, v: Vec3A) -> Vec3A {
    assert!(self.ns.is_normalized());
    let tangent = self.dpdu.normalize();
    let bitangent = self.ns.cross(tangent);
    Mat3A::from_cols(tangent, bitangent, self.ns).mul_vec3a(v)
  }

  pub fn world_to_local(&self, v: Vec3A) -> Vec3A {
    assert!(self.ns.is_normalized());
    let tangent = self.dpdu.normalize();
    let bitangent = self.ns.cross(tangent);
    Mat3A::from_cols(tangent, bitangent, self.ns).transpose().mul_vec3a(v)
  }
}
