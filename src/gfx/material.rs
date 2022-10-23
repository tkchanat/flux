use specs::{Component, DenseVecStorage};
use specs_derive::Component;
use super::BindGroup;

// pub enum RenderQueue {
//   Background,
//   Opaque,
//   Translucent,
//   LateOpaque,
//   Transparent,
//   Overlay,
// }

// #[derive(Component)]
// pub struct Material {
//   queue: RenderQueue,
// }
// impl Material {
//   pub fn new(queue: RenderQueue) -> Self {
//     Self { queue }
//   }
// }

#[derive(Component)]
pub struct MaterialOverlay {
  pub(super) bind_group: BindGroup,
}
impl MaterialOverlay {
  pub fn new(bind_group: BindGroup) -> Self {
    Self { bind_group }
  }
}
