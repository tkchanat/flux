use flux_gfx::buffer::{IndexBuffer, VertexBuffer};

use crate::core::node::Component;

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

pub struct RawMeshData {
  pub positions: Vec<glam::Vec3>,
  pub indices: Option<Vec<u32>>,
  pub normals: Option<Vec<glam::Vec3>>,
  pub tangents: Option<Vec<glam::Vec3>>,
  pub texcoords: Option<Vec<glam::Vec2>>,
  pub colors: Option<Vec<glam::Vec3>>,
}

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
          vertex_buffer: VertexBuffer::from_slice(vertices.as_slice()),
          index_buffer: indices
            .as_ref()
            .and_then(|indices| Some(IndexBuffer::new(indices.as_slice()))),
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
    let sphere = uv_sphere(10, 10, radius);
    Self::new(
      sphere.positions,
      sphere.normals,
      sphere.texcoords,
      sphere.indices,
    )
  }
}
// impl Component for Mesh {}

fn uv_sphere(segments: u32, rings: u32, radius: f32) -> RawMeshData {
  use std::f32::consts::PI;

  let mut positions = Vec::new();
  let mut normals = Vec::new();
  let mut texcoords = Vec::new();

  // Vertices
  positions.push(glam::Vec3::new(0.0, radius, 0.0));
  normals.push(glam::Vec3::new(0.0, radius, 0.0).normalize());
  texcoords.push(glam::Vec2::new(0.0, 0.0));
  for j in 1..rings {
    let v = j as f32 / (rings - 1) as f32;
    let polar = PI * j as f32 / rings as f32;
    let sp = polar.sin();
    let cp = polar.cos();
    for i in 0..segments {
      let u = i as f32 / (segments - 1) as f32;
      let azimuth = 2.0 * PI * i as f32 / segments as f32;
      let sa = azimuth.sin();
      let ca = azimuth.cos();
      let x = sp * ca * radius;
      let y = cp * radius;
      let z = sp * sa * radius;
      positions.push(glam::Vec3::new(x, y, z));
      normals.push(glam::Vec3::new(x, y, z).normalize());
      texcoords.push(glam::Vec2::new(u, v));
    }
  }
  positions.push(glam::Vec3::new(0.0, -radius, 0.0));
  normals.push(glam::Vec3::new(0.0, -radius, 0.0).normalize());
  texcoords.push(glam::Vec2::new(1.0, 1.0));

  // Indices
  let mut indices = Vec::new();
  for i in 0..segments {
    let a = i + 1;
    let b = (i + 1).rem_euclid(segments) + 1;
    indices.extend_from_slice(&[0, b, a]);
  }

  for j in 0..rings - 2 {
    let a_start = j * segments + 1;
    let b_start = (j + 1) * segments + 1;
    for i in 0..segments {
      let a = a_start + i;
      let a1 = a_start + (i + 1).rem_euclid(segments);
      let b = b_start + i;
      let b1 = b_start + (i + 1).rem_euclid(segments);
      indices.extend_from_slice(&[a, a1, b1]);
      indices.extend_from_slice(&[a, b1, b]);
    }
  }

  for i in 0..segments {
    let a = i + segments * (rings - 2) + 1;
    let b = (i + 1).rem_euclid(segments) + segments * (rings - 2) + 1;
    indices.extend_from_slice(&[(positions.len() - 1) as u32, a, b]);
  }

  RawMeshData {
    positions,
    indices: Some(indices),
    normals: Some(normals),
    tangents: None,
    texcoords: Some(texcoords),
    colors: None,
  }
}
