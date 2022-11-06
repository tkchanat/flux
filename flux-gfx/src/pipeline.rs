use super::texture::Texture;
use std::ops::Range;

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Viewport {
  pub offset: (f32, f32),
  pub dimensions: (f32, f32),
  pub depth_range: Range<f32>,
}
impl Viewport {
  pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
    Self {
      offset: (x, y),
      dimensions: (width, height),
      depth_range: 0.0..1.0
    }
  }
}

pub struct GraphicsPipelineDesc<'a> {
  pub vs_spv: &'a [u8],
  pub fs_spv: &'a [u8],
  pub viewport: Viewport,
}

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
