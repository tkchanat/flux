use super::shape::{Shape, Triangle};
use bvh::aabb::AABB;
use glam::Affine3A;

pub struct Mesh<T> {
  pub shapes: Vec<T>,
  pub transform: Affine3A,
  pub local_bound: AABB,
}

impl<T> Mesh<T> {
  fn subdivide(&mut self) {
    todo!()
  }
}

pub type TriangleMesh = Mesh<Triangle>;