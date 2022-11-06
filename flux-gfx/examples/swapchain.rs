use flux_gfx::{
  backend::Vulkan,
  buffer::{Buffer, BufferUsage, VertexBuffer},
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
  let vertex_buffer = VertexBuffer::new(vertices);
  let pipeline = render_device.create_graphics_pipeline(
    &GraphicsPipelineDesc {
      vs_spv: include_bytes!("shaders/triangle.vert.spv"),
      fs_spv: include_bytes!("shaders/triangle.frag.spv"),
      viewport: Viewport::new(0.0, 0.0, window_size.width as f32, window_size.height as f32),
    },
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
        command_list.draw(3, 1, 0, 0);
        command_list.end_render_pass();
      });
    }
    _ => (),
  });

  // let texture = render_device.create_texture((1024, 1024, 1), Format::R8G8B8A8_SRGB);
  // let buffer = Buffer::new(BufferUsage::TRANSFER_DST, 1024 * 1024 * 4);

  // let vertices: [[f32; 3]; 6] = [
  //   // position       // color
  //   [-0.5, 0.5, 0.0],
  //   [0.0, 0.0, 1.0],
  //   [0.5, 0.5, 0.0],
  //   [0.0, 1.0, 0.0],
  //   [0.0, -0.5, 0.0],
  //   [1.0, 0.0, 0.0],
  // ];
  // let vertex_buffer = VertexBuffer::new(vertices);
  // let render_pass = render_device.create_render_pass(&[texture], None);
  // let pipeline = render_device.create_graphics_pipeline(
  //   &GraphicsPipelineDesc {
  //     vs_spv: include_bytes!("shaders/triangle.vert.spv"),
  //     fs_spv: include_bytes!("shaders/triangle.frag.spv"),
  //   },
  //   &render_pass,
  // );
  // let command_list = render_device.create_command_list();
  // command_list
  //   .begin_render_pass(&render_pass)
  //   .bind_graphics_pipeline(&pipeline)
  //   .bind_vertex_buffer(&vertex_buffer)
  //   .draw(3, 1, 0, 0)
  //   .end_render_pass()
  //   .copy_texture_to_buffer(&texture, &buffer)
  //   .submit();

  // buffer.map(|buffer_content: &mut [u8; 1024 * 1024 * 4]| {
  //   let image =
  //     image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
  //   image.save("image.png").unwrap();
  // });
}
