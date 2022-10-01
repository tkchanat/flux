mod scene;
mod prefabs;
pub use scene::*;

// use std::time::{Duration, Instant};
use instant::{Instant, Duration};

pub struct Timer {
  instant: Instant,
}

impl Timer {
  pub fn new() -> Self {
    Self {
      instant: Instant::now(),
    }
  }

  pub fn elapsed(&self) -> Duration {
    self.instant.elapsed()
  }
}
