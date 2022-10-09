mod accelerator;
mod bsdf;
mod camera;
mod film;
mod hit;
mod integrator;
mod sampler;
mod scene;
mod shape;

pub use self::scene::SceneEngine;
use self::{
  accelerator::Accelerator,
  camera::{Camera, PinholeCamera},
  film::Film,
  integrator::{Integrator, PathIntegrator},
};
use crate::{
  core::Timer,
  raytrace::sampler::{Sampler, StratifiedSampler},
};
use glam::{Vec2, Vec3};
use std::{
  sync::{Arc, RwLock, RwLockReadGuard, Weak},
  thread,
};

#[derive(Clone)]
pub struct RenderSettings {
  pub resolution: (u32, u32),
  pub samples_per_pixel: u32,
  pub max_bounce: u32,
}

impl Default for RenderSettings {
  fn default() -> Self {
    Self {
      resolution: (640, 480),
      samples_per_pixel: 64,
      max_bounce: 8,
    }
  }
}

pub struct RenderEngine {
  pub film: Arc<RwLock<Film>>,
  pub settings: RenderSettings,
}

pub struct RenderContext {
  settings: RenderSettings,
  accelerator: Arc<Accelerator>,
  camera: Weak<dyn Camera>,
}

impl RenderEngine {
  pub fn new(settings: RenderSettings) -> Self {
    let film = Arc::new(RwLock::new(Film::new(
      settings.resolution.0,
      settings.resolution.1,
    )));
    Self { film, settings }
  }
  pub fn prepare_render(&mut self, scene: &SceneEngine) -> RenderContext {
    let timer = Timer::new();

    let accelerator = Arc::new(Accelerator::build(&scene));
    println!("BVH building took: {:?}", timer.elapsed());

    let camera = Arc::downgrade(&scene.cameras[scene.active_cam]);

    RenderContext {
      settings: self.settings.clone(),
      accelerator,
      camera,
    }
  }
  pub fn render_frame(&self, context: RenderContext) {
    let width = context.settings.resolution.0;
    let height = context.settings.resolution.1;
    let film_handle = self.film.clone();
    let camera = context.camera.upgrade().expect("Camera no longer exists");

    thread::spawn(move || {
      let timer = Timer::new();
      let integrator = PathIntegrator::new(context.settings.max_bounce);
      let mut sampler = StratifiedSampler::new();
      for spp in 1..=context.settings.samples_per_pixel {
        for y in 0..height {
          for x in 0..width {
            let jitter = sampler.get_2d() - 0.5;
            let ndc = Vec2::new(
              (x as f32 + jitter.x) / (width - 1) as f32,
              (y as f32 + jitter.y) / (height - 1) as f32,
            ) * 2.0
              - 1.0;
            let ray = camera.ray(&ndc);
            let color = integrator.li(&context.accelerator, &mut sampler, ray, 0);
            {
              let film_rw_lock = film_handle.clone();
              let mut film = film_rw_lock.write().unwrap();
              let acc_pixel = if spp == 1 {
                color
              } else {
                let p = film.pixel(x, y);
                p + (color - p) / spp as f32
              };
              film.write_pixel(x, y, acc_pixel.into());
            }
          }
        }
      }
      println!("Full render took: {:?}", timer.elapsed());
    });
  }
}
