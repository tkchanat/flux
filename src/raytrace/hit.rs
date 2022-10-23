use crate::math::Ray;

use super::{scene::Primitive, shape::Shape};

pub(super) struct Hit<'a> {
  pub primitive: Option<&'a Primitive>,
  pub shape: Option<&'a Shape>,
  pub p: glam::Vec3A,
  pub ng: glam::Vec3A,
  pub ns: glam::Vec3A,
  pub uv: glam::Vec2,
  pub t: f32,
  pub dpdu: glam::Vec3A,
  pub dpdv: glam::Vec3A,
  pub front: bool,
}

impl<'a> Default for Hit<'a> {
  fn default() -> Self {
    Self {
      primitive: None,
      shape: None,
      p: glam::Vec3A::ZERO,
      ng: glam::Vec3A::ZERO,
      ns: glam::Vec3A::ZERO,
      uv: glam::Vec2::ZERO,
      t: f32::INFINITY,
      dpdu: glam::Vec3A::ZERO,
      dpdv: glam::Vec3A::ZERO,
      front: false,
    }
  }
}

impl<'a> Hit<'a> {
  pub fn spawn_ray(&self, direction: &glam::Vec3A) -> Ray {
    match self.primitive {
      Some(primitive) => match primitive {
        Primitive::Empty => todo!(),
        Primitive::Camera(_) => todo!(),
        Primitive::Sphere(_, _) => todo!(),
        Primitive::TriangleMesh(mesh) => {
          let direction = mesh
            .world_to_object
            .inverse()
            .transform_vector3a(*direction);
          Ray {
            origin: self.p,
            direction,
            t_min: 0.001,
            t_max: f32::INFINITY,
          }
        }
      },
      None => Ray::new(self.p, *direction),
    }
  }

  pub fn local_to_world(&self, v: glam::Vec3A) -> glam::Vec3A {
    assert!(self.ns.is_normalized());
    let tangent = self.dpdu.normalize();
    let bitangent = self.ns.cross(tangent);
    glam::Mat3A::from_cols(tangent, bitangent, self.ns).mul_vec3a(v)
  }

  pub fn world_to_local(&self, v: glam::Vec3A) -> glam::Vec3A {
    assert!(self.ns.is_normalized());
    let tangent = self.dpdu.normalize();
    let bitangent = self.ns.cross(tangent);
    glam::Mat3A::from_cols(tangent, bitangent, self.ns)
      .transpose()
      .mul_vec3a(v)
  }
}
