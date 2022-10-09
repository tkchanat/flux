use crate::{core::Node, gfx::Transform};
use specs::{Component, DenseVecStorage};
use specs_derive::Component;

pub enum Projection {
  Perspective {
    field_of_view: f32,
    aspect: f32,
  },
  Orthographic {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
  },
}

#[derive(Component)]
pub struct Camera {
  pub projection: Projection,
  pub clipping_planes: (f32, f32),
}
impl Camera {
  pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Node {
    let node = Node::new();
    node.add_component(Transform::default());
    node.add_component(Camera {
      projection: Projection::Perspective {
        field_of_view: fov_y,
        aspect,
      },
      clipping_planes: (near, far),
    });
    node
  }
}
