mod color;
mod ray;
pub use color::*;
pub use ray::*;

use glam::Vec3A;
pub fn sample_hemisphere(theta: f32, phi: f32) -> Vec3A {
  let x = phi.cos() * theta.sin();
  let y = phi.sin() * theta.sin();
  let z = theta.cos();
  Vec3A::new(x, y, z)
}