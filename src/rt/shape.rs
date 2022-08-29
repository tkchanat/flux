use super::hit::Hit;
use crate::math::Ray;
use bvh::aabb::{Bounded, AABB};
use glam::{Vec3, Vec3A};

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
  fn intersect(&self, ray: &Ray, hit: &mut Hit) -> bool {
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

    hit.t = t.min(hit.t);
    hit.ng = ((ray.origin + ray.direction * t) - center).normalize();
    hit.front = hit.ng.dot(ray.direction) > 0.0;
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
  vertices: [Vec3; 3],
}

impl Triangle {
  pub fn new(p0: Vec3, p1: Vec3, p2: Vec3) -> Self {
    Self {
      vertices: [p0, p1, p2],
    }
  }
}

impl Shape for Triangle {
  fn intersect<'a>(&'a self, ray: &Ray, hit: &mut Hit<'a>) -> bool {
    let p0 = Vec3A::from(self.vertices[0]);
    let p1 = Vec3A::from(self.vertices[1]);
    let p2 = Vec3A::from(self.vertices[2]);

    // compute plane's normal
    let v0v1 = p1 - p0;
    let v0v2 = p2 - p0;
    // no need to normalize
    let n = v0v1.cross(v0v2); // normal

    // Step 1: finding P

    // check if ray and plane are parallel ?
    let n_dot_ray = n.dot(ray.direction);
    if n_dot_ray.abs() < 0.0001 {
      return false; //they are parallel so they don't intersect !
    }

    // compute d parameter using equation 2
    let d = -n.dot(p0);

    // compute t (equation 3)
    let t = -(n.dot(ray.origin) + d) / n_dot_ray;

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
    if n.dot(c) < 0.0 {
      return false; //P is on the right side
    }

    // edge 1
    let edge1 = p2 - p1;
    let vp1 = p - p1;
    let c = edge1.cross(vp1);
    if n.dot(c) < 0.0 {
      return false; //P is on the right side
    }

    // edge 2
    let edge2 = p0 - p2;
    let vp2 = p - p2;
    let c = edge2.cross(vp2);
    if n.dot(c) < 0.0 {
      return false; //P is on the right side;
    }

    hit.shape = Some(self);
    hit.p = p;
    hit.t = t.min(hit.t);
    hit.ng = n.normalize();
    hit.ns = hit.ng;
    hit.front = hit.ng.dot(-ray.direction) > 0.0;
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
