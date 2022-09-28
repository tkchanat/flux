use crate::ecs::storage::Component;

pub struct Transform {
  affine: glam::Affine3A,
}
impl Component for Transform {}
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

pub struct GeomSphere {
  radius: f32,
}
impl Component for GeomSphere {}
impl GeomSphere {
  pub fn new(radius: f32) -> Self {
    Self { radius }
  }
}