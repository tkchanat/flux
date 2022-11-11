pub mod app;
pub mod input;
pub mod node;
pub mod scene;
pub use app::*;
pub use input::*;
pub use node::Node;
pub use scene::*;

// use std::time::{Duration, Instant};
use instant::{Duration, Instant};

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
