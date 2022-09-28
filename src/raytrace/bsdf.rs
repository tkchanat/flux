use std::f32::consts::PI;

use super::hit::Hit;
use crate::math::{cosine_sample_hemisphere, uniform_sample_sphere, Color};
use glam::Vec3A;

pub trait BSDF {
  fn eval(&self, hit: &Hit, wo: &Vec3A, wi: &Vec3A, pdf: &mut f32) -> Color;
  fn sample(&self, hit: &Hit, wo: &Vec3A, wi: &mut Vec3A, pdf: &mut f32, sample: &glam::Vec2) -> Color;
}

pub struct Lambertian {
  diffuse_color: Color,
}

impl Lambertian {
  pub fn new(diffuse_color: Color) -> Self {
    Self { diffuse_color }
  }
}

impl Default for Lambertian {
  fn default() -> Self {
    Self {
      diffuse_color: Color::new(0.8, 0.8, 0.8),
    }
  }
}

impl BSDF for Lambertian {
  fn eval(&self, hit: &Hit, _wo: &Vec3A, wi: &Vec3A, pdf: &mut f32) -> Color {
    let cos_theta_i = hit.ns.dot(*wi);
    if cos_theta_i <= 0.0 {
      *pdf = 0.0;
      return Color::BLACK;
    }
    *pdf = cos_theta_i / PI;
    self.diffuse_color
  }

  fn sample(&self, hit: &Hit, _wo: &Vec3A, wi: &mut Vec3A, pdf: &mut f32, sample: &glam::Vec2) -> Color {
    *wi = hit.local_to_world(cosine_sample_hemisphere(sample));

    let cos_theta_i = hit.ns.dot(*wi);
    if cos_theta_i <= 0.0 {
      *pdf = 0.0;
      *wi = Vec3A::ZERO;
      return Color::BLACK;
    }
    *pdf = cos_theta_i / PI;
    self.diffuse_color
  }
}
