use rand::Rng;

pub trait Sampler {
  fn get_1d(&mut self) -> f32;
  fn get_2d(&mut self) -> glam::Vec2;
}

pub struct StratifiedSampler {
  samples_per_pixel: u32,
  rng: rand::rngs::ThreadRng,
}

impl StratifiedSampler {
  pub fn new() -> Self {
    Self {
      samples_per_pixel: 64,
      rng: rand::thread_rng(),
    }
  }
}

impl Sampler for StratifiedSampler {
  fn get_1d(&mut self) -> f32 {
    self.rng.gen()
  }

  fn get_2d(&mut self) -> glam::Vec2 {
    glam::Vec2::new(self.rng.gen(), self.rng.gen())
  }
}
