use gltf::json::Index;
use specs::{Component, DenseVecStorage};
use specs_derive::Component;

use crate::prefabs;

use super::{
  buffer::{IndexBuffer, VertexBuffer},
  procedural, RenderDeviceOld,
};

pub trait Vertex: bytemuck::Pod {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VertexP3([f32; 3]);
impl VertexP3 {
  fn new(position: &glam::Vec3) -> Self {
    Self(position.to_array())
  }
}
impl Vertex for VertexP3 {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[wgpu::VertexAttribute {
        offset: 0,
        shader_location: 0,
        format: wgpu::VertexFormat::Float32x3,
      }],
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VertexP3N3([f32; 3], [f32; 3]);
impl VertexP3N3 {
  fn new(position: &glam::Vec3, normal: &glam::Vec3) -> Self {
    Self(position.to_array(), normal.to_array())
  }
}
impl Vertex for VertexP3N3 {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: 12,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        },
      ],
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VertexP3U2([f32; 3], [f32; 2]);
impl VertexP3U2 {
  fn new(position: &glam::Vec3, uv: &glam::Vec2) -> Self {
    Self(position.to_array(), uv.to_array())
  }
}
impl Vertex for VertexP3U2 {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: 12,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VertexP3N3U2([f32; 3], [f32; 3], [f32; 2]);
impl VertexP3N3U2 {
  fn new(position: &glam::Vec3, normal: &glam::Vec3, uv: &glam::Vec2) -> Self {
    Self(position.to_array(), normal.to_array(), uv.to_array())
  }
}
impl Vertex for VertexP3N3U2 {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: 12,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: 24,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }
}

pub struct CPUMeshData {
  pub positions: Vec<glam::Vec3>,
  pub normals: Option<Vec<glam::Vec3>>,
  pub uvs: Option<Vec<glam::Vec2>>,
  pub indices: Option<Vec<u32>>,
}

pub(super) struct GPUMeshData {
  pub(super) vertex_buffer: VertexBuffer,
  pub(super) index_buffer: Option<IndexBuffer>,
  pub(super) count: u32,
}

#[derive(Component)]
pub struct Mesh {
  pub(super) gpu_data: Option<GPUMeshData>,
  pub data: Option<CPUMeshData>,
  pub renderable: bool,
}
impl Mesh {
  pub fn new(
    positions: Vec<glam::Vec3>,
    normals: Option<Vec<glam::Vec3>>,
    uvs: Option<Vec<glam::Vec2>>,
    indices: Option<Vec<u32>>,
  ) -> Self {
    let vertices: Vec<f32> = match &normals {
      Some(normals) => match &uvs {
        Some(uvs) => itertools::izip!(&positions, normals, uvs)
          .map(|(p, n, u)| [p.x, p.y, p.z, n.x, n.y, n.z, u.x, u.y])
          .flatten()
          .collect::<Vec<_>>(),
        None => itertools::izip!(&positions, normals)
          .map(|(p, n)| [p.x, p.y, p.z, n.x, n.y, n.z])
          .flatten()
          .collect::<Vec<_>>(),
      },
      None => match &uvs {
        Some(uvs) => itertools::izip!(&positions, uvs)
          .map(|(p, u)| [p.x, p.y, p.z, u.x, u.y])
          .flatten()
          .collect::<Vec<_>>(),
        None => positions
          .iter()
          .map(|p| [p.x, p.y, p.z])
          .flatten()
          .collect::<Vec<_>>(),
      },
    };
    let gpu_data = unsafe {
      crate::core::app::APP_INSTANCE.as_mut().and_then(|app| {
        Some(GPUMeshData {
          vertex_buffer: VertexBuffer::new(
            &mut app.render_device,
            bytemuck::cast_slice(vertices.as_slice()),
          ),
          index_buffer: indices.as_ref().and_then(|indices| {
            Some(IndexBuffer::new(
              &mut app.render_device,
              bytemuck::cast_slice(indices.as_slice()),
              wgpu::IndexFormat::Uint32,
            ))
          }),
          count: match indices.as_ref() {
            Some(indices) => indices.len() as u32,
            None => positions.len() as u32,
          },
        })
      })
    };
    Self {
      gpu_data,
      data: Some(CPUMeshData {
        positions,
        normals,
        uvs,
        indices,
      }),
      renderable: true,
    }
  }

  pub fn quad() -> Self {
    let positions = vec![
      glam::Vec3::new(-1.0, -1.0, 0.0),
      glam::Vec3::new(1.0, -1.0, 0.0),
      glam::Vec3::new(1.0, 1.0, 0.0),
      glam::Vec3::new(-1.0, 1.0, 0.0),
    ];
    let uvs = vec![
      glam::Vec2::new(0.0, 0.0),
      glam::Vec2::new(1.0, 0.0),
      glam::Vec2::new(1.0, 1.0),
      glam::Vec2::new(0.0, 1.0),
    ];
    let indices = vec![0, 1, 2, 2, 3, 0];
    Self::new(positions, None, Some(uvs), Some(indices))
  }

  pub fn sphere(radius: f32) -> Self {
    let sphere = procedural::create_uv_sphere(10, 10, radius);
    Self::new(
      sphere.positions,
      sphere.normals,
      sphere.texcoords,
      sphere.indices,
    )
  }
}
