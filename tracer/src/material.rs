use utility::color::Color;
use crate::hittable::Hit;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, hit: &Hit, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        false
    }
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, hit: &Hit, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        let mut scatter_direction = hit.normal + Vec3::random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = hit.normal;
        }

        scattered.origin = hit.point;
        scattered.direction = scatter_direction;

        attenuation.r = self.albedo.r;
        attenuation.g = self.albedo.g;
        attenuation.b = self.albedo.b;

        true
    }
}

pub struct Metal {
    pub albedo: Color,
}

impl Metal {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, hit: &Hit, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        let reflected = Vec3::reflect(&r_in.direction, &hit.normal);

        scattered.origin = hit.point;
        scattered.direction = reflected;

        attenuation.r = self.albedo.r;
        attenuation.g = self.albedo.g;
        attenuation.b = self.albedo.b;

        true
    }
}
