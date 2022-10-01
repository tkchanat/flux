use specs::{Component, DenseVecStorage};
use specs_derive::Component;

#[derive(Component)]
pub struct Transform {
  affine: glam::Affine3A,
}
impl Transform {
  pub fn new() -> Self {
    Self {
      affine: glam::Affine3A::IDENTITY,
    }
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
}