use specs::{Join, WorldExt};

use super::{Mesh, RenderDevice, Transform, UniformBuffer, VertexBuffer};
use crate::{core::AppData, prefabs::Camera};

pub trait Renderer {
  fn new(device: &RenderDevice) -> Self
  where
    Self: Sized;
  fn render(&mut self, app: AppData, device: &RenderDevice) -> Result<(), wgpu::SurfaceError>;
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
  pipeline_diffuse: wgpu::RenderPipeline,
  camera_buffer: UniformBuffer<CameraUniform>,
  camera_bind_group: wgpu::BindGroup,
  depth_stencil: wgpu::Texture,
}
impl Renderer for SimpleRenderer {
  fn new(device: &RenderDevice) -> Self {
    let camera_buffer = UniformBuffer::new(device);
    let camera_bind_group_layout = device.create_bind_group_layout(
      Some("camera_bind_group_layout"),
      &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    );
    let camera_bind_group = device.create_bind_group(
      Some("camera_bind_group"),
      &camera_bind_group_layout,
      &[wgpu::BindGroupEntry {
        binding: 0,
        resource: camera_buffer.binding(),
      }],
    );

    let depth_stencil = device.create_texture(
      Some("Depth"),
      wgpu::Extent3d {
        width: 400,  // TODO
        height: 400, // TODO
        depth_or_array_layers: 1,
      },
      1,
      1,
      wgpu::TextureDimension::D2,
      wgpu::TextureFormat::Depth24PlusStencil8,
      wgpu::TextureUsages::RENDER_ATTACHMENT,
    );

    let diffuse_render_pipeline_layout = device.create_pipeline_layout(
      Some("Diffuse Render Pipeline Layout"),
      &[&camera_bind_group_layout],
      &[wgpu::PushConstantRange {
        stages: wgpu::ShaderStages::VERTEX,
        range: 0..64,
      }],
    );
    let shader_diffuse =
      device.create_shader_module(Some("Shader"), include_str!("shaders/diffuse.wgsl"));
    let pipeline_diffuse = device.create_pipeline(
      Some("Render Pipeline Diffuse"),
      Some(&diffuse_render_pipeline_layout),
      wgpu::VertexState {
        module: &shader_diffuse,
        entry_point: "vs_main",
        buffers: &[Vertex::desc()],
      },
      Some(wgpu::FragmentState {
        module: &shader_diffuse,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: device.surface_format(),
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),
      wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
    );

    Self {
      pipeline_diffuse,
      camera_buffer,
      camera_bind_group,
      depth_stencil,
    }
  }
  fn render(&mut self, app: AppData, device: &RenderDevice) -> Result<(), wgpu::SurfaceError> {
    let output = device.surface_texture()?;
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let depth_view = self
      .depth_stencil
      .create_view(&wgpu::TextureViewDescriptor::default());

    let world = app.world();
    let transform_storage = world.read_storage::<Transform>();
    let camera_storage = world.read_storage::<Camera>();
    for (transform, camera) in (&transform_storage, &camera_storage).join().take(1) {
      self.camera_buffer.data.view = transform.to_matrix().to_cols_array_2d();
      self.camera_buffer.data.projection = camera.projection().to_cols_array_2d();
      self.camera_buffer.update(device);
    }

    device.encode_commands(&|encoder: &mut wgpu::CommandEncoder| {
      let mesh_storage = world.read_storage::<Mesh>();

      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
      });

      render_pass.set_pipeline(&self.pipeline_diffuse);
      render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
      for (transform, mesh) in (&transform_storage, &mesh_storage).join() {
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.buffer.slice(..));
        render_pass.set_push_constants(
          wgpu::ShaderStages::VERTEX,
          0,
          bytemuck::cast_slice(&[transform.to_matrix().to_cols_array()]),
        );
        if let Some(index_buffer) = &mesh.index_buffer {
          render_pass.set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
          render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
      }
    });
    output.present();

    Ok(())
  }
}
