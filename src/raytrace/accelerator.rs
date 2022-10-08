use super::{
  hit::Hit,
  scene::{Primitive, SceneEngine},
  shape::{Shape, Sphere, Triangle},
};
use crate::math::Ray;
use bvh::{
  aabb::{Bounded, AABB},
  bounding_hierarchy::BHShape,
  bvh::BVH,
};
use std::collections::VecDeque;

struct L1Node {
  l2_bvh: BVH,
  bound: AABB,
  l2nodes: Vec<L2Node>,
  node_index: usize,
}
impl Bounded for L1Node {
  fn aabb(&self) -> bvh::aabb::AABB {
    self.bound
  }
}
impl BHShape for L1Node {
  fn set_bh_node_index(&mut self, node_index: usize) {
    self.node_index = node_index;
  }
  fn bh_node_index(&self) -> usize {
    self.node_index
  }
}

struct L2Node {
  shape: Shape,
  node_index: usize,
}
impl Bounded for L2Node {
  fn aabb(&self) -> bvh::aabb::AABB {
    self.shape.aabb()
  }
}
impl BHShape for L2Node {
  fn set_bh_node_index(&mut self, node_index: usize) {
    self.node_index = node_index;
  }
  fn bh_node_index(&self) -> usize {
    self.node_index
  }
}

pub struct Accelerator {
  l1_bvh: BVH,
  l1nodes: Vec<L1Node>,
}
impl Accelerator {
  pub(super) fn build(scene: &SceneEngine) -> Self {
    let mut l1nodes = Vec::new();
    let mut stack = VecDeque::new();
    stack.push_back(&scene.root);
    while !stack.is_empty() {
      if let Some(current_node) = stack.pop_front() {
        // Process primitive
        let mut bound = AABB::empty();
        let mut l2nodes = Vec::new();
        match &current_node.prim {
          Primitive::Empty => (),
          Primitive::Sphere(center, radius) => {
            let sphere = Sphere::new(*center, *radius);
            bound.join_mut(&sphere.aabb());
            l2nodes.push(L2Node {
              shape: Shape::Sphere(sphere),
              node_index: 0,
            })
          }
          Primitive::TriangleMesh(tri_mesh) => {
            for id in 0..tri_mesh.tri_count {
              let triangle = Triangle::new(tri_mesh.clone(), id);
              l2nodes.push(L2Node {
                shape: Shape::Triangle(triangle),
                node_index: 0,
              })
            }
          }
          _ => (),
        }

        if !l2nodes.is_empty() {
          let l1node = L1Node {
            l2_bvh: BVH::build(&mut l2nodes),
            bound,
            l2nodes,
            node_index: 0,
          };
          l1nodes.push(l1node);
        }

        // Push the remaining children
        for child in &current_node.children {
          stack.push_back(&child);
        }
      }
    }
    Self {
      l1_bvh: BVH::build(&mut l1nodes),
      l1nodes,
    }
  }

  pub(super) fn intersect<'a>(&'a self, ray: &Ray, hit: &mut Hit<'a>) -> bool {
    let mut any_hit = false;
    let bvh_ray = bvh::ray::Ray::new(
      bvh::Point3::new(ray.origin.x, ray.origin.y, ray.origin.z),
      bvh::Vector3::new(ray.direction.x, ray.direction.y, ray.direction.z),
    );
    let mut closest_hit = f32::INFINITY;
    for l1 in self.l1_bvh.traverse(&bvh_ray, &self.l1nodes) {
      for l2 in l1.l2_bvh.traverse(&bvh_ray, &l1.l2nodes) {
        let mut tmp_hit = Hit::default();
        if l2.shape.intersect(ray, &mut tmp_hit) && tmp_hit.front {
          any_hit = true;
          if tmp_hit.t < closest_hit {
            closest_hit = tmp_hit.t;
            *hit = tmp_hit;
            hit.shape = Some(&l2.shape);
          }
        }
      }
    }
    any_hit
  }
}
