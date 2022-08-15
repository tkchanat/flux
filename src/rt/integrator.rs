use crate::math::{Color, Ray};
use super::{hit::Hit, scene::Scene, RenderSettings};

pub trait Integrator {
  fn li(&self, scene: &Scene, ray: &Ray) -> Color;
}

pub struct PathIntegrator {
  max_bounce: u32,
  rr_threshold: f32,
}

impl PathIntegrator {
  pub fn new(settings: &RenderSettings) -> Self {
    Self {
      max_bounce: settings.max_bounce,
      rr_threshold: 1.0,
    }
  }
}

impl Integrator for PathIntegrator {
  fn li(&self, scene: &Scene, ray: &Ray) -> Color {
    let mut bounce = 0;
    // while true {
    //   let mut hit = Hit::default();
    // }
    Color::BLACK
  }
}
