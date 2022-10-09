use glam::{Affine3A, Mat3, Mat4, Vec2, Vec3, Vec3A};

use crate::math::Ray;

pub trait Camera: Sync + Send {
  fn ray(&self, ndc: &Vec2) -> Ray;
}

pub struct PinholeCamera {
  fov_y: f32,
  aspect: f32,
  near: f32,
  far: f32,
  world_to_view: Affine3A,
  view_to_world: Affine3A,
  view_to_ndc: Mat4,
  ndc_to_view: Mat4,
}

impl PinholeCamera {
  pub fn new(fov_y: f32, aspect: f32, near: f32, far: f32, world_to_view: glam::Affine3A) -> Self {
    let view_to_ndc = Mat4::perspective_rh(fov_y, aspect, near, far);
    Self {
      fov_y,
      aspect,
      near,
      far,
      world_to_view,
      view_to_world: world_to_view.inverse(),
      view_to_ndc,
      ndc_to_view: view_to_ndc.inverse(),
    }
  }
}

impl Camera for PinholeCamera {
  fn ray(&self, ndc: &Vec2) -> Ray {
    let far_plane_hy = self.far * (self.fov_y * 0.5).tan();
    let far_plane_hx = self.aspect * far_plane_hy;
    let direction = Vec3A::new(
      ndc.x as f32 * far_plane_hx,
      ndc.y as f32 * far_plane_hy,
      -self.far,
    ).normalize();
    let origin = self.world_to_view.translation;
    let direction = self.world_to_view.transform_vector3a(direction);
    Ray::new(origin, direction)
  }
}
