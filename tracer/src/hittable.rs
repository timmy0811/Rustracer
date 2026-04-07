pub mod sphere;

use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, interval: &Interval, hit_record: &mut Hit) -> bool;
}

pub struct Hit{
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f64,
    front_face: bool
}

impl Hit {
    pub fn default() -> Self {
        Self{
            point: Vec3(0.0, 0.0, 0.0),
            normal: Vec3(0.0, 0.0, 0.0),
            t: 0.0,
            front_face: true
        }
    }

    fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3){
        let v = outward_normal.normalized();
        self.front_face = ray.direction.dot(*outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal.clone()
        }else {
            outward_normal.clone() * -1.0
        }
    }
}