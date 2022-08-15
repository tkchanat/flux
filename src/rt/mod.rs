use self::{
  camera::{Camera, PinholeCamera},
  hit::Hit,
  shape::{Shape, Sphere},
};
use crate::{core::Timer, math::Color, rt::{scene::Scene, accelerator::Accelerator}};
use bvh::bvh::BVH;
use glam::{Vec3, Vec3A};
use std::{
  sync::{Arc, RwLock},
  thread,
};

mod accelerator;
mod camera;
mod film;
mod hit;
mod integrator;
mod mesh;
mod scene;
mod shape;

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
  pub film: Arc<RwLock<film::Film>>,
  pub settings: RenderSettings,
}

impl RenderEngine {
  pub fn new(settings: RenderSettings) -> Self {
    let film = Arc::new(RwLock::new(film::Film::new(
      settings.resolution.0,
      settings.resolution.1,
    )));
    Self { film, settings }
  }

  pub fn render_frame(&mut self) {
    let timer = Timer::new();
    let scene = Scene::from_gltf(
      "C:/Users/tkchanat/Desktop/glTF-Sample-Models-master/2.0/Suzanne/glTF/Suzanne.gltf",
    );
    println!("Model loading took: {:?}", timer.elapsed());
    
    let width = self.settings.resolution.0;
    let height = self.settings.resolution.1;
    let film_handle = self.film.clone();
    // let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 0.5);
    let mut camera = PinholeCamera::new(45f32.to_radians(), 1.0, 0.01, 1000.0);
    camera.look_at(Vec3::new(2.0, 1.0, 2.0), Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    
    thread::spawn(move || {
      let accelerator = Accelerator::build(&scene);
      println!("BVH building took: {:?}", timer.elapsed());

      for y in 0..height {
        for x in 0..width {
          let ndc = (
            (x as f32 / (width - 1) as f32) * 2.0 - 1.0,
            (y as f32 / (height - 1) as f32) * 2.0 - 1.0,
          );
          let mut ray = camera.ray(ndc);
          let mut hit = Hit::default();
          let mut color = [0, 0, 0, 0];

          if accelerator.intersect(&ray, &mut hit) {
            if !hit.front {
              continue;
            }
            ray.t_max = hit.t;
            color = Color::new(
              (hit.ng.x + 1.0) * 0.5,
              (hit.ng.y + 1.0) * 0.5,
              (hit.ng.z + 1.0) * 0.5,
            )
            .into()
          }

          {
            let film_rw_lock = film_handle.clone();
            let mut film = film_rw_lock.write().unwrap();
            film.write_pixel(x, y, color);
          }
        }
      }
      println!("Full render took: {:?}", timer.elapsed());
    });
  }
}
