use super::{
  buffer::{Buffer, BufferUsage},
  pipeline::{GraphicsPipeline, RenderPass},
  texture::Format,
  texture::{Sampler, Texture},
};
use crate::pipeline::GraphicsPipelineDesc;
use crate::{backend::Vulkan, pipeline::DescriptorWrite};
use bytemuck::Pod;
use std::sync::{Arc, RwLock};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AcquireSwapchainError {
  Timeout,
  Suboptimal,
  OutOfDate,
  Lost,
  OutOfMemory,
}

pub(crate) trait Backend {
  type Device;
  type Swapchain;
  type Buffer;
  type Texture;
  type Sampler;
  type Descriptor;
  type RenderPass;
  type Framebuffer;
  type GraphicsPipeline;
  type CommandList;

  // Device
  fn create_device(
    window: Option<Arc<winit::window::Window>>,
  ) -> (Self::Device, Option<(Self::Swapchain, Self::RenderPass)>);

  // Swapchain
  fn begin_frame(
    device: &Self::Device,
    swapchain: &Self::Swapchain,
  ) -> Result<Self::CommandList, AcquireSwapchainError>;

  // Buffer
  fn create_buffer(device: &Self::Device, usage: BufferUsage, data: &[u8]) -> Self::Buffer;
  fn create_buffer_uninit(device: &Self::Device, usage: BufferUsage, size: usize) -> Self::Buffer;
  fn map_buffer<T: bytemuck::Pod, F: Fn(&mut T)>(buffer: &Self::Buffer, f: F);

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
    render_pass: &Self::RenderPass,
  ) -> Self::GraphicsPipeline;

  // Command List
  fn create_command_list(device: &Self::Device) -> Self::CommandList;
  fn begin_final_pass(command_list: &mut Self::CommandList);
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
  fn bind_index_buffer(command_list: &mut Self::CommandList, buffer: &Self::Buffer);
  fn bind_descriptors(
    command_list: &mut Self::CommandList,
    set: u32,
    writes: &[DescriptorWriteAccess],
  );
  fn draw(
    command_list: &mut Self::CommandList,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
  );
  fn draw_indexed(
    command_list: &mut Self::CommandList,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    vertex_offset: i32,
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
  pub fn begin_final_pass(&mut self) -> &mut Self {
    B::begin_final_pass(&mut self.command_list);
    self
  }
  pub fn begin_render_pass(&mut self, render_pass: &RenderPass) -> &mut Self {
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
  pub fn end_render_pass(&mut self) -> &mut Self {
    B::end_render_pass(&mut self.command_list);
    self
  }
  pub fn bind_graphics_pipeline(&mut self, pipeline: &GraphicsPipeline) -> &mut Self {
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
  pub fn bind_vertex_buffer(&mut self, buffer: &Buffer) -> &mut Self {
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
  pub fn bind_index_buffer(&mut self, buffer: &Buffer) -> &mut Self {
    if let Some(buffer) = self
      .device
      .buffers
      .read()
      .unwrap()
      .get(buffer.handle.unwrap())
    {
      B::bind_index_buffer(&mut self.command_list, buffer);
    }
    self
  }
  pub fn bind_descriptors(&mut self, set: u32, writes: &[DescriptorWrite]) -> &mut Self {
    let buffers_read = self.device.buffers.read().unwrap();
    let samplers_read = self.device.samplers.read().unwrap();
    let textures_read = self.device.textures.read().unwrap();

    let access = writes
      .iter()
      .filter_map(|write| match write {
        DescriptorWrite::Invalid => None,
        DescriptorWrite::Buffer(binding, handle) => Some(DescriptorWriteAccess::Buffer(
          *binding,
          buffers_read.get(*handle).unwrap(),
        )),
        DescriptorWrite::Sampler(binding, handle) => Some(DescriptorWriteAccess::Sampler(
          *binding,
          samplers_read.get(*handle).unwrap(),
        )),
        DescriptorWrite::Texture(binding, handle) => Some(DescriptorWriteAccess::Texture(
          *binding,
          textures_read.get(*handle).unwrap(),
        )),
      })
      .collect::<Vec<_>>();
    B::bind_descriptors(&mut self.command_list, set, &access);
    self
  }
  pub fn draw(
    &mut self,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
  ) -> &mut Self {
    B::draw(
      &mut self.command_list,
      vertex_count,
      instance_count,
      first_vertex,
      first_instance,
    );
    self
  }
  pub fn draw_indexed(
    &mut self,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    vertex_offset: i32,
    first_instance: u32,
  ) -> &mut Self {
    B::draw_indexed(
      &mut self.command_list,
      index_count,
      instance_count,
      first_index,
      vertex_offset,
      first_instance,
    );
    self
  }
  pub fn copy_buffer_to_buffer(&mut self, src: &Buffer, dst: &Buffer) -> &mut Self {
    if let (Some(src), Some(dst)) = (
      self.device.buffers.read().unwrap().get(src.handle.unwrap()),
      self.device.buffers.read().unwrap().get(dst.handle.unwrap()),
    ) {
      B::copy_buffer_to_buffer(&mut self.command_list, src, dst);
    }
    self
  }
  pub fn copy_texture_to_buffer(&mut self, src: &Texture, dst: &Buffer) -> &mut Self {
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

pub(super) enum DescriptorWriteAccess<'a> {
  Buffer(u32, &'a <B as Backend>::Buffer),
  Texture(u32, &'a <B as Backend>::Texture),
  Sampler(u32, &'a <B as Backend>::Sampler),
}

pub struct RenderDevice {
  device: <B as Backend>::Device,
  swapchain: Option<(<B as Backend>::Swapchain, <B as Backend>::RenderPass)>,
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

  pub fn create_buffer(&self, usage: BufferUsage, data: &[u8]) -> Buffer {
    let buffer = B::create_buffer(&self.device, usage, data);
    let handle = Some(self.buffers.write().unwrap().insert(buffer));
    Buffer { handle }
  }

  pub fn create_buffer_raw(&self, usage: BufferUsage, size: usize) -> Buffer {
    let buffer = B::create_buffer_uninit(&self.device, usage, size);
    let handle = Some(self.buffers.write().unwrap().insert(buffer));
    Buffer { handle }
  }

  pub fn map_buffer<T: bytemuck::Pod, F: Fn(&mut T)>(&self, buffer: &Buffer, f: F) {
    if let Some(buffer) = self.buffers.read().unwrap().get(buffer.handle.unwrap()) {
      B::map_buffer(buffer, f);
    }
  }

  pub fn create_texture(&self, extent: (u32, u32, u32), format: Format) -> Texture {
    let texture = B::create_texture(&self.device, extent, format);
    let handle = Some(self.textures.write().unwrap().insert(texture));
    Texture { handle, extent }
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
    render_pass: Option<&RenderPass>,
  ) -> GraphicsPipeline {
    let render_passes_read = self.render_passes.read().unwrap();
    let render_pass = match render_pass {
      Some(render_pass) => render_passes_read.get(render_pass.handle).unwrap(),
      None => &self.swapchain.as_ref().expect("No swapchain").1,
    };
    let pipeline = B::create_graphics_pipeline(&self.device, &desc, render_pass);
    let handle = self.graphics_pipelines.write().unwrap().insert(pipeline);
    GraphicsPipeline { handle }
  }

  pub fn create_command_list(&self) -> CommandList {
    CommandList {
      device: &self,
      command_list: B::create_command_list(&self.device),
    }
  }

  pub fn execute_frame<F: Fn(&mut CommandList)>(&self, f: F) {
    let swapchain = self.swapchain.as_ref().expect("No swapchain");
    match B::begin_frame(&self.device, &swapchain.0) {
      Ok(command_list) => {
        let mut command_list = CommandList {
          device: self,
          command_list,
        };
        f(&mut command_list);
        command_list.submit();
      }
      Err(_) => todo!("recreate swapchain"),
    }
  }
}
impl Drop for RenderDevice {
  fn drop(&mut self) {
    unsafe {
      RENDER_DEVICE = None;
    }
  }
}
