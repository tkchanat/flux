use flux_gfx::{
  backend::Vulkan, buffer::BufferUsage, device::RenderDevice, pipeline::GraphicsPipelineDesc,
  texture::Format,
};

fn main() {
  let mut render_device = RenderDevice::<Vulkan>::new(None);

  let texture = render_device.create_texture((1024, 1024, 1), Format::R8G8B8A8_UNORM);
  let vertex_buffer = render_device.create_buffer_with_init(
    BufferUsage::VERTEX_BUFFER,
    [[-0.5, -0.5], [0.0, 0.5], [0.5, -0.25]],
  );
  let render_pass = render_device.create_render_pass(&[texture], None);
  let pipeline = render_device.create_graphics_pipeline(
    &GraphicsPipelineDesc {
      vs_spv: include_bytes!("shaders/test.vert.spv"),
      fs_spv: include_bytes!("shaders/test.frag.spv"),
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
    .submit();

  render_device.save_texture_to_disk(&texture);
}
