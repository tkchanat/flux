use super::hit::Hit;
use crate::math::{coordinate_system, Ray};
use bvh::aabb::{Bounded, AABB};
use glam::{Mat3A, Vec2, Vec3, Vec3A};
use std::f32::consts::PI;

pub trait Shape: bvh::aabb::Bounded + Sync + Send {
  fn intersect<'a>(&'a self, ray: &Ray, hit: &mut Hit<'a>) -> bool;
}

pub struct Sphere {
  center: Vec3,
  radius: f32,
}

impl Sphere {
  pub fn new(center: Vec3, radius: f32) -> Self {
    Self { center, radius }
  }
}

impl Shape for Sphere {
  fn intersect<'a>(&'a self, ray: &Ray, hit: &mut Hit<'a>) -> bool {
    let center = Vec3A::from(self.center);

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

    hit.shape = Some(self);
    hit.p = ray.origin + t * ray.direction;
    hit.t = t.min(hit.t);
    hit.ng = (hit.p - center).normalize();
    hit.ns = hit.ng;

    let theta = (-hit.ns.y).acos();
    let phi = (-hit.ns.z).atan2(hit.ns.x) + PI;
    hit.uv = Vec2::new(phi / (2.0 * PI), theta / PI);
    hit.dpdu = Vec3A::new(-theta.sin(), 0.0, phi.cos());
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

#[derive(Clone)]
pub struct Triangle {
  pub vertices: [Vec3; 3],
  pub normals: [Vec3; 3],
  pub texcoords: Option<[Vec2; 3]>,
}

impl Shape for Triangle {
  fn intersect<'a>(&'a self, ray: &Ray, hit: &mut Hit<'a>) -> bool {
    let p0 = Vec3A::from(self.vertices[0]);
    let p1 = Vec3A::from(self.vertices[1]);
    let p2 = Vec3A::from(self.vertices[2]);
    let n0 = Vec3A::from(self.normals[0]);
    let n1 = Vec3A::from(self.normals[1]);
    let n2 = Vec3A::from(self.normals[2]);

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

    hit.shape = Some(self);
    hit.p = p;
    hit.t = t.min(hit.t);
    hit.ng = ng;
    hit.ns = (n0 * u + n1 * v + n2 * w).normalize();
    hit.front = hit.ng.dot(-ray.direction) > 0.0;

    let texcoords = match self.texcoords {
      Some(texcoords) => texcoords,
      None => [
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
      ],
    };

    let dp1 = p1 - p0;
    let dp2 = p2 - p0;
    let duv1 = texcoords[1] - texcoords[0];
    let duv2 = texcoords[2] - texcoords[0];
    hit.uv = texcoords[0] * u + texcoords[1] * v + texcoords[2] * w;
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
    let min = self
      .vertices
      .iter()
      .fold(Vec3::splat(f32::INFINITY), |acc, x| acc.min(*x));
    let max = self
      .vertices
      .iter()
      .fold(Vec3::splat(-f32::INFINITY), |acc, x| acc.max(*x));
    AABB::with_bounds(
      bvh::Vector3::new(min.x, min.y, min.z),
      bvh::Vector3::new(max.x, max.y, max.z),
    )
  }
}
