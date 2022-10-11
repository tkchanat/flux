use specs::{Component, DenseVecStorage};
use specs_derive::Component;

use crate::prefabs;

use super::{
  buffer::{IndexBuffer, VertexBuffer},
  procedural, RenderDevice,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
  position: [f32; 3],
  tex_coords: [f32; 2],
}

#[derive(Component)]
pub struct Mesh {
  pub vertex_buffer: VertexBuffer,
  pub index_buffer: Option<IndexBuffer>,
  pub index_count: u32,
}
impl Mesh {
  pub fn from_mesh(device: &RenderDevice, mesh: &prefabs::Mesh) -> Self {
    let mesh_data = mesh.try_get_data().expect("Mesh must have data");
    let vertices = itertools::izip!(&mesh_data.vertices, mesh_data.uvs.as_ref().unwrap())
      .map(|(position, tex_coords)| Vertex {
        position: position.to_array(),
        tex_coords: tex_coords.to_array(),
      })
      .collect::<Vec<_>>();
    let vertex_buffer = VertexBuffer::new(device, bytemuck::cast_slice(vertices.as_slice()));
    let (index_buffer, index_count) = match &mesh_data.indices {
      Some(indices) => (
        Some(IndexBuffer::new(
          device,
          bytemuck::cast_slice(indices.as_slice()),
        )),
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
  pub fn from_geomsphere(device: &RenderDevice, sphere: &prefabs::GeomSphere) -> Self {
    let uv_sphere = procedural::create_uv_sphere(10, 10, sphere.radius);
    let vertices = itertools::izip!(&uv_sphere.positions, &uv_sphere.texcoords.unwrap())
      .map(|(position, tex_coords)| Vertex {
        position: *position,
        tex_coords: *tex_coords,
      })
      .collect::<Vec<_>>();
    let vertex_buffer = VertexBuffer::new(device, bytemuck::cast_slice(vertices.as_slice()));
    let (index_buffer, index_count) = match &uv_sphere.indices {
      Some(indices) => (
        Some(IndexBuffer::new(
          device,
          bytemuck::cast_slice(indices.as_slice()),
        )),
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
