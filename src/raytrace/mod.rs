mod accelerator;
mod bsdf;
mod camera;
mod film;
mod hit;
mod integrator;
mod mesh;
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
  sync::{Arc, RwLock},
  thread,
};

pub struct RenderSettings {
  pub resolution: (u32, u32),
  pub max_bounce: u32,
}

impl Default for RenderSettings {
  fn default() -> Self {
    Self {
      resolution: (640, 480),
      max_bounce: 8,
    }
  }
}

pub struct RenderEngine {
  pub film: Arc<RwLock<Film>>,
  pub settings: RenderSettings,
}

impl RenderEngine {
  pub fn new(settings: RenderSettings) -> Self {
    let film = Arc::new(RwLock::new(Film::new(
      settings.resolution.0,
      settings.resolution.1,
    )));
    Self { film, settings }
  }

  pub fn prepare_scene(&mut self, scene: &SceneEngine) {}

  pub fn render_frame<'a>(&mut self, scene: Arc<RwLock<SceneEngine>>) {
    let timer = Timer::new();
    // let scene = Scene::from_gltf(
    //   "C:/Users/tkchanat/Desktop/glTF-Sample-Models-master/2.0/Suzanne/glTF/Suzanne.gltf",
    // );
    // let scene = Scene::raytracing_in_one_weekend();
    println!("Scene loading took: {:?}", timer.elapsed());

    let width = self.settings.resolution.0;
    let height = self.settings.resolution.1;
    let samples_per_pixel = 64;
    let film_handle = self.film.clone();
    let mut camera = PinholeCamera::new(60f32.to_radians(), 1.0, 0.01, 1000.0);
    camera.look_at(Vec3::new(2.0, 1.0, 2.0), Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    
    thread::spawn(move || {
      let accelerator = Accelerator::build(&scene.read().unwrap());
      println!("BVH building took: {:?}", timer.elapsed());
      
      let integrator = PathIntegrator::new(8);
      let mut sampler = StratifiedSampler::new();
      for spp in 1..=samples_per_pixel {
        for y in 0..height {
          for x in 0..width {
            let jitter = sampler.get_2d() - 0.5;
            let ndc = Vec2::new(
              (x as f32 + jitter.x) / (width - 1) as f32,
              (y as f32 + jitter.y) / (height - 1) as f32,
            ) * 2.0
              - 1.0;
            let ray = camera.ray(&ndc);
            let color = integrator.li(&accelerator, &mut sampler, ray, 0);
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
