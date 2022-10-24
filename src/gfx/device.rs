use std::{cell::RefCell, collections::HashMap, io::Read, ops::Range, sync::Arc};

use super::{
  BindGroup, BindGroupEntry, BindGroupLayout, Buffer, Format, GraphicsPipeline, IndexBuffer,
  RenderPass, Sampler, Texture, VertexBuffer,
};
use crate::core::app;
use bytemuck::{
  bytes_of, cast_slice, try_cast_slice, try_cast_slice_mut, try_from_bytes, try_from_bytes_mut,
  Pod, PodCastError,
};
use wgpu::util::DeviceExt;

struct WgpuTexture {
  handle: wgpu::Texture,
  view: wgpu::TextureView,
  size: wgpu::Extent3d,
  format: wgpu::TextureFormat,
}

pub struct RenderDeviceOld {
  pub(super) device: wgpu::Device,
  pub(super) surface: wgpu::Surface,
  pub(super) queue: wgpu::Queue,
  pub(super) config: wgpu::SurfaceConfiguration,
  pub(super) extent: winit::dpi::PhysicalSize<u32>,
  textures: slab::Slab<WgpuTexture>,
  samplers: slab::Slab<wgpu::Sampler>,
  buffers: slab::Slab<wgpu::Buffer>,
  bind_groups: slab::Slab<wgpu::BindGroup>,
  bind_group_layouts: slab::Slab<wgpu::BindGroupLayout>,
  render_pipelines: slab::Slab<wgpu::RenderPipeline>,
}

impl RenderDeviceOld {
  pub fn new(window: &winit::window::Window) -> Self {
    let extent = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::default(),
      compatible_surface: Some(&surface),
      force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::PUSH_CONSTANTS | wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
        // WebGL doesn't support all of wgpu's features, so if
        // we're building for the web we'll have to disable some.
        limits: {
          let mut limit = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
          } else {
            wgpu::Limits::default()
          };
          limit.max_push_constant_size = 128;
          limit
        },
        label: None,
      },
      None, // Trace path
    ))
    .unwrap();

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface.get_supported_formats(&adapter)[0],
      width: extent.width,
      height: extent.height,
      present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    Self {
      surface,
      device,
      queue,
      config,
      extent,
      textures: slab::Slab::new(),
      samplers: slab::Slab::new(),
      buffers: slab::Slab::new(),
      bind_groups: slab::Slab::new(),
      bind_group_layouts: slab::Slab::new(),
      render_pipelines: slab::Slab::new(),
    }
  }

  pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
    self.extent = new_size.clone();
    self.config.width = new_size.width;
    self.config.height = new_size.height;
    self.surface.configure(&self.device, &self.config);
  }

  pub fn create_sampler(&mut self, desc: &wgpu::SamplerDescriptor) -> Sampler {
    let sampler = self.device.create_sampler(desc);
    let handle = Some(self.samplers.insert(sampler));
    Sampler { handle }
  }

  pub fn create_texture(&mut self, desc: &wgpu::TextureDescriptor) -> Texture {
    let texture = self.device.create_texture(desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
      label: None,
      format: None,
      dimension: None,
      aspect: wgpu::TextureAspect::All, // TODO
      base_mip_level: 0,
      mip_level_count: None,
      base_array_layer: 0,
      array_layer_count: None,
    });
    let handle = Some(self.textures.insert(WgpuTexture {
      handle: texture,
      view,
      size: desc.size,
      format: desc.format,
    }));
    Texture { handle }
  }

  pub fn create_buffer(&mut self, desc: &wgpu::BufferDescriptor) -> Buffer {
    let buffer = self.device.create_buffer(desc);
    let handle = self.buffers.insert(buffer);
    Buffer { handle }
  }

  pub fn create_buffer_init(&mut self, desc: &wgpu::util::BufferInitDescriptor) -> Buffer {
    let buffer = self.device.create_buffer_init(desc);
    let handle = self.buffers.insert(buffer);
    Buffer { handle }
  }

  pub fn create_bind_group_layout(
    &mut self,
    desc: &wgpu::BindGroupLayoutDescriptor,
  ) -> BindGroupLayout {
    let bind_group_layout = self.device.create_bind_group_layout(desc);
    let handle = Some(self.bind_group_layouts.insert(bind_group_layout));
    BindGroupLayout { handle }
  }

  pub fn create_bind_group(
    &mut self,
    bind_group_layout: &BindGroupLayout,
    entries: &[BindGroupEntry],
  ) -> BindGroup {
    if bind_group_layout.handle.is_none() {
      return BindGroup { handle: None };
    }
    let entries = entries
      .iter()
      .map(|entry| match entry {
        BindGroupEntry::Buffer(binding, buffer) => wgpu::BindGroupEntry {
          binding: *binding,
          resource: self.buffers.get(buffer.handle).unwrap().as_entire_binding(),
        },
        BindGroupEntry::Texture(binding, texture) => wgpu::BindGroupEntry {
          binding: *binding,
          resource: wgpu::BindingResource::TextureView(
            &self.textures.get(texture.handle.unwrap()).unwrap().view,
          ),
        },
        BindGroupEntry::Sampler(binding, sampler) => wgpu::BindGroupEntry {
          binding: *binding,
          resource: wgpu::BindingResource::Sampler(
            &self.samplers.get(sampler.handle.unwrap()).unwrap(),
          ),
        },
      })
      .collect::<Vec<wgpu::BindGroupEntry>>();
    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: self
        .bind_group_layouts
        .get(bind_group_layout.handle.unwrap())
        .unwrap(),
      entries: entries.as_slice(),
    });
    let handle = Some(self.bind_groups.insert(bind_group));
    BindGroup { handle }
  }

  pub fn create_pipeline_layout(
    &mut self,
    desc: &wgpu::PipelineLayoutDescriptor,
  ) -> Option<wgpu::PipelineLayout> {
    Some(self.device.create_pipeline_layout(desc))
  }

  pub fn create_render_pipeline(
    &mut self,
    vertex: &str,
    fragment: Option<&str>,
  ) -> GraphicsPipeline {
    let mut stages = Vec::from_iter([(wgpu::ShaderStages::VERTEX, vertex)]);
    if let Some(fragment) = fragment {
      stages.push((wgpu::ShaderStages::FRAGMENT, fragment));
    }

    let mut vertex_state = None;
    let mut fragment_state = None;
    let mut input_attributes = Vec::new();
    let mut output_targets = Vec::new();
    let mut bind_groups = HashMap::<u32, HashMap<String, wgpu::BindGroupLayoutEntry>>::new();
    let mut push_constants_map = HashMap::<String, (wgpu::ShaderStages, Range<u32>)>::new();
    for (stage, path) in stages {
      let code = {
        let mut f = std::fs::File::open(path).expect("File does not exist");
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)
          .expect("Unable to read file to buffer");
        buffer
      };
      let shader_module = unsafe {
        self
          .device
          .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
            label: None,
            source: wgpu::util::make_spirv_raw(code.as_slice()),
          })
      };
      let reflect_module = spirv_reflect::ShaderModule::load_u8_data(code.as_slice()).unwrap();
      let entry_point = reflect_module.get_entry_point_name();

      for descriptor in reflect_module.enumerate_descriptor_sets(None).unwrap() {
        let set = descriptor.set;
        for binding in descriptor.bindings {
          let binding_entry = wgpu::BindGroupLayoutEntry {
            binding: binding.binding,
            visibility: stage,
            ty: to_binding_type(binding.descriptor_type),
            count: None, // TODO
          };
          match bind_groups.get_mut(&set) {
            Some(entry) => match entry.get_mut(&binding.name) {
              Some(entry) => entry.visibility |= stage,
              None => {
                entry.insert(binding.name, binding_entry);
              }
            },
            None => {
              bind_groups.insert(set, HashMap::from_iter([(binding.name, binding_entry)]));
            }
          }
        }
      }
      for pc in reflect_module.enumerate_push_constant_blocks(None).unwrap() {
        match push_constants_map.get_mut(&pc.name) {
          Some(entry) => entry.0 |= stage,
          None => {
            push_constants_map.insert(pc.name, (stage, 0..pc.size));
          }
        }
      }

      if stage == wgpu::ShaderStages::VERTEX {
        let mut offset = 0;
        for input in reflect_module.enumerate_input_variables(None) {
          let mut attr = Vec::new();
          for variable in input {
            let size = format_to_size(variable.format);
            attr.push(wgpu::VertexAttribute {
              format: to_vertex_type(variable.format),
              offset,
              shader_location: variable.location,
            });
            // println!("{}.size = {}", variable.name, size);
            offset += size;
          }
          input_attributes.push(attr);
        }
        vertex_state = Some((shader_module, entry_point, offset));
      } else if stage == wgpu::ShaderStages::FRAGMENT {
        for output in reflect_module.enumerate_output_variables(None) {
          for _variable in output {
            output_targets.push(Some(wgpu::ColorTargetState {
              format: self.config.format, // FIXME
              blend: Some(wgpu::BlendState::REPLACE),
              write_mask: wgpu::ColorWrites::ALL,
            }))
          }
        }
        fragment_state = Some((shader_module, entry_point));
      }
    }
    let bind_group_layouts = bind_groups
      .into_iter()
      .map(|group| {
        let bindings = group.1.into_iter().map(|x| x.1).collect::<Vec<_>>();
        self
          .device
          .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: bindings.as_slice(),
          })
      })
      .collect::<Vec<_>>();
    let push_constant_ranges = push_constants_map
      .iter()
      .map(|pc| wgpu::PushConstantRange {
        stages: pc.1 .0,
        range: pc.1 .1.clone(),
      })
      .collect::<Vec<_>>();
    let pipeline_layout = self
      .device
      .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: bind_group_layouts
          .iter()
          .map(|x| x)
          .collect::<Vec<_>>()
          .as_slice(),
        push_constant_ranges: push_constant_ranges.as_slice(),
      });
    let vertex = {
      let (module, entry_point, stride) = vertex_state.as_ref().unwrap();
      wgpu::VertexState {
        module,
        entry_point,
        buffers: &input_attributes
          .iter()
          .map(|attr| wgpu::VertexBufferLayout {
            array_stride: *stride,
            step_mode: wgpu::VertexStepMode::Vertex, // TODO
            attributes: &attr,
          })
          .collect::<Vec<_>>(),
      }
    };
    let fragment = fragment_state.as_ref().and_then(|(module, entry_point)| {
      Some(wgpu::FragmentState {
        module,
        entry_point,
        targets: &output_targets,
      })
    });
    let desc = wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex,
      fragment,
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      depth_stencil: Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
    };

    let pipeline = self.device.create_render_pipeline(&desc);
    let handle = self.render_pipelines.insert(pipeline);
    GraphicsPipeline { handle }
  }

  pub fn get_buffer_binding(&self, buffer: &Buffer) -> wgpu::BindingResource {
    self.buffers.get(buffer.handle).unwrap().as_entire_binding()
  }

  pub fn begin_render_pass<F: FnMut(&mut RenderPassOld)>(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    desc: &wgpu::RenderPassDescriptor,
    mut f: F,
  ) {
    let mut render_pass = RenderPassOld {
      render_device: &self,
      handle: encoder.begin_render_pass(desc),
    };
    f(&mut render_pass);
  }

  pub fn update_buffer(&self, buffer: &Buffer, data: &[u8]) {
    if let Some(buffer) = self.buffers.get(buffer.handle) {
      let offset = 0;
      self.queue.write_buffer(buffer, offset, data);
    }
  }

  pub fn update_texture(&self, texture: &Texture, data: &[u8]) {
    if texture.handle.is_none() {
      return;
    }
    if let Some(texture) = self.textures.get(texture.handle.unwrap()) {
      let x_stride = texture.format.describe().block_size as u32;
      let y_stride = texture.size.width * x_stride;
      self.queue.write_texture(
        wgpu::ImageCopyTexture {
          texture: &texture.handle,
          mip_level: 0,
          origin: wgpu::Origin3d::ZERO,
          aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: std::num::NonZeroU32::new(y_stride),
          rows_per_image: std::num::NonZeroU32::new(texture.size.height),
        },
        texture.size,
      );
    }
  }
}

pub struct RenderPassOld<'a> {
  render_device: &'a RenderDeviceOld,
  handle: wgpu::RenderPass<'a>,
}
impl<'a> RenderPassOld<'a> {
  pub fn set_pipeline(&mut self, pipeline: &GraphicsPipeline) {
    if let Some(pipeline) = self.render_device.render_pipelines.get(pipeline.handle) {
      self.handle.set_pipeline(pipeline);
    }
  }

  pub fn set_bind_group(&mut self, index: u32, bind_group: &BindGroup, offsets: &[u32]) {
    if let Some(bind_group) = bind_group
      .handle
      .and_then(|handle| self.render_device.bind_groups.get(handle))
    {
      self.handle.set_bind_group(index, bind_group, offsets);
    }
  }

  pub fn set_vertex_buffer(&mut self, slot: u32, vertex_buffer: &VertexBuffer) {
    if let Some(buffer) = self.render_device.buffers.get(vertex_buffer.buffer.handle) {
      self.handle.set_vertex_buffer(slot, buffer.slice(..));
    }
  }

  pub fn set_index_buffer(&mut self, index_buffer: &IndexBuffer) {
    if let Some(buffer) = self.render_device.buffers.get(index_buffer.buffer.handle) {
      self
        .handle
        .set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint32);
    }
  }

  pub fn set_push_constants(&mut self, stages: wgpu::ShaderStages, offset: u32, data: &[u8]) {
    self.handle.set_push_constants(stages, offset, data);
  }

  pub fn draw(&mut self, vertices: Range<u32>, instance: Range<u32>) {
    self.handle.draw(vertices, instance);
  }

  pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
    self.handle.draw_indexed(indices, base_vertex, instances);
  }
}

fn to_binding_type(
  ty: spirv_reflect::types::descriptor::ReflectDescriptorType,
) -> wgpu::BindingType {
  match ty {
    spirv_reflect::types::ReflectDescriptorType::Undefined => panic!(),
    spirv_reflect::types::ReflectDescriptorType::Sampler => {
      wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    }
    spirv_reflect::types::ReflectDescriptorType::CombinedImageSampler => todo!(),
    spirv_reflect::types::ReflectDescriptorType::SampledImage => wgpu::BindingType::Texture {
      sample_type: wgpu::TextureSampleType::default(),
      view_dimension: wgpu::TextureViewDimension::D2,
      multisampled: false,
    },
    spirv_reflect::types::ReflectDescriptorType::StorageImage => todo!(),
    spirv_reflect::types::ReflectDescriptorType::UniformTexelBuffer => todo!(),
    spirv_reflect::types::ReflectDescriptorType::StorageTexelBuffer => todo!(),
    spirv_reflect::types::ReflectDescriptorType::UniformBuffer => wgpu::BindingType::Buffer {
      ty: wgpu::BufferBindingType::Uniform,
      has_dynamic_offset: false,
      min_binding_size: None,
    },
    spirv_reflect::types::ReflectDescriptorType::StorageBuffer => todo!(),
    spirv_reflect::types::ReflectDescriptorType::UniformBufferDynamic => {
      wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: true,
        min_binding_size: None,
      }
    }
    spirv_reflect::types::ReflectDescriptorType::StorageBufferDynamic => todo!(),
    spirv_reflect::types::ReflectDescriptorType::InputAttachment => todo!(),
    spirv_reflect::types::ReflectDescriptorType::AccelerationStructureNV => todo!(),
  }
}

fn to_vertex_type(ty: spirv_reflect::types::ReflectFormat) -> wgpu::VertexFormat {
  match ty {
    spirv_reflect::types::ReflectFormat::Undefined => panic!(),
    spirv_reflect::types::ReflectFormat::R32_UINT => wgpu::VertexFormat::Uint32,
    spirv_reflect::types::ReflectFormat::R32_SINT => wgpu::VertexFormat::Sint32,
    spirv_reflect::types::ReflectFormat::R32_SFLOAT => wgpu::VertexFormat::Float32,
    spirv_reflect::types::ReflectFormat::R32G32_UINT => wgpu::VertexFormat::Uint32x2,
    spirv_reflect::types::ReflectFormat::R32G32_SINT => wgpu::VertexFormat::Sint32x2,
    spirv_reflect::types::ReflectFormat::R32G32_SFLOAT => wgpu::VertexFormat::Float32x2,
    spirv_reflect::types::ReflectFormat::R32G32B32_UINT => wgpu::VertexFormat::Uint32x3,
    spirv_reflect::types::ReflectFormat::R32G32B32_SINT => wgpu::VertexFormat::Sint32x3,
    spirv_reflect::types::ReflectFormat::R32G32B32_SFLOAT => wgpu::VertexFormat::Float32x3,
    spirv_reflect::types::ReflectFormat::R32G32B32A32_UINT => wgpu::VertexFormat::Uint32x4,
    spirv_reflect::types::ReflectFormat::R32G32B32A32_SINT => wgpu::VertexFormat::Sint32x4,
    spirv_reflect::types::ReflectFormat::R32G32B32A32_SFLOAT => wgpu::VertexFormat::Float32x4,
  }
}
fn format_to_size(ty: spirv_reflect::types::ReflectFormat) -> u64 {
  match ty {
    spirv_reflect::types::ReflectFormat::Undefined => panic!(),
    spirv_reflect::types::ReflectFormat::R32_UINT => 4,
    spirv_reflect::types::ReflectFormat::R32_SINT => 4,
    spirv_reflect::types::ReflectFormat::R32_SFLOAT => 4,
    spirv_reflect::types::ReflectFormat::R32G32_UINT => 8,
    spirv_reflect::types::ReflectFormat::R32G32_SINT => 8,
    spirv_reflect::types::ReflectFormat::R32G32_SFLOAT => 8,
    spirv_reflect::types::ReflectFormat::R32G32B32_UINT => 12,
    spirv_reflect::types::ReflectFormat::R32G32B32_SINT => 12,
    spirv_reflect::types::ReflectFormat::R32G32B32_SFLOAT => 12,
    spirv_reflect::types::ReflectFormat::R32G32B32A32_UINT => 16,
    spirv_reflect::types::ReflectFormat::R32G32B32A32_SINT => 16,
    spirv_reflect::types::ReflectFormat::R32G32B32A32_SFLOAT => 16,
  }
}

pub unsafe trait BufferContents: Send + Sync + 'static {
  /// Converts an immutable reference to `Self` to an immutable byte slice.
  fn as_bytes(&self) -> &[u8];

  /// Converts an immutable byte slice into an immutable reference to `Self`.
  fn from_bytes(bytes: &[u8]) -> Result<&Self, PodCastError>;

  /// Converts a mutable byte slice into a mutable reference to `Self`.
  fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut Self, PodCastError>;

  /// Returns the size of an element of the type.
  fn size_of_element() -> u64;
}

unsafe impl<T> BufferContents for T
where
  T: Pod + Send + Sync,
{
  #[inline]
  fn as_bytes(&self) -> &[u8] {
    bytes_of(self)
  }

  #[inline]
  fn from_bytes(bytes: &[u8]) -> Result<&T, PodCastError> {
    try_from_bytes(bytes)
  }

  #[inline]
  fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut T, PodCastError> {
    try_from_bytes_mut(bytes)
  }

  #[inline]
  fn size_of_element() -> u64 {
    1
  }
}

unsafe impl<T> BufferContents for [T]
where
  T: Pod + Send + Sync,
{
  #[inline]
  fn as_bytes(&self) -> &[u8] {
    cast_slice(self)
  }

  #[inline]
  fn from_bytes(bytes: &[u8]) -> Result<&[T], PodCastError> {
    try_cast_slice(bytes)
  }

  #[inline]
  fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut [T], PodCastError> {
    try_cast_slice_mut(bytes)
  }

  #[inline]
  fn size_of_element() -> u64 {
    std::mem::size_of::<T>() as u64
  }
}

bitflags::bitflags! {
  pub struct BufferUsage: u32 {
    const TRANSFER_SRC = 0b1;
    const TRANSFER_DST = 0b10;
    const UNIFORM_TEXEL_BUFFER = 0b100;
    const STORAGE_TEXEL_BUFFER = 0b1000;
    const UNIFORM_BUFFER = 0b1_0000;
    const STORAGE_BUFFER = 0b10_0000;
    const INDEX_BUFFER = 0b100_0000;
    const VERTEX_BUFFER = 0b1000_0000;
    const INDIRECT_BUFFER = 0b1_0000_0000;
  }
}

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
  fn create_buffer_with_init<T: BufferContents + Pod>(
    device: &Self::Device,
    usage: BufferUsage,
    data: T,
  ) -> Self::Buffer;
  fn update_buffer<T: BufferContents + Pod, F: FnMut(&mut T)>(
    device: &Self::Device,
    buffer: &Self::Buffer,
    f: F,
  );
  fn copy_buffer_to_buffer(device: &Self::Device, src: &Self::Buffer, dst: &Self::Buffer);

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
    framebuffer: &Self::RenderPass,
  ) -> Self::GraphicsPipeline;

  // TODO: to be removed
  fn save_texture_to_disk(device: &Self::Device, texture: &Self::Texture);

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
  fn submit(device: &Self::Device, command_list: Self::CommandList);
}

pub struct CommandList<'a, B: Backend> {
  device: &'a RenderDevice<B>,
  command_list: B::CommandList,
}
impl<'a, B: Backend> CommandList<'a, B> {
  pub fn begin_render_pass(mut self, render_pass: &RenderPass) -> Self {
    let color_attachments = render_pass
      .bound_color_attachments
      .iter()
      .map(|color| self.device.textures.get(color.handle.unwrap()).unwrap())
      .collect::<Vec<_>>();
    let depth_attachment = render_pass
      .bound_depth_attachment
      .and_then(|depth| Some(self.device.textures.get(depth.handle.unwrap()).unwrap()));
    let render_pass = self.device.render_passes.get(render_pass.handle).unwrap();
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
    if let Some(pipeline) = self.device.graphics_pipelines.get(pipeline.handle) {
      B::bind_graphics_pipeline(&mut self.command_list, pipeline);
    }
    self
  }
  pub fn bind_vertex_buffer(mut self, buffer: &Buffer) -> Self {
    if let Some(buffer) = self.device.buffers.get(buffer.handle) {
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
  pub fn submit(self) {
    B::submit(&self.device.device, self.command_list);
  }
}

pub struct RenderDevice<B: Backend> {
  device: B::Device,
  swapchain: Option<B::Swapchain>,
  buffers: slab::Slab<B::Buffer>,
  textures: slab::Slab<B::Texture>,
  samplers: slab::Slab<B::Sampler>,
  descriptors: slab::Slab<B::Descriptor>,
  render_passes: slab::Slab<B::RenderPass>,
  graphics_pipelines: slab::Slab<B::GraphicsPipeline>,
}
impl<B: Backend> RenderDevice<B> {
  pub fn new(window: Option<Arc<winit::window::Window>>) -> Self {
    let (device, swapchain) = B::create_device(window);
    Self {
      device,
      swapchain,
      buffers: slab::Slab::new(),
      textures: slab::Slab::new(),
      samplers: slab::Slab::new(),
      descriptors: slab::Slab::new(),
      render_passes: slab::Slab::new(),
      graphics_pipelines: slab::Slab::new(),
    }
  }

  pub fn create_buffer_with_init<T: BufferContents + Pod>(
    &mut self,
    usage: BufferUsage,
    data: T,
  ) -> Buffer {
    let buffer = B::create_buffer_with_init(&self.device, usage, data);
    let handle = self.buffers.insert(buffer);
    Buffer { handle }
  }

  pub fn update_buffer<T: BufferContents + Pod, F: FnMut(&mut T)>(&self, buffer: &Buffer, f: F) {
    if let Some(buffer) = self.buffers.get(buffer.handle) {
      B::update_buffer(&self.device, buffer, f);
    }
  }

  pub fn copy_buffer_to_buffer(&self, src: &Buffer, dst: &Buffer) {
    if let (Some(src), Some(dst)) = (self.buffers.get(src.handle), self.buffers.get(dst.handle)) {
      B::copy_buffer_to_buffer(&self.device, src, dst);
    }
  }

  pub fn create_texture(&mut self, extent: (u32, u32, u32), format: Format) -> Texture {
    let texture = B::create_texture(&self.device, extent, format);
    let handle = Some(self.textures.insert(texture));
    Texture { handle }
  }

  pub fn create_render_pass(
    &mut self,
    color_attachments: &[Texture],
    depth_attachment: Option<Texture>,
  ) -> RenderPass {
    let render_pass = {
      let color_attachments = color_attachments
        .iter()
        .map(|color| self.textures.get(color.handle.unwrap()).unwrap())
        .collect::<Vec<_>>();
      let depth_attachment =
        depth_attachment.and_then(|depth| Some(self.textures.get(depth.handle.unwrap()).unwrap()));
      B::create_render_pass(&self.device, color_attachments.as_slice(), depth_attachment)
    };
    let handle = self.render_passes.insert(render_pass);
    RenderPass {
      handle,
      bound_color_attachments: color_attachments.to_vec(),
      bound_depth_attachment: depth_attachment,
    }
  }

  pub fn create_graphics_pipeline(&mut self, framebuffer: &RenderPass) -> GraphicsPipeline {
    let framebuffer = self.render_passes.get(framebuffer.handle).unwrap();
    let pipeline = B::create_graphics_pipeline(&self.device, framebuffer);
    let handle = self.graphics_pipelines.insert(pipeline);
    GraphicsPipeline { handle }
  }

  pub fn create_command_list<'a>(&'a self) -> CommandList<B> {
    CommandList {
      device: self,
      command_list: B::create_command_list(&self.device),
    }
  }

  pub fn save_texture_to_disk(&self, texture: &Texture) {
    if let Some(texture) = self.textures.get(texture.handle.unwrap()) {
      B::save_texture_to_disk(&self.device, texture);
    }
  }
}
