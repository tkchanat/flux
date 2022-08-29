use super::{mesh::TriangleMesh, shape::Triangle};
use bvh::{aabb::AABB, Vector3};
use glam::{Affine3A, Vec3};

pub enum Primitive {
  Empty,
  TriangleMesh(TriangleMesh),
}

pub struct Node {
  pub prim: Primitive,
  pub children: Vec<Node>,
}

pub struct Scene {
  pub root: Node,
  // accelerator: Option<Accelerator>,
}

impl Scene {
  pub fn from_gltf(path: &str) -> Self {
    let (gltf, buffers, _) = gltf::import(path).expect("Unable to load gltf file");

    let mut top_level_nodes = Vec::new();
    for scene in gltf.scenes() {
      for node in scene.nodes() {
        top_level_nodes.push(translate_node(&buffers, node));
      }
    }
    Self {
      root: Node {
        prim: Primitive::Empty,
        children: top_level_nodes,
      },
    }
  }
}

fn translate_node(buffers: &Vec<gltf::buffer::Data>, node: gltf::Node) -> Node {
  let mut prim = Primitive::Empty;
  if let Some(mesh) = node.mesh() {
    prim = translate_mesh(buffers, mesh);
  }

  let mut children = Vec::new();
  for child in node.children() {
    children.push(translate_node(buffers, child));
  }

  Node { prim, children }
}

fn translate_mesh(buffers: &Vec<gltf::buffer::Data>, mesh: gltf::Mesh) -> Primitive {
  let mut meshes = Vec::new();
  for prim in mesh.primitives() {
    let mut triangles = Vec::new();
    let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
    let positions = match reader.read_positions() {
      Some(iter) => iter.collect::<Vec<[f32; 3]>>(),
      None => continue,
    };
    match reader.read_indices() {
      Some(indices) => match indices {
        gltf::mesh::util::ReadIndices::U8(_iter) => unimplemented!(),
        gltf::mesh::util::ReadIndices::U16(iter) => {
          let indices = iter.collect::<Vec<u16>>();
          for triangle in indices.chunks(3) {
            triangles.push(Triangle::new(
              Vec3::from_array(positions[triangle[0] as usize]),
              Vec3::from_array(positions[triangle[1] as usize]),
              Vec3::from_array(positions[triangle[2] as usize]),
            ));
          }
        }
        gltf::mesh::util::ReadIndices::U32(_iter) => unimplemented!(),
      },
      None => {
        for vertices in positions.chunks(3) {
          triangles.push(Triangle::new(
            Vec3::from_array(vertices[0]),
            Vec3::from_array(vertices[1]),
            Vec3::from_array(vertices[2]),
          ));
        }
      }
    }

    let bound_min = positions
      .iter()
      .clone()
      .fold(Vector3::splat(f32::INFINITY), |acc, x| {
        acc.min(Vector3::from_slice(x))
      });
    let bound_max = positions
      .iter()
      .clone()
      .fold(Vector3::splat(-f32::INFINITY), |acc, x| {
        acc.max(Vector3::from_slice(x))
      });

    // Only read the first primitive, then terminate.
    let mesh = TriangleMesh {
      shapes: triangles,
      transform: Affine3A::IDENTITY,
      local_bound: AABB::with_bounds(bound_min, bound_max),
    };
    meshes.push(Primitive::TriangleMesh(mesh));
  }

  if meshes.len() == 1 {
    meshes.swap_remove(0)
  } else {
    Primitive::Empty
  }
}
