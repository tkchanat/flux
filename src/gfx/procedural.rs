use std::f32::consts::PI;

pub struct ProceduralMesh {
  pub positions: Vec<[f32; 3]>,
  pub indices: Option<Vec<u16>>,
  pub normals: Option<Vec<[f32; 3]>>,
  pub tangents: Option<Vec<[f32; 3]>>,
  pub texcoords: Option<Vec<[f32; 2]>>,
  pub colors: Option<Vec<[f32; 3]>>,
}

pub fn create_uv_sphere(segments: u16, rings: u16, radius: f32) -> ProceduralMesh {
  let mut positions = Vec::new();
  let mut texcoords = Vec::new();

  // Vertices
  positions.push([0.0, radius, 0.0]);
  texcoords.push([0.0, 0.0]);
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
      positions.push([x, y, z]);
      texcoords.push([u, v]);
    }
  }
  positions.push([0.0, -radius, 0.0]);
  texcoords.push([1.0, 1.0]);

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
    indices.extend_from_slice(&[(positions.len() - 1) as u16, a, b]);
  }

  ProceduralMesh {
    positions,
    indices: Some(indices),
    normals: None,
    tangents: None,
    texcoords: Some(texcoords),
    colors: None,
  }
}
