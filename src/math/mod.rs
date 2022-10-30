mod color;
mod ray;
use std::f32::consts::PI;

pub use color::*;
pub use ray::*;
// pub type Transform = glam::Affine3A;

/// Uniformly distribute samples over a unit disk.
pub fn concentric_sample_disk(u: &glam::Vec2) -> glam::Vec2 {
  // map uniform random numbers to $[-1,1]^2$
  let u_offset = *u * 2.0 - 1.0;
  // handle degeneracy at the origin
  if u_offset.x == 0.0 as f32 && u_offset.y == 0.0 as f32 {
    return glam::Vec2::ZERO;
  }
  // apply concentric mapping to point
  let theta;
  let r;
  if u_offset.x.abs() > u_offset.y.abs() {
    r = u_offset.x;
    theta = (PI / 4.0) * (u_offset.y / u_offset.x);
  } else {
    r = u_offset.y;
    theta = (PI / 2.0) - (PI / 4.0) * (u_offset.x / u_offset.y);
  }
  glam::Vec2::new(theta.cos() * r, theta.sin() * r)
}

/// Cosine-weighted hemisphere sampling using Malley's method.
pub fn cosine_sample_hemisphere(u: &glam::Vec2) -> glam::Vec3A {
  let d = concentric_sample_disk(u);
  let z = (1.0 as f32 - d.x * d.x - d.y * d.y).max(0.0).sqrt();
  glam::Vec3A::new(d.x, d.y, z)
}

/// Uniformly sample rays in a full sphere. Choose a direction.
pub fn uniform_sample_sphere(u: &glam::Vec2) -> glam::Vec3A {
  let z = 1.0 - 2.0 * u.x;
  let r = (0.0f32).max(1.0 - z * z).sqrt();
  let phi = 2.0 * PI * u.y;
  glam::Vec3A::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn coordinate_system(v1: &glam::Vec3A, v2: &mut glam::Vec3A, v3: &mut glam::Vec3A) {
  if v1.x.abs() > v1.y.abs() {
    *v2 = glam::Vec3A::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt();
  } else {
    *v2 = glam::Vec3A::new(0.0, v1.z, -v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt();
  }
  *v3 = v1.cross(*v2);
}
