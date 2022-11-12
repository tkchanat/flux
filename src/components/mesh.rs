use crate::core::node::Component;
use flux_gfx::buffer::{IndexBuffer, VertexBuffer};

pub struct Mesh {
  pub renderable: bool,
  pub(crate) vertex_buffer: VertexBuffer,
  pub(crate) index_buffer: IndexBuffer,
}
impl Mesh {
  pub fn new(vertex_buffer: VertexBuffer, index_buffer: IndexBuffer) -> Self {
    Self {
      renderable: true,
      vertex_buffer,
      index_buffer,
    }
  }
}
#[typetag::serde]
impl Component for Mesh {
  fn type_name() -> &'static str {
    "Mesh"
  }
}
impl serde::ser::Serialize for Mesh {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    todo!()
  }
}
impl<'de> serde::de::Deserialize<'de> for Mesh {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    todo!()
  }
}
