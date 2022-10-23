use super::{Buffer, Texture, Sampler};

#[derive(Clone, Debug)]
pub struct BindGroup {
  pub(super) handle: Option<usize>,
}
impl BindGroup {
  pub fn new(bind_group_layout: &BindGroupLayout, entries: &[BindGroupEntry]) -> Self {
    unsafe {
      if let Some(app) = crate::core::app::APP_INSTANCE.as_mut() {
        app.render_device.create_bind_group(bind_group_layout, entries)
      } else {
        Self { handle: None }
      }
    }
  }
}

#[derive(Clone, Debug)]
pub struct BindGroupLayout {
  pub(super) handle: Option<usize>,
}
impl BindGroupLayout {
  pub fn new(desc: &wgpu::BindGroupLayoutDescriptor) -> Self {
    unsafe {
      if let Some(app) = crate::core::app::APP_INSTANCE.as_mut() {
        app.render_device.create_bind_group_layout(desc)
      }
      else {
        Self { handle: None }
      }
    }
  }
}

pub enum BindGroupEntry<'a> {
  Buffer(u32, &'a Buffer),
  Texture(u32, &'a Texture),
  Sampler(u32, &'a Sampler)
}
