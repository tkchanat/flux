use super::{
  buffer::{Buffer, BufferUsage},
  pipeline::{GraphicsPipeline, RenderPass},
  texture::Format,
  texture::{Sampler, Texture},
};
use crate::backend::Vulkan;
use crate::pipeline::GraphicsPipelineDesc;
use bytemuck::Pod;
use std::sync::{Arc, RwLock};
use vulkano::buffer::BufferContents;

pub trait Backend {
  type Device;
  type Swapchain;
  type Buffer;
  type Texture;
  type Sampler;
  type Descriptor;
  type RenderPass;
  type GraphicsPipeline;
  type CommandList;

  fn create_device(
    window: Option<Arc<winit::window::Window>>,
  ) -> (Self::Device, Option<Self::Swapchain>);

  // Buffer
  fn create_buffer<T: BufferContents + Pod>(
    device: &Self::Device,
    usage: BufferUsage,
    data: T,
  ) -> Self::Buffer;
  fn create_buffer_raw(device: &Self::Device, usage: BufferUsage, size: usize) -> Self::Buffer;
  fn map_buffer<T: BufferContents + Pod, F: Fn(&mut T)>(buffer: &Self::Buffer, f: F);

  // Texture
  fn create_texture(
    device: &Self::Device,
    extent: (u32, u32, u32),
    format: Format,
  ) -> Self::Texture;

  // Render Pass
  fn create_render_pass(
    device: &Self::Device,
    color_attachments: &[&Self::Texture],
    depth_attachment: Option<&Self::Texture>,
  ) -> Self::RenderPass;

  // Graphics Pipeline
  fn create_graphics_pipeline(
    device: &Self::Device,
    desc: &GraphicsPipelineDesc,
    framebuffer: &Self::RenderPass,
  ) -> Self::GraphicsPipeline;

  // Command List
  fn create_command_list(device: &Self::Device) -> Self::CommandList;
  fn begin_render_pass(
    command_list: &mut Self::CommandList,
    render_pass: &Self::RenderPass,
    color_attachments: &[&Self::Texture],
    depth_attachment: Option<&Self::Texture>,
  );
  fn end_render_pass(command_list: &mut Self::CommandList);
  fn bind_graphics_pipeline(
    command_list: &mut Self::CommandList,
    pipeline: &Self::GraphicsPipeline,
  );
  fn bind_vertex_buffer(command_list: &mut Self::CommandList, buffer: &Self::Buffer);
  fn draw(
    command_list: &mut Self::CommandList,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
  );
  fn copy_buffer_to_buffer(
    command_list: &mut Self::CommandList,
    src: &Self::Buffer,
    dst: &Self::Buffer,
  );
  fn copy_texture_to_buffer(
    command_list: &mut Self::CommandList,
    src: &Self::Texture,
    dst: &Self::Buffer,
  );
  fn submit(device: &Self::Device, command_list: Self::CommandList);
}

pub struct CommandList<'a> {
  device: &'a RenderDevice,
  command_list: <B as Backend>::CommandList,
}
impl<'a> CommandList<'a> {
  pub fn begin_render_pass(mut self, render_pass: &RenderPass) -> Self {
    let textures_read = self.device.textures.read().unwrap();
    let render_passes_read = self.device.render_passes.read().unwrap();
    let color_attachments = render_pass
      .bound_color_attachments
      .iter()
      .map(|color| textures_read.get(color.handle.unwrap()).unwrap())
      .collect::<Vec<_>>();
    let depth_attachment = render_pass
      .bound_depth_attachment
      .and_then(|depth| Some(textures_read.get(depth.handle.unwrap()).unwrap()));
    let render_pass = render_passes_read.get(render_pass.handle).unwrap();
    B::begin_render_pass(
      &mut self.command_list,
      render_pass,
      color_attachments.as_slice(),
      depth_attachment,
    );
    self
  }
  pub fn end_render_pass(mut self) -> Self {
    B::end_render_pass(&mut self.command_list);
    self
  }
  pub fn bind_graphics_pipeline(mut self, pipeline: &GraphicsPipeline) -> Self {
    if let Some(pipeline) = self
      .device
      .graphics_pipelines
      .read()
      .unwrap()
      .get(pipeline.handle)
    {
      B::bind_graphics_pipeline(&mut self.command_list, pipeline);
    }
    self
  }
  pub fn bind_vertex_buffer(mut self, buffer: &Buffer) -> Self {
    if let Some(buffer) = self
      .device
      .buffers
      .read()
      .unwrap()
      .get(buffer.handle.unwrap())
    {
      B::bind_vertex_buffer(&mut self.command_list, buffer);
    }
    self
  }
  pub fn draw(
    mut self,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
  ) -> Self {
    B::draw(
      &mut self.command_list,
      vertex_count,
      instance_count,
      first_vertex,
      first_instance,
    );
    self
  }
  pub fn copy_buffer_to_buffer(mut self, src: &Buffer, dst: &Buffer) -> Self {
    if let (Some(src), Some(dst)) = (
      self.device.buffers.read().unwrap().get(src.handle.unwrap()),
      self.device.buffers.read().unwrap().get(dst.handle.unwrap()),
    ) {
      B::copy_buffer_to_buffer(&mut self.command_list, src, dst);
    }
    self
  }
  pub fn copy_texture_to_buffer(mut self, src: &Texture, dst: &Buffer) -> Self {
    if let (Some(src), Some(dst)) = (
      self
        .device
        .textures
        .read()
        .unwrap()
        .get(src.handle.unwrap()),
      self.device.buffers.read().unwrap().get(dst.handle.unwrap()),
    ) {
      B::copy_texture_to_buffer(&mut self.command_list, src, dst);
    }
    self
  }
  pub fn submit(self) {
    B::submit(&self.device.device, self.command_list);
  }
}

pub(crate) static mut RENDER_DEVICE: Option<Arc<RenderDevice>> = None;

#[cfg(feature = "vulkan")]
type B = Vulkan;

pub struct RenderDevice {
  device: <B as Backend>::Device,
  swapchain: Option<<B as Backend>::Swapchain>,
  buffers: RwLock<slab::Slab<<B as Backend>::Buffer>>,
  textures: RwLock<slab::Slab<<B as Backend>::Texture>>,
  samplers: RwLock<slab::Slab<<B as Backend>::Sampler>>,
  descriptors: RwLock<slab::Slab<<B as Backend>::Descriptor>>,
  render_passes: RwLock<slab::Slab<<B as Backend>::RenderPass>>,
  graphics_pipelines: RwLock<slab::Slab<<B as Backend>::GraphicsPipeline>>,
}
impl RenderDevice {
  pub fn new(window: Option<Arc<winit::window::Window>>) -> Arc<Self> {
    let (device, swapchain) = B::create_device(window);
    let render_device = Arc::new(Self {
      device,
      swapchain,
      buffers: RwLock::new(slab::Slab::new()),
      textures: RwLock::new(slab::Slab::new()),
      samplers: RwLock::new(slab::Slab::new()),
      descriptors: RwLock::new(slab::Slab::new()),
      render_passes: RwLock::new(slab::Slab::new()),
      graphics_pipelines: RwLock::new(slab::Slab::new()),
    });
    unsafe {
      RENDER_DEVICE = Some(render_device.clone());
    }
    render_device
  }

  pub fn create_buffer<T: BufferContents + Pod>(&self, usage: BufferUsage, data: T) -> Buffer {
    let buffer = B::create_buffer(&self.device, usage, data);
    let handle = Some(self.buffers.write().unwrap().insert(buffer));
    Buffer { handle }
  }

  pub fn create_buffer_raw(&self, usage: BufferUsage, size: usize) -> Buffer {
    let buffer = B::create_buffer_raw(&self.device, usage, size);
    let handle = Some(self.buffers.write().unwrap().insert(buffer));
    Buffer { handle }
  }

  pub fn map_buffer<T: BufferContents + Pod, F: Fn(&mut T)>(&self, buffer: &Buffer, f: F) {
    if let Some(buffer) = self.buffers.read().unwrap().get(buffer.handle.unwrap()) {
      B::map_buffer(buffer, f);
    }
  }

  pub fn create_texture(&self, extent: (u32, u32, u32), format: Format) -> Texture {
    let texture = B::create_texture(&self.device, extent, format);
    let handle = Some(self.textures.write().unwrap().insert(texture));
    Texture { handle }
  }

  pub fn create_render_pass(
    &self,
    color_attachments: &[Texture],
    depth_attachment: Option<Texture>,
  ) -> RenderPass {
    let textures_read = self.textures.read().unwrap();
    let render_pass = {
      let color_attachments = color_attachments
        .iter()
        .map(|color| textures_read.get(color.handle.unwrap()).unwrap())
        .collect::<Vec<_>>();
      let depth_attachment =
        depth_attachment.and_then(|depth| Some(textures_read.get(depth.handle.unwrap()).unwrap()));
      B::create_render_pass(&self.device, color_attachments.as_slice(), depth_attachment)
    };
    let handle = self.render_passes.write().unwrap().insert(render_pass);
    RenderPass {
      handle,
      bound_color_attachments: color_attachments.to_vec(),
      bound_depth_attachment: depth_attachment,
    }
  }

  pub fn create_graphics_pipeline(
    &self,
    desc: &GraphicsPipelineDesc,
    render_pass: &RenderPass,
  ) -> GraphicsPipeline {
    let render_passes_read = self.render_passes.read().unwrap();
    let render_pass = render_passes_read.get(render_pass.handle).unwrap();
    let pipeline = B::create_graphics_pipeline(&self.device, desc, render_pass);
    let handle = self.graphics_pipelines.write().unwrap().insert(pipeline);
    GraphicsPipeline { handle }
  }

  pub fn create_command_list<'a>(&'a self) -> CommandList {
    CommandList {
      device: self,
      command_list: B::create_command_list(&self.device),
    }
  }
}
// impl Drop for RenderDevice {
//   fn drop(&mut self) {
//     unsafe {
//       RENDER_DEVICE = None;
//     }
//   }
// }