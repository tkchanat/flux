use flux_gfx::{
  backend::Vulkan,
  buffer::{Buffer, BufferUsage, VertexBuffer},
  device::RenderDevice,
  pipeline::{GraphicsPipelineDesc, Viewport},
  texture::Format,
};

fn main() {
  let render_device = RenderDevice::new(None);

  const WIDTH: u32 = 1024;
  const HEIGHT: u32 = 1024;
  const BUFFER_SIZE: usize = WIDTH as usize * HEIGHT as usize * 4;
  let texture = render_device.create_texture((WIDTH, HEIGHT, 1), Format::R8G8B8A8_SRGB);
  let buffer = Buffer::new(BufferUsage::TRANSFER_DST, BUFFER_SIZE);

  let vertices: [[f32; 3]; 6] = [
    // position       // color
    [-0.5, 0.5, 0.0], [0.0, 0.0, 1.0],
    [0.5, 0.5, 0.0],  [0.0, 1.0, 0.0],
    [0.0, -0.5, 0.0], [1.0, 0.0, 0.0],
  ];
  let vertex_buffer = VertexBuffer::new(vertices);
  let render_pass = render_device.create_render_pass(&[texture], None);
  let pipeline = render_device.create_graphics_pipeline(
    &GraphicsPipelineDesc::new()
      .vertex_shader(include_bytes!("shaders/triangle.vert.spv"))
      .fragment_shader(include_bytes!("shaders/triangle.frag.spv"))
      .viewport(0.0, 0.0, WIDTH as f32, HEIGHT as f32, 0.0..1.0),
    Some(&render_pass),
  );
  let mut command_list = render_device.create_command_list();
  command_list.begin_render_pass(&render_pass);
  command_list.bind_graphics_pipeline(&pipeline);
  command_list.bind_vertex_buffer(&vertex_buffer);
  command_list.draw(3, 1, 0, 0);
  command_list.end_render_pass();
  command_list.copy_texture_to_buffer(&texture, &buffer);
  command_list.submit();

  buffer.map(|buffer_content: &mut [u8; BUFFER_SIZE]| {
    let image =
      image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(WIDTH, HEIGHT, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();
  });
}
