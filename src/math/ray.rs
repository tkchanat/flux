use glam::Vec3A;

#[derive(Clone, Debug)]
pub struct Ray {
  pub origin: Vec3A,
  pub direction: Vec3A,
  pub t_min: f32,
  pub t_max: f32,
}

impl Ray {
  pub fn new(origin: Vec3A, direction: Vec3A) -> Self {
    Self {
      origin,
      direction,
      t_min: 0.0,
      t_max: f32::INFINITY,
    }
  }
}

impl Into<bvh::ray::Ray> for Ray {
  fn into(self) -> bvh::ray::Ray {
    bvh::ray::Ray::new(
      bvh::Point3::new(self.origin.x, self.origin.y, self.origin.z),
      bvh::Vector3::new(self.direction.x, self.direction.y, self.direction.z),
    )
  }
}

pub fn transform_ray(transform: &glam::Affine3A, ray: &Ray) -> Ray {
  let mut ray = ray.clone();
  ray.origin = transform.transform_point3a(ray.origin);
  ray.direction = transform.transform_vector3a(ray.direction);
  ray
}
