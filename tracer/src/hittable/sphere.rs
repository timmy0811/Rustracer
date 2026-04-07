use crate::hittable::{Hit, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Sphere {
    center: Vec3,
    radius: f64,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64) -> Self {
        Sphere{
            center,
            radius
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, interval: &Interval, hit_record: &mut Hit) -> bool {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return false;
        }

        let sqrt = discriminant.sqrt();
        let mut root = (h - sqrt) / a;
        if root <= interval.min || interval.max <= root {
            root = (h + sqrt) / a;
            if root <= interval.min || interval.max <= root {
                return false;
            }
        }

        hit_record.t = root;
        hit_record.point = ray.at(root);
        hit_record.normal = (hit_record.point - self.center) / self.radius;

        let outward = (hit_record.point - self.center) / self.radius;
        hit_record.set_face_normal(ray, &outward);

        true
    }
}
