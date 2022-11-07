use super::{buffer::Buffer, texture::Texture};
use std::ops::Range;

#[derive(Clone, Debug, Default)]
pub struct Viewport {
  pub offset: (f32, f32),
  pub dimensions: (f32, f32),
  pub depth_range: Range<f32>,
}

#[derive(Clone, Debug)]
pub enum DescriptorWrite {
  Invalid,
  Buffer(u32, usize),
  Texture(u32, usize),
  Sampler(u32, usize),
}
impl DescriptorWrite {
  pub fn buffer(binding: u32, buffer: &Buffer) -> Self {
    if let Some(handle) = buffer.handle {
      Self::Buffer(binding, handle)
    } else {
      Self::Invalid
    }
  }

  pub fn texture(binding: u32, texture: &Texture) -> Self {
    if let Some(handle) = texture.handle {
      Self::Texture(binding, handle)
    } else {
      Self::Invalid
    }
  }

  // pub fn sampler(binding: u32, sampler: &Sampler) -> Self {
  //   if let Some(handle) = sampler.handle {
  //     Self::Sampler(binding, handle)
  //   } else {
  //     Self::Invalid
  //   }
  // }
}

#[derive(Clone, Debug)]
pub struct GraphicsPipelineDesc {
  pub vs_spv: &'static [u8],
  pub fs_spv: &'static [u8],
  pub viewport: Viewport,
  pub depth_test: bool,
}
impl GraphicsPipelineDesc {
  pub fn new() -> Self {
    GraphicsPipelineDesc {
      vs_spv: b"",
      fs_spv: b"",
      viewport: Viewport::default(),
      depth_test: false,
    }
  }
  #[inline]
  pub fn vertex_shader(mut self, spv: &'static [u8]) -> Self {
    self.vs_spv = spv;
    self
  }
  #[inline]
  pub fn fragment_shader(mut self, spv: &'static [u8]) -> Self {
    self.fs_spv = spv;
    self
  }
  #[inline]
  pub fn viewport(
    mut self,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    depth_range: Range<f32>,
  ) -> Self {
    self.viewport = Viewport {
      offset: (x, y),
      dimensions: (width, height),
      depth_range,
    };
    self
  }
  #[inline]
  pub fn depth_test(mut self, enabled: bool) -> Self {
    self.depth_test = enabled;
    self
  }
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
