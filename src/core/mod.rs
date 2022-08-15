use std::time::{Duration, Instant};

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
