use super::{RenderDevice, Texture};

#[derive(Clone, Debug)]
pub struct GraphicsPipeline {
  pub(super) handle: usize,
}

#[derive(Clone, Debug)]
pub struct ComputePipeline {
  pub(super) handle: usize,
}

/*********************/
/**** Render Pass ****/
/*********************/

#[derive(Clone, Debug)]
pub struct RenderPass {
  pub(super) handle: usize,
  pub(super) bound_color_attachments: Vec<Texture>,
  pub(super) bound_depth_attachment: Option<Texture>,
}