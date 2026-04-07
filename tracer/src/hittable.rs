pub mod sphere;

use std::sync::Arc;
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub type SharedMaterial = Arc<dyn Material + Send + Sync>;

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, interval: &Interval, hit_record: &mut Hit) -> bool;
}

pub struct Hit{
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub material: Option<SharedMaterial>,
}

impl Hit {
    pub fn default() -> Self {
        Self{
            point: Vec3(0.0, 0.0, 0.0),
            normal: Vec3(0.0, 0.0, 0.0),
            t: 0.0,
            front_face: true,
            material: None,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3){
        self.front_face = ray.direction.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        }else {
            *outward_normal * -1.0
        }
    }
}