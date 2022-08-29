use glam::{Vec3, Vec3A};

use super::{
  accelerator::Accelerator,
  bsdf::{Lambertian, BSDF},
  hit::Hit,
  scene::Scene,
  RenderSettings,
};
use crate::math::{Color, Ray};

pub trait Integrator {
  fn li(&self, accel: &Accelerator, ray: Ray, bounce: u32) -> Color;
}

pub struct PathIntegrator {
  max_bounce: u32,
  rr_threshold: f32,
}

impl PathIntegrator {
  pub fn new(max_bounce: u32) -> Self {
    Self {
      max_bounce,
      rr_threshold: 1.0,
    }
  }
}

impl Integrator for PathIntegrator {
  fn li(&self, accel: &Accelerator, mut ray: Ray, bounce: u32) -> Color {
    let mut hit = Hit::default();
    let found_intersection = accel.intersect(&ray, &mut hit);
    if !found_intersection || bounce >= self.max_bounce {
      return Color::WHITE;
    }

    let wo = -ray.direction;
    let mut wi = Vec3A::default();
    let mut pdf = 0.0;
    let bsdf = Lambertian::default();
    let f = bsdf.sample(&hit, &wo, &mut wi, &mut pdf);
    // if f == Color::BLACK || pdf == 0.0 {
    //   return Color::BLACK;
    // }

    let le = Color::BLACK;
    let cosine = wi.dot(hit.ns).max(0.0);
    let new_ray = Ray {
      origin: hit.p,
      direction: wi,
      t_min: 0.001,
      t_max: f32::INFINITY,
    };
    // le + f * self.li(accel, new_ray, bounce + 1) * cosine
    hit.ns.into()
  }
}
