use crate::{core::Node, gfx::Transform};
use specs::{Component, DenseVecStorage};
use specs_derive::Component;

#[derive(Component)]
pub struct GeomSphere {
  pub center: glam::Vec3,
  pub radius: f32,
}
impl GeomSphere {
  pub fn new(center: glam::Vec3, radius: f32) -> Node {
    let node = Node::new();
    node.add_component(Transform::from_translation(center));
    node.add_component(GeomSphere { center, radius });
    node
  }
}
