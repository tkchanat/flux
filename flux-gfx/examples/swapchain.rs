use flux_gfx::{
  backend::Vulkan,
  buffer::{Buffer, BufferUsage, VertexBuffer, IndexBuffer},
  device::RenderDevice,
  pipeline::{GraphicsPipelineDesc, Viewport},
  texture::Format,
};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

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
  let vertices: [[f32; 3]; 6] = [
    // position       // color
    [-0.5, 0.5, 0.0], [0.0, 0.0, 1.0],
    [0.5, 0.5, 0.0],  [0.0, 1.0, 0.0],
    [0.0, -0.5, 0.0], [1.0, 0.0, 0.0],
  ];
  let indices: [u32; 3] = [0, 1, 2];

  let vertex_buffer = VertexBuffer::new(vertices);
  let index_buffer = IndexBuffer::new(&indices);
  let pipeline = render_device.create_graphics_pipeline(
    &GraphicsPipelineDesc::new()
      .vertex_shader(include_bytes!("shaders/triangle.vert.spv"))
      .fragment_shader(include_bytes!("shaders/triangle.frag.spv"))
      .viewport(0.0, 0.0, window_size.width as f32, window_size.height as f32, 0.0..1.0),
    None,
  );

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
      render_device.execute_frame(|command_list| {
        command_list.begin_final_pass();
        command_list.bind_graphics_pipeline(&pipeline);
        command_list.bind_vertex_buffer(&vertex_buffer);
        command_list.bind_index_buffer(&index_buffer);
        command_list.draw_indexed(indices.len() as u32, 1, 0, 0, 0);
        command_list.end_render_pass();
      });
    }
    _ => (),
  });
}
