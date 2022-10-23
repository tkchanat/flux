#[derive(Clone, Debug)]
pub struct GraphicsPipeline {
  pub(super) handle: usize,
}

#[derive(Clone, Debug)]
pub struct ComputePipeline {
  pub(super) handle: usize,
}

#[derive(Clone, Debug)]
pub struct RenderPass {
  pub(super) handle: usize,
}