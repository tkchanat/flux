use crate::core::app;
use wgpu::util::DeviceExt;

pub struct RenderDevice {
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  extent: winit::dpi::PhysicalSize<u32>,
}

impl RenderDevice {
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
        features: wgpu::Features::PUSH_CONSTANTS,
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
    }
  }

  pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
    self.extent = new_size.clone();
    self.config.width = new_size.width;
    self.config.height = new_size.height;
    self.surface.configure(&self.device, &self.config);
  }

  pub fn surface_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
    self.surface.get_current_texture()
  }

  pub fn surface_format(&self) -> wgpu::TextureFormat {
    self.config.format
  }

  pub fn create_buffer(
    &self,
    label: Option<&str>,
    contents: &[u8],
    usage: wgpu::BufferUsages,
  ) -> wgpu::Buffer {
    self
      .device
      .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label,
        contents,
        usage,
      })
  }

  pub fn create_texture(
    &self,
    label: Option<&str>,
    size: wgpu::Extent3d,
    mip_level_count: u32,
    sample_count: u32,
    dimension: wgpu::TextureDimension,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
  ) -> wgpu::Texture {
    self.device.create_texture(&wgpu::TextureDescriptor {
      size,
      mip_level_count,
      sample_count,
      dimension,
      format,
      usage,
      label,
    })
  }

  pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
    self.device.create_sampler(desc)
  }

  pub fn create_bind_group(
    &self,
    label: Option<&str>,
    layout: &wgpu::BindGroupLayout,
    entries: &[wgpu::BindGroupEntry],
  ) -> wgpu::BindGroup {
    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout,
      entries,
      label,
    })
  }

  pub fn create_shader_module(
    &self,
    label: Option<&str>,
    source: &'static str,
  ) -> wgpu::ShaderModule {
    self
      .device
      .create_shader_module(wgpu::ShaderModuleDescriptor {
        label,
        source: wgpu::ShaderSource::Wgsl(source.into()),
      })
  }

  pub fn create_bind_group_layout(
    &self,
    label: Option<&str>,
    entries: &[wgpu::BindGroupLayoutEntry],
  ) -> wgpu::BindGroupLayout {
    self
      .device
      .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { entries, label })
  }

  pub fn create_pipeline_layout(
    &self,
    label: Option<&str>,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    push_constant_ranges: &[wgpu::PushConstantRange],
  ) -> wgpu::PipelineLayout {
    self
      .device
      .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label,
        bind_group_layouts,
        push_constant_ranges,
      })
  }

  pub fn create_pipeline(
    &self,
    label: Option<&str>,
    layout: Option<&wgpu::PipelineLayout>,
    vertex: wgpu::VertexState,
    fragment: Option<wgpu::FragmentState>,
    primitive: wgpu::PrimitiveState,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
  ) -> wgpu::RenderPipeline {
    self
      .device
      .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout,
        vertex,
        fragment,
        primitive,
        depth_stencil,
        multisample,
        multiview: None,
      })
  }

  pub fn update_buffer(&self, buffer: &wgpu::Buffer, offset: u64, data: &[u8]) {
    self.queue.write_buffer(buffer, offset, data);
  }

  pub fn update_texture(
    &self,
    texture: wgpu::ImageCopyTexture,
    data: &[u8],
    data_layout: wgpu::ImageDataLayout,
    size: wgpu::Extent3d,
  ) {
    self.queue.write_texture(texture, data, data_layout, size);
  }

  pub fn encode_commands(&self, f: &dyn Fn(&mut wgpu::CommandEncoder)) {
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });
    f(&mut encoder);
    self.queue.submit(std::iter::once(encoder.finish()));
  }
}
