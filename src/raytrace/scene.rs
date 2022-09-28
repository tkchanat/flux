use super::{
  mesh::TriangleMesh,
  shape::{Sphere, Triangle},
};
use bvh::{aabb::AABB, Vector3};
use glam::{Affine3A, Vec2, Vec3};

pub enum Primitive {
  Empty,
  SphereGeometry(Sphere),
  TriangleMesh(TriangleMesh),
}

pub struct Node {
  pub prim: Primitive,
  pub children: Vec<Node>,
}

pub struct Scene {
  pub root: Node,
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

  pub fn raytracing_in_one_weekend() -> Self {
    // let mut spheres = Vec::new();
    let top_sphere = Node {
      prim: Primitive::SphereGeometry(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5)),
      children: Vec::new()
    };
    let bottom_sphere = Node {
      prim: Primitive::SphereGeometry(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0)),
      children: Vec::new()
    };

    Self {
      root: Node {
        prim: Primitive::Empty,
        children: vec![top_sphere, bottom_sphere],
      }
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
    let normals = match reader.read_normals() {
      Some(iter) => Some(iter.collect::<Vec<[f32; 3]>>()),
      None => None,
    };
    let texcoords = match reader.read_tex_coords(0) {
      Some(iter) => Some(iter.into_f32().collect::<Vec<[f32; 2]>>()),
      None => None,
    };
    match reader.read_indices() {
      Some(indices) => match indices {
        gltf::mesh::util::ReadIndices::U8(_iter) => unimplemented!(),
        gltf::mesh::util::ReadIndices::U16(iter) => {
          let indices = iter.collect::<Vec<u16>>();
          for triangle in indices.chunks(3) {
            let vertices = [
              Vec3::from_array(positions[triangle[0] as usize]),
              Vec3::from_array(positions[triangle[1] as usize]),
              Vec3::from_array(positions[triangle[2] as usize]),
            ];
            let normals = match &normals {
              Some(normals) => [
                Vec3::from_array(normals[triangle[0] as usize]),
                Vec3::from_array(normals[triangle[1] as usize]),
                Vec3::from_array(normals[triangle[2] as usize]),
              ],
              None => {
                let normal = (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]);
                [normal; 3]
              }
            };
            let texcoords = match &texcoords {
              Some(texcoords) => Some([
                Vec2::from_array(texcoords[triangle[0] as usize]),
                Vec2::from_array(texcoords[triangle[1] as usize]),
                Vec2::from_array(texcoords[triangle[2] as usize]),
              ]),
              None => None,
            };
            triangles.push(Triangle {
              vertices,
              normals,
              texcoords,
            });
          }
        }
        gltf::mesh::util::ReadIndices::U32(_iter) => unimplemented!(),
      },
      None => {
        for i in (0..positions.len()).step_by(3) {
          let vertices = [
            Vec3::from_array(positions[i + 0]),
            Vec3::from_array(positions[i + 1]),
            Vec3::from_array(positions[i + 2]),
          ];
          let normals = match &normals {
            Some(normals) => [
              Vec3::from_array(normals[i + 0]),
              Vec3::from_array(normals[i + 1]),
              Vec3::from_array(normals[i + 2]),
            ],
            None => {
              let normal = (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]);
              [normal; 3]
            }
          };
          let texcoords = match &texcoords {
            Some(texcoords) => Some([
              Vec2::from_array(texcoords[i + 0]),
              Vec2::from_array(texcoords[i + 1]),
              Vec2::from_array(texcoords[i + 2]),
            ]),
            None => None,
          };
          triangles.push(Triangle {
            vertices,
            normals,
            texcoords,
          });
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
