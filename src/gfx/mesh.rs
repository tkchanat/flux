use super::{
  buffer::{IndexBuffer, VertexBuffer},
  procedural,
};

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

pub struct Mesh {
  pub vertex_buffer: VertexBuffer,
  pub index_buffer: Option<IndexBuffer>,
  pub index_count: u32,
}

impl Mesh {
  pub fn sphere() -> Self {
    let uv_sphere = procedural::create_uv_sphere(10, 10);
    let vertices = itertools::izip!(&uv_sphere.positions, &uv_sphere.texcoords.unwrap())
      .map(|(position, tex_coords)| Vertex {
        position: *position,
        tex_coords: *tex_coords,
      })
      .collect::<Vec<_>>();
    let vertex_buffer = VertexBuffer::new(bytemuck::cast_slice(vertices.as_slice()));
    let (index_buffer, index_count) = match &uv_sphere.indices {
      Some(indices) => (
        Some(IndexBuffer::new(bytemuck::cast_slice(indices.as_slice()))),
        indices.len() as u32,
      ),
      None => (None, 0),
    };

    Self {
      vertex_buffer,
      index_buffer,
      index_count,
    }
  }
}
