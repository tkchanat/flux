use specs::{Component, DenseVecStorage};
use specs_derive::Component;

use super::{
  buffer::{IndexBuffer, VertexBuffer},
  procedural,
};
use crate::core::Node;

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

// trait Depends<T: Component> {
//   fn on_added(&mut self, node: &Node, component: &T);
// }

#[derive(Component)]
pub struct Mesh {
  pub vertex_buffer: VertexBuffer,
  pub index_buffer: Option<IndexBuffer>,
  pub index_count: u32,
}

// impl Depends<crate::prefabs::Mesh> for Mesh {
//   fn on_added(&mut self, node: &Node, mesh: &crate::prefabs::Mesh) {
//     let mesh_data = mesh.try_get_data().expect("Mesh must have data");
//     let vertices = itertools::izip!(&mesh_data.vertices, mesh_data.uvs.as_ref().unwrap())
//       .map(|(position, tex_coords)| Vertex {
//         position: position.to_array(),
//         tex_coords: tex_coords.to_array(),
//       })
//       .collect::<Vec<_>>();
//     let vertex_buffer = VertexBuffer::new(bytemuck::cast_slice(vertices.as_slice()));
//     let (index_buffer, index_count) = match &mesh_data.indices {
//       Some(indices) => (
//         Some(IndexBuffer::new(bytemuck::cast_slice(indices.as_slice()))),
//         indices.len() as u32,
//       ),
//       None => (None, 0),
//     };

//     node.add_component(Mesh {
//       vertex_buffer,
//       index_buffer,
//       index_count,
//     });
//     println!("gfx::Mesh added!");
//   }
// }

impl Mesh {
  pub fn sphere(segments: u16, rings: u16, radius: f32) -> Self {
    let uv_sphere = procedural::create_uv_sphere(segments, rings, radius);
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
