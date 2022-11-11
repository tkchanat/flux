use crate::core::{node::Component, Node};
use crate::gfx::Transform;

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

pub struct Camera {
  pub projection: Projection,
  pub clipping_planes: (f32, f32),
}
impl Camera {
  pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Node {
    let node = Node::new("camera");
    // node.add_component(Transform::default());
    // node.add_component(Camera {
    //   projection: Projection::Perspective {
    //     field_of_view: fov_y,
    //     aspect,
    //   },
    //   clipping_planes: (near, far),
    // });
    node
  }
  pub fn projection(&self) -> glam::Mat4 {
    let (near, far) = self.clipping_planes;
    match &self.projection {
      Projection::Perspective {
        field_of_view,
        aspect,
      } => glam::Mat4::perspective_rh(*field_of_view, *aspect, near, far),
      Projection::Orthographic {
        top,
        bottom,
        left,
        right,
      } => todo!(),
    }
  }
}
// impl Component for Camera {}
