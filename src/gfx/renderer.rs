use specs::{Join, WorldExt};

use super::{
  BindGroup, BindGroupEntry, MaterialOverlay, Mesh, RenderDeviceOld, GraphicsPipeline, Transform,
  UniformBuffer, VertexBuffer,
};
use crate::{core::AppData, prefabs::Camera};

pub trait Renderer {
  fn new(render_device: &mut RenderDeviceOld) -> Self
  where
    Self: Sized;
  fn render(
    &mut self,
    app: AppData,
    render_device: &RenderDeviceOld,
  ) -> Result<(), wgpu::SurfaceError>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
  position: [f32; 3],
  tex_coords: [f32; 2],
}

impl Vertex {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
  pub view: [[f32; 4]; 4],
  pub projection: [[f32; 4]; 4],
}

pub struct SimpleRenderer {
  pipeline_opaque: GraphicsPipeline,
  pipeline_overlay: GraphicsPipeline,
  camera_buffer: UniformBuffer<CameraUniform>,
  camera_bind_group: BindGroup,
  depth_stencil: wgpu::Texture,
}
impl Renderer for SimpleRenderer {
  fn new(render_device: &mut RenderDeviceOld) -> Self {
    let camera_buffer = UniformBuffer::new(render_device);
    let camera_bind_group_layout =
      render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
      });
    let camera_bind_group = render_device.create_bind_group(
      &camera_bind_group_layout,
      &[BindGroupEntry::Buffer(0, &camera_buffer.buffer)],
    );

    let depth_stencil = render_device
      .device
      .create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
          width: 400,  // TODO
          height: 400, // TODO
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      });

    let pipeline_opaque = render_device.create_render_pipeline(
      "src/gfx/shaders/opaque.vert.spv",
      Some("src/gfx/shaders/opaque.frag.spv"),
    );
    let pipeline_overlay = render_device.create_render_pipeline(
      "src/gfx/shaders/overlay.vert.spv",
      Some("src/gfx/shaders/overlay.frag.spv"),
    );

    Self {
      pipeline_opaque,
      pipeline_overlay,
      camera_buffer,
      camera_bind_group,
      depth_stencil,
    }
  }
  fn render(
    &mut self,
    app: AppData,
    render_device: &RenderDeviceOld,
  ) -> Result<(), wgpu::SurfaceError> {
    let output = render_device.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let depth_view = self
      .depth_stencil
      .create_view(&wgpu::TextureViewDescriptor::default());

    let world = app.world();
    let transform_storage = world.read_storage::<Transform>();
    let camera_storage = world.read_storage::<Camera>();
    let mesh_storage = world.read_storage::<Mesh>();

    for (transform, camera) in (&transform_storage, &camera_storage).join().take(1) {
      self.camera_buffer.data.view = transform.to_matrix().to_cols_array_2d();
      self.camera_buffer.data.projection = camera.projection().to_cols_array_2d();
      render_device.update_buffer(
        &self.camera_buffer.buffer,
        bytemuck::cast_slice(&[self.camera_buffer.data]),
      );
    }

    let mut encoder =
      render_device
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("Render Encoder"),
        });
    render_device.begin_render_pass(
      &mut encoder,
      &wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.1,
              g: 0.2,
              b: 0.3,
              a: 1.0,
            }),
            store: true,
          },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
          view: &depth_view,
          depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: false,
          }),
          stencil_ops: None,
        }),
      },
      |render_pass| {
        // Opaque Pass
        render_pass.set_pipeline(&self.pipeline_opaque);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for (transform, mesh) in (&transform_storage, &mesh_storage).join() {
          if !mesh.renderable {
            continue;
          }
          let mesh_data = mesh.gpu_data.as_ref().unwrap();
          render_pass.set_vertex_buffer(0, &mesh_data.vertex_buffer);
          render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            bytemuck::cast_slice(&[transform.to_matrix().to_cols_array()]),
          );
          match &mesh_data.index_buffer {
            Some(index_buffer) => {
              render_pass.set_index_buffer(index_buffer);
              render_pass.draw_indexed(0..mesh_data.count, 0, 0..1);
            }
            None => {
              render_pass.draw(0..mesh_data.count, 0..1);
            }
          }
        }

        // Overlay Pass
        let material_storage = world.read_storage::<MaterialOverlay>();
        render_pass.set_pipeline(&self.pipeline_overlay);
        for (mesh, material) in (&mesh_storage, &material_storage).join() {
          if !mesh.renderable {
            continue;
          }
          let mesh_data = mesh.gpu_data.as_ref().unwrap();
          render_pass.set_bind_group(0, &material.bind_group, &[]);
          render_pass.set_vertex_buffer(0, &mesh_data.vertex_buffer);
          render_pass.set_index_buffer(mesh_data.index_buffer.as_ref().unwrap());
          render_pass.draw_indexed(0..mesh_data.count, 0, 0..1);
        }
      },
    );

    render_device
      .queue
      .submit(std::iter::once(encoder.finish()));

    output.present();

    Ok(())
  }
}
