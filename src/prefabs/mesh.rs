use crate::{core::Node, gfx::Transform};
use specs::{Component, DenseVecStorage};
use specs_derive::Component;

#[derive(Component)]
pub struct Mesh {
  data: Option<Box<MeshData>>,
}
impl Mesh {
  pub fn new(
    vertices: Vec<glam::Vec3>,
    normals: Vec<glam::Vec3>,
    uvs: Option<Vec<glam::Vec2>>,
    indices: Option<Vec<u32>>
  ) -> Self {
    Self {
      data: Some(Box::new(MeshData {
        vertices,
        normals,
        uvs,
        indices
      })),
    }
  }
  pub fn is_readable(&self) -> bool {
    self.data.is_some()
  }
  pub fn drop_data(&mut self) {
    self.data = None
  }
  pub fn try_get_data(&self) -> Result<&MeshData, MeshAccessError> {
    if let Some(data) = &self.data {
      return Ok(data.as_ref());
    }
    Err(MeshAccessError::MeshDataDropped)
  }
}

#[derive(Debug)]
pub enum MeshAccessError {
  MeshDataDropped,
}
pub struct MeshData {
  pub vertices: Vec<glam::Vec3>,
  pub normals: Vec<glam::Vec3>,
  pub uvs: Option<Vec<glam::Vec2>>,
  pub indices: Option<Vec<u32>>,
}
