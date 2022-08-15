use glam::Vec3A;

pub struct Ray {
  pub origin: Vec3A,
  pub direction: Vec3A,
  pub t_min: f32,
  pub t_max: f32,
}

impl Ray {
  pub fn new(origin: Vec3A, direction: Vec3A) -> Self {
    Self {
      origin,
      direction,
      t_min: 0.0,
      t_max: f32::INFINITY,
    }
  }
}