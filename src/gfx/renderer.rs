use crate::components::StaticCamera;
use crate::core::AppData;
use flux_gfx::{
  buffer::UniformBuffer,
  device::RenderDevice,
  pipeline::{DescriptorWrite, GraphicsPipeline, GraphicsPipelineDesc},
  texture::{Format, Texture},
};

pub trait Renderer {
  fn new(render_device: &RenderDevice) -> Self
  where
    Self: Sized;
  fn render(&mut self, app: AppData, render_device: &RenderDevice);
  fn on_resize(&mut self, render_device: &RenderDevice, new_extent: &winit::dpi::PhysicalSize<u32>);
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
  pub view: [[f32; 4]; 4],
  pub projection: [[f32; 4]; 4],
}

pub struct StandardRenderer {
  camera_buffer: UniformBuffer<CameraUniform>,
  pipeline_opaque: GraphicsPipeline,
  pipeline_overlay: GraphicsPipeline,
}
impl Renderer for StandardRenderer {
  fn new(render_device: &RenderDevice) -> Self {
    let camera_buffer = UniformBuffer::new();
    let pipeline_opaque = render_device.create_graphics_pipeline(
      &GraphicsPipelineDesc::new()
        .vertex_shader(include_bytes!("shaders/opaque.vert.spv"))
        .fragment_shader(include_bytes!("shaders/opaque.frag.spv"))
        .viewport(0.0, 0.0, 400.0, 400.0, 0.0..1.0),
      None,
    );
    let pipeline_overlay = render_device.create_graphics_pipeline(
      &GraphicsPipelineDesc::new()
        .vertex_shader(include_bytes!("shaders/overlay.vert.spv"))
        .fragment_shader(include_bytes!("shaders/overlay.frag.spv"))
        .viewport(0.0, 0.0, 400.0, 400.0, 0.0..1.0),
      None,
    );

    Self {
      camera_buffer,
      pipeline_opaque,
      pipeline_overlay,
    }
  }
  fn render(&mut self, app: AppData, render_device: &RenderDevice) {
    let camera = StaticCamera::perspective(90f32.to_radians(), 1.0, 0.01, 1000.0);
    self.camera_buffer.map(|buffer: &mut CameraUniform| {
      buffer.view = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 1.0)).to_cols_array_2d();
      buffer.projection = camera.projection().to_cols_array_2d();
    });
    // let world = app.world();
    // let transform_storage = world.read_storage::<Transform>();
    // let camera_storage = world.read_storage::<Camera>();
    // let mesh_storage = world.read_storage::<Mesh>();

    // for (transform, camera) in (&transform_storage, &camera_storage).join().take(1) {
    //   self.camera_buffer.data.view = transform.to_matrix().to_cols_array_2d();
    //   self.camera_buffer.data.projection = camera.projection().to_cols_array_2d();
    //   render_device.update_buffer(
    //     &self.camera_buffer.buffer,
    //     bytemuck::cast_slice(&[self.camera_buffer.data]),
    //   );
    // }

    render_device.execute_frame(|command_list| {
      command_list.begin_final_pass();
      command_list.bind_graphics_pipeline(&self.pipeline_opaque);
      // command_list.bind_vertex_buffer(&vertex_buffer);
      // command_list.bind_index_buffer(&index_buffer);
      command_list.bind_descriptors(0, &[DescriptorWrite::buffer(0, &self.camera_buffer)]);
      // command_list.draw_indexed(indices.len() as u32, 1, 0, 0, 0);
      command_list.end_render_pass();
    });
  }
  fn on_resize(
    &mut self,
    render_device: &RenderDevice,
    new_extent: &winit::dpi::PhysicalSize<u32>,
  ) {
    self.pipeline_opaque = render_device.create_graphics_pipeline(
      &GraphicsPipelineDesc::new()
        .vertex_shader(include_bytes!("shaders/opaque.vert.spv"))
        .fragment_shader(include_bytes!("shaders/opaque.frag.spv"))
        .viewport(
          0.0,
          0.0,
          new_extent.width as f32,
          new_extent.height as f32,
          0.0..1.0,
        ),
      None,
    );
    self.pipeline_overlay = render_device.create_graphics_pipeline(
      &GraphicsPipelineDesc::new()
        .vertex_shader(include_bytes!("shaders/overlay.vert.spv"))
        .fragment_shader(include_bytes!("shaders/overlay.frag.spv"))
        .viewport(
          0.0,
          0.0,
          new_extent.width as f32,
          new_extent.height as f32,
          0.0..1.0,
        ),
      None,
    );
  }
}
