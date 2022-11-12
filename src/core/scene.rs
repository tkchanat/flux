use flux_gfx::buffer::{IndexBuffer, VertexBuffer};

use super::node::Node;
use crate::components::Mesh;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Scene {
  pub root: Node,
  resource_registry: ResourceRegistry,
}
impl Scene {
  pub fn new() -> Self {
    Self {
      root: Node::new("root"),
      resource_registry: ResourceRegistry::default(),
    }
  }

  pub fn from_gltf(path: &str) -> Self {
    let (gltf, buffers, _) =
      gltf::import(path).expect(format!("Unable to load gltf file {}", path).as_str());
    let mut registry = ResourceRegistry::default();

    let root = Node::new("root");
    for scene in gltf.scenes() {
      for node in scene.nodes() {
        root.add_child(translate_gltf_node(&mut registry, &buffers, node));
      }
    }
    Self {
      root,
      resource_registry: ResourceRegistry::default(),
    }
  }
}
fn translate_gltf_node(
  registry: &mut ResourceRegistry,
  buffers: &Vec<gltf::buffer::Data>,
  node: gltf::Node,
) -> Node {
  let result = Node::new(node.name().unwrap_or("Unnamed"));
  if let Some(mesh) = node.mesh() {
    result.add_component(translate_gltf_mesh(registry, buffers, mesh));
  }
  // Process child nodes
  for child in node.children() {
    result.add_child(translate_gltf_node(registry, &buffers, child));
  }
  result
}
fn translate_gltf_mesh(
  registry: &mut ResourceRegistry,
  buffers: &Vec<gltf::buffer::Data>,
  mesh: gltf::Mesh,
) -> Mesh {
  let prim = mesh.primitives().take(1).next().unwrap();

  let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
  let positions = reader
    .read_positions()
    .expect("Mesh must have positions!")
    .collect::<Vec<_>>();
  let indices = reader.read_indices().and_then(|ty| match ty {
    gltf::mesh::util::ReadIndices::U8(_) => todo!(),
    gltf::mesh::util::ReadIndices::U16(iter) => Some(iter.map(|i| i as u32).collect::<Vec<_>>()),
    gltf::mesh::util::ReadIndices::U32(iter) => Some(iter.collect::<Vec<_>>()),
  });
  let normals = reader
    .read_normals()
    .and_then(|iter| Some(iter.collect::<Vec<_>>()));
  let texcoords = reader
    .read_tex_coords(0)
    .and_then(|iter| Some(iter.into_f32().collect::<Vec<_>>()));
  let tangents = reader
    .read_tangents()
    .and_then(|iter| Some(iter.collect::<Vec<_>>()));
  let colors = reader.read_colors(0).and_then(|ty| match ty {
    gltf::mesh::util::ReadColors::RgbU8(_) => todo!(),
    gltf::mesh::util::ReadColors::RgbU16(_) => todo!(),
    gltf::mesh::util::ReadColors::RgbF32(_) => todo!(),
    gltf::mesh::util::ReadColors::RgbaU8(_) => todo!(),
    gltf::mesh::util::ReadColors::RgbaU16(_) => todo!(),
    gltf::mesh::util::ReadColors::RgbaF32(_) => todo!(),
  });

  let mesh = {
    let indices = indices.as_ref().map_or_else(
      || (0..(positions.len() / 3) as u32).collect::<Vec<u32>>(),
      |indices| indices.to_vec(),
    );
    let vertices = {
      let normals = normals.as_ref().map_or_else(
        || {
          indices
            .chunks(3)
            .map(|indices| {
              let p0 = glam::Vec3::from_array(positions[indices[0] as usize]);
              let p1 = glam::Vec3::from_array(positions[indices[1] as usize]);
              let p2 = glam::Vec3::from_array(positions[indices[2] as usize]);
              let normal = (p1 - p0).cross(p2 - p0).to_array();
              [normal; 3]
            })
            .flatten()
            .collect::<Vec<_>>()
        },
        |normals| normals.to_vec(),
      );
      let texcoords = texcoords.as_ref().map_or_else(
        || {
          indices
            .chunks(3)
            .map(|indices| [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]])
            .flatten()
            .collect::<Vec<_>>()
        },
        |texcoords| texcoords.to_vec(),
      );
      assert!(positions.len() == normals.len());
      assert!(normals.len() == texcoords.len());
      itertools::izip!(&positions, &normals, &texcoords)
        .map(|(position, normal, texcoord)| {
          position
            .iter()
            .cloned()
            .chain(normal.iter().cloned())
            .chain(texcoord.iter().cloned())
            .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>()
    };
    let vertex_buffer = VertexBuffer::from_slice(vertices.as_slice());
    let index_buffer = IndexBuffer::new(indices.as_slice());
    Mesh::new(vertex_buffer, index_buffer)
  };

  registry.mesh_data.push(MeshData {
    positions,
    indices,
    normals,
    texcoords,
    tangents,
    colors,
  });
  mesh
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MeshData {
  positions: Vec<[f32; 3]>,
  indices: Option<Vec<u32>>,
  normals: Option<Vec<[f32; 3]>>,
  texcoords: Option<Vec<[f32; 2]>>,
  tangents: Option<Vec<[f32; 4]>>,
  colors: Option<Vec<[f32; 3]>>,
}
impl ResourceEntry for MeshData {}

#[derive(serde::Serialize, serde::Deserialize)]
enum TextureData {
  Raw(Vec<u8>),
  Compressed,
}

pub(crate) trait ResourceEntry {}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct ResourceRegistry {
  mesh_data: Vec<MeshData>,
  texture_data: Vec<TextureData>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::components::Mesh;

  #[test]
  fn test_serde() {
    let scene = Scene::new();
    let cube = Node::new("cube");
    // cube.add_component(Mesh::cube());
    scene.root.add_child(cube);

    let ser = serde_json::to_string(&scene).expect("Unable to serialize scene");
    println!("Serialized scene: {:?}", ser);
  }
}
