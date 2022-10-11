use specs::{Component, DenseVecStorage};
use specs_derive::Component;

#[derive(Component, Default)]
pub struct Transform {
  affine: glam::Affine3A,
}
impl Transform {
  pub fn new() -> Self {
    Self {
      affine: glam::Affine3A::IDENTITY,
    }
  }
  pub fn translation(&self) -> glam::Vec3A {
    self.affine.translation
  }
  pub fn from_translation(translation: glam::Vec3) -> Self {
    Self {
      affine: glam::Affine3A::from_translation(translation),
    }
  }
  pub fn from_translation_rotation(translation: glam::Vec3, rotation: glam::Quat) -> Self {
    Self {
      affine: glam::Affine3A::from_rotation_translation(rotation, translation),
    }
  }
  pub fn from_translation_rotation_scale(
    translation: glam::Vec3,
    rotation: glam::Quat,
    scale: glam::Vec3,
  ) -> Self {
    Self {
      affine: glam::Affine3A::from_scale_rotation_translation(scale, rotation, translation),
    }
  }
  pub fn look_at(eye: glam::Vec3, target: glam::Vec3, up: glam::Vec3) -> Self {
    let ez = (eye - target).normalize();
    let ex = up.cross(ez).normalize();
    let ey = ez.cross(ex);
    let affine = glam::Affine3A::from_mat3_translation(glam::Mat3::from_cols(ex, ey, ez), eye);
    Self { affine }
  }
  pub fn affine(&self) -> &glam::Affine3A {
    &self.affine
  }
  pub fn to_matrix(&self) -> glam::Mat4 {
    glam::Mat4::from(self.affine)
  }
}
impl From<glam::Affine3A> for Transform {
  fn from(affine: glam::Affine3A) -> Self {
    Self { affine }
  }
}
