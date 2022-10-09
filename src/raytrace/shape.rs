use super::hit::Hit;
use crate::math::{coordinate_system, Ray};
use bvh::aabb::{Bounded, AABB};
use std::{f32::consts::PI, sync::Arc};

pub(super) enum Shape {
  Sphere(Sphere),
  Triangle(Triangle),
}
impl Shape {
  pub(super) fn aabb(&self) -> bvh::aabb::AABB {
    match &self {
      Shape::Sphere(sphere) => sphere.aabb(),
      Shape::Triangle(triangle) => triangle.aabb(),
    }
  }
  pub(super) fn intersect<'a>(&'a self, ray: &Ray, hit: &mut Hit<'a>) -> bool {
    match &self {
      Shape::Sphere(sphere) => sphere.intersect(ray, hit),
      Shape::Triangle(triangle) => triangle.intersect(ray, hit),
    }
  }
}

pub struct Sphere {
  center: glam::Vec3,
  radius: f32,
}

impl Sphere {
  pub fn new(center: glam::Vec3, radius: f32) -> Self {
    Self { center, radius }
  }
  fn intersect<'a>(&'a self, ray: &Ray, hit: &mut Hit<'a>) -> bool {
    let center = glam::Vec3A::from(self.center);

    let oc = ray.origin - center;
    let a = ray.direction.length_squared();
    let half_b = oc.dot(ray.direction);
    let c = oc.length_squared() - self.radius * self.radius;
    let det = half_b * half_b - a * c;
    if det < 0.0 {
      return false;
    }

    // Find the nearest root that lies in the acceptable range.
    let sqrtd = det.sqrt();
    let mut t = (-half_b - sqrtd) / a;
    if t < ray.t_min || ray.t_max < t {
      t = (-half_b + sqrtd) / a;
      if t < ray.t_min || t > ray.t_max {
        return false;
      }
    }

    hit.p = ray.origin + t * ray.direction;
    hit.t = t.min(hit.t);
    hit.ng = (hit.p - center).normalize();
    hit.ns = hit.ng;

    let theta = (-hit.ns.y).acos();
    let phi = (-hit.ns.z).atan2(hit.ns.x) + PI;
    hit.uv = glam::Vec2::new(phi / (2.0 * PI), theta / PI);

    let north = glam::Vec3A::Y;
    hit.dpdu = north.cross(hit.ns).normalize();
    hit.dpdv = hit.ns.cross(hit.dpdu);
    hit.front = hit.ng.dot(-ray.direction) > 0.0;
    true
  }
}
impl Bounded for Sphere {
  fn aabb(&self) -> AABB {
    let center = bvh::Vector3::new(self.center.x, self.center.y, self.center.z);
    let min = center - bvh::Vector3::splat(self.radius);
    let max = center + bvh::Vector3::splat(self.radius);
    AABB::with_bounds(min, max)
  }
}

pub struct TriangleMesh {
  pub points: Vec<glam::Vec3>,
  pub normals: Vec<glam::Vec3>,
  pub texcoords: Option<Vec<glam::Vec2>>,
  pub indices: Vec<u32>,
  pub tri_count: u32,
  object_to_world: glam::Affine3A,
  world_to_object: glam::Affine3A,
}
impl TriangleMesh {
  pub fn new(
    points: Vec<glam::Vec3>,
    normals: Vec<glam::Vec3>,
    texcoords: Option<Vec<glam::Vec2>>,
    indices: Vec<u32>,
    tri_count: u32,
    object_to_world: glam::Affine3A,
    world_to_object: glam::Affine3A,
  ) -> Self {
    Self {
      points,
      normals,
      texcoords,
      indices,
      tri_count,
      object_to_world,
      world_to_object,
    }
  }
}

pub struct Triangle {
  mesh: Arc<TriangleMesh>,
  pub id: u32,
}
impl Triangle {
  pub(super) fn new(mesh: Arc<TriangleMesh>, id: u32) -> Self {
    Self { mesh, id }
  }
  fn uvs(&self) -> [glam::Vec2; 3] {
    match &self.mesh.texcoords {
      Some(texcoords) => [
        texcoords[self.mesh.indices[(self.id * 3) as usize] as usize],
        texcoords[self.mesh.indices[(self.id * 3) as usize + 1] as usize],
        texcoords[self.mesh.indices[(self.id * 3) as usize + 2] as usize],
      ],
      None => [
        glam::Vec2::new(0.0, 0.0),
        glam::Vec2::new(1.0, 0.0),
        glam::Vec2::new(1.0, 1.0),
      ],
    }
  }
  fn points(&self) -> [glam::Vec3; 3] {
    [
      self.mesh.points[self.mesh.indices[(self.id * 3) as usize] as usize],
      self.mesh.points[self.mesh.indices[(self.id * 3) as usize + 1] as usize],
      self.mesh.points[self.mesh.indices[(self.id * 3) as usize + 2] as usize],
    ]
  }
  fn normals(&self) -> [glam::Vec3; 3] {
    [
      self.mesh.normals[self.mesh.indices[(self.id * 3) as usize] as usize],
      self.mesh.normals[self.mesh.indices[(self.id * 3) as usize + 1] as usize],
      self.mesh.normals[self.mesh.indices[(self.id * 3) as usize + 2] as usize],
    ]
  }
  fn intersect<'a>(&'a self, ray: &Ray, hit: &mut Hit<'a>) -> bool {
    let uvs = self.uvs();
    let [p0, p1, p2] = self.points().map(|p| glam::Vec3A::from(p));
    let [n0, n1, n2] = self.normals().map(|n| glam::Vec3A::from(n));

    // compute plane's normal
    let v0v1 = p1 - p0;
    let v0v2 = p2 - p0;
    // no need to normalize
    let ng = v0v1.cross(v0v2); // normal
    let area = ng.length() / 2.0;

    // Step 1: finding P

    // check if ray and plane are parallel ?
    let n_dot_ray = ng.dot(ray.direction);
    if n_dot_ray.abs() < 0.0001 {
      return false; //they are parallel so they don't intersect !
    }

    // compute d parameter using equation 2
    let d = -ng.dot(p0);

    // compute t (equation 3)
    let t = -(ng.dot(ray.origin) + d) / n_dot_ray;

    // check if the triangle is in behind the ray
    if t < ray.t_min || t > ray.t_max {
      return false; //the triangle is behind
    }

    // compute the intersection point using equation 1
    let p = ray.origin + t * ray.direction;

    // Step 2: inside-outside test

    // edge 0
    let edge0 = p1 - p0;
    let vp0 = p - p0;
    let c = edge0.cross(vp0);
    if ng.dot(c) < 0.0 {
      return false; // P is on the right side
    }

    // edge 1
    let edge1 = p2 - p1;
    let vp1 = p - p1;
    let c = edge1.cross(vp1);
    let u = (c.length() / 2.0) / area;
    if ng.dot(c) < 0.0 {
      return false; // P is on the right side
    }

    // edge 2
    let edge2 = p0 - p2;
    let vp2 = p - p2;
    let c = edge2.cross(vp2);
    let v = (c.length() / 2.0) / area;
    if ng.dot(c) < 0.0 {
      return false; // P is on the right side
    }

    let w = 1.0 - u - v;
    assert!(u >= 0.0 && u <= 1.0, "u={}, v={}, w={}", u, v, w);
    assert!(v >= 0.0 && v <= 1.0, "u={}, v={}, w={}", u, v, w);
    // assert!(w >= 0.0, "u={}, v={}, w={}", u, v, w);

    hit.p = p;
    hit.t = t.min(hit.t);
    hit.ng = ng;
    hit.ns = (n0 * u + n1 * v + n2 * w).normalize();
    hit.front = hit.ng.dot(-ray.direction) > 0.0;

    let dp1 = p1 - p0;
    let dp2 = p2 - p0;
    let duv1 = uvs[1] - uvs[0];
    let duv2 = uvs[2] - uvs[0];
    hit.uv = uvs[0] * u + uvs[1] * v + uvs[2] * w;
    let determinant = duv1.x * duv2.y - duv1.y * duv2.x;
    // Handle degenerate uv
    if determinant.abs() < 1e-8 {
      coordinate_system(&hit.ns, &mut hit.dpdu, &mut hit.dpdv);
    } else {
      let r = 1.0 / determinant;
      hit.dpdu = (dp1 * duv2.y - dp2 * duv1.y) * r;
      hit.dpdv = (dp2 * duv1.x - dp1 * duv2.x) * r;
    }

    true //this ray hits the triangle
  }
}
impl Bounded for Triangle {
  fn aabb(&self) -> bvh::aabb::AABB {
    let [p0, p1, p2] = self.points();
    let min = glam::Vec3::splat(f32::INFINITY).min(p0).min(p1).min(p2);
    let max = glam::Vec3::splat(-f32::INFINITY).max(p0).max(p1).max(p2);
    bvh::aabb::AABB::with_bounds(
      bvh::Vector3::new(min.x, min.y, min.z),
      bvh::Vector3::new(max.x, max.y, max.z),
    )
  }
}
