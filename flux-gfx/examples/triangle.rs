use flux_gfx::{
  backend::Vulkan,
  buffer::{Buffer, BufferUsage, VertexBuffer},
  device::RenderDevice,
  pipeline::GraphicsPipelineDesc,
  texture::Format,
};

fn main() {
  let render_device = RenderDevice::new(None);

  let texture = render_device.create_texture((1024, 1024, 1), Format::R8G8B8A8_SRGB);
  let buffer = Buffer::new(BufferUsage::TRANSFER_DST, 1024 * 1024 * 4);

  let vertices: [[f32; 3]; 6] = [
    // position       // color
    [-0.5, 0.5, 0.0], [0.0, 0.0, 1.0],
    [0.5, 0.5, 0.0],  [0.0, 1.0, 0.0],
    [0.0, -0.5, 0.0], [1.0, 0.0, 0.0],
  ];
  let vertex_buffer = VertexBuffer::new(vertices);
  let render_pass = render_device.create_render_pass(&[texture], None);
  let pipeline = render_device.create_graphics_pipeline(
    &GraphicsPipelineDesc {
      vs_spv: include_bytes!("shaders/triangle.vert.spv"),
      fs_spv: include_bytes!("shaders/triangle.frag.spv"),
    },
    &render_pass,
  );
  let command_list = render_device.create_command_list();
  command_list
    .begin_render_pass(&render_pass)
    .bind_graphics_pipeline(&pipeline)
    .bind_vertex_buffer(&vertex_buffer)
    .draw(3, 1, 0, 0)
    .end_render_pass()
    .copy_texture_to_buffer(&texture, &buffer)
    .submit();

  buffer.map(|buffer_content: &mut [u8; 1024 * 1024 * 4]| {
    let image =
      image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();
  });
}
