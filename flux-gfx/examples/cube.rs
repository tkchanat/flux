use flux_gfx::{
  backend::Vulkan,
  buffer::{Buffer, BufferUsage, VertexBuffer, IndexBuffer, UniformBuffer},
  device::RenderDevice,
  pipeline::{DescriptorWrite, GraphicsPipelineDesc, Viewport},
  texture::Format,
};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformBufferData {
  model: [f32; 16],
  view: [f32; 16],
  projection: [f32; 16],
}

fn main() {
  let event_loop = EventLoop::new();
  let window_size = PhysicalSize::new(400, 400);
  let window = Arc::new(
    WindowBuilder::new()
      .with_inner_size(window_size)
      .build(&event_loop)
      .unwrap(),
  );
  let render_device = RenderDevice::new(Some(window));
  let vertices : [[f32; 3]; 16] = [
    [-0.3,-0.3, 0.3], [0.0, 0.0, 1.0],
    [ 0.3,-0.3, 0.3], [1.0, 0.0, 1.0],
    [-0.3, 0.3, 0.3], [0.0, 1.0, 1.0],
    [ 0.3, 0.3, 0.3], [1.0, 1.0, 1.0],
    [-0.3,-0.3,-0.3], [0.0, 0.0, 0.0],
    [ 0.3,-0.3,-0.3], [1.0, 0.0, 0.0],
    [-0.3, 0.3,-0.3], [0.0, 1.0, 0.0],
    [ 0.3, 0.3,-0.3], [1.0, 1.0, 0.0],
  ];
  let indices: [u32; 36] = [
    2, 6, 7, 2, 3, 7, // Top
    0, 4, 5, 0, 1, 5, // Bottom
    0, 2, 6, 0, 4, 6, // Left
    1, 3, 7, 1, 5, 7, // Right
    0, 2, 3, 0, 1, 3, // Front
    4, 6, 7, 4, 5, 7, // Back
  ];

  let vertex_buffer = VertexBuffer::new(vertices);
  let index_buffer = IndexBuffer::new(&indices);
  let uniform_buffer = UniformBuffer::<UniformBufferData>::new();
  uniform_buffer.map(|ubo: &mut UniformBufferData| {
    ubo.model = glam::Mat4::IDENTITY.to_cols_array();
    ubo.view = 
      glam::Mat4::look_at_rh(glam::Vec3::Z, glam::Vec3::ZERO, glam::Vec3::Y).to_cols_array();
    ubo.projection = 
      glam::Mat4::perspective_rh(90f32.to_radians(), 1.0, 0.01, 100.0).to_cols_array();
  });
  let pipeline = render_device.create_graphics_pipeline(
    &GraphicsPipelineDesc::new()
      .vertex_shader(include_bytes!("shaders/cube.vert.spv"))
      .fragment_shader(include_bytes!("shaders/cube.frag.spv"))
      .viewport(0.0, 0.0, window_size.width as f32, window_size.height as f32, 0.0..1.0)
      .depth_test(true),
    None
  );

  let mut rotation = 0.0;
  let mut window_resized = false;
  event_loop.run(move |event, _, control_flow| match event {
    Event::WindowEvent {
      event:
        WindowEvent::CloseRequested
        | WindowEvent::KeyboardInput {
          input:
            KeyboardInput {
              state: ElementState::Pressed,
              virtual_keycode: Some(VirtualKeyCode::Escape),
              ..
            },
          ..
        },
      ..
    } => {
      *control_flow = ControlFlow::Exit;
    }
    Event::WindowEvent {
      event: WindowEvent::Resized(_),
      ..
    } => {
      window_resized = true;
    }
    Event::MainEventsCleared => {
      // Update cube transform
      uniform_buffer.map(|ubo: &mut UniformBufferData| {
        ubo.model = glam::Mat4::from_rotation_translation(
          glam::Quat::from_euler(glam::EulerRot::default(), rotation, rotation, 0.0), 
          glam::Vec3::new(0.0, 0.1 * rotation.sin(), 0.0),
        ).to_cols_array();
      });
      rotation += 0.01;

      render_device.execute_frame(|command_list| {
        command_list.begin_final_pass();
        command_list.bind_graphics_pipeline(&pipeline);
        command_list.bind_vertex_buffer(&vertex_buffer);
        command_list.bind_index_buffer(&index_buffer);
        command_list.bind_descriptors(0, &[
          DescriptorWrite::buffer(0, &uniform_buffer),
        ]);
        command_list.draw_indexed(indices.len() as u32, 1, 0, 0, 0);
        command_list.end_render_pass();
      });
    }
    _ => (),
  });
}