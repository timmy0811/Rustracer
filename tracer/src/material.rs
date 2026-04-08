use crate::hittable::Hit;
use crate::ray::Ray;
use crate::vec3::Vec3;
use utility::color::LinearColor;
use utility::random::random_f64;

pub trait Material: Send + Sync {
    fn scatter(
        &self,
        r_in: &Ray,
        hit: &Hit,
        attenuation: &mut LinearColor,
        scattered: &mut Ray,
    ) -> bool {
        false
    }
}

pub struct Lambertian {
    pub albedo: LinearColor,
}

impl Lambertian {
    pub fn new<T: Into<LinearColor>>(albedo: T) -> Self {
        Self {
            albedo: albedo.into(),
        }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        r_in: &Ray,
        hit: &Hit,
        attenuation: &mut LinearColor,
        scattered: &mut Ray,
    ) -> bool {
        let mut scatter_direction = hit.normal + Vec3::random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = hit.normal;
        }

        scattered.origin = hit.point;
        scattered.direction = scatter_direction;

        *attenuation = self.albedo;

        true
    }
}

pub struct Metal {
    pub albedo: LinearColor,
    pub fuzz: f64,
}

impl Metal {
    pub fn new<T: Into<LinearColor>>(albedo: T, fuzz: f64) -> Self {
        Self {
            albedo: albedo.into(),
            fuzz,
        }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        r_in: &Ray,
        hit: &Hit,
        attenuation: &mut LinearColor,
        scattered: &mut Ray,
    ) -> bool {
        let mut reflected = Vec3::reflect(&r_in.direction, &hit.normal);
        reflected = reflected.normalized() + (self.fuzz * Vec3::random_unit_vector());

        scattered.origin = hit.point;
        scattered.direction = reflected;

        *attenuation = self.albedo;

        scattered.direction.dot(&hit.normal) > 0.0
    }
}

pub struct Dielectric {
    pub index_of_refraction: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Self {
        Self {
            index_of_refraction,
        }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 *= r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        r_in: &Ray,
        hit: &Hit,
        attenuation: &mut LinearColor,
        scattered: &mut Ray,
    ) -> bool {
        *attenuation = LinearColor::white();

        let unit_direction = r_in.direction.normalized();
        let ri = if hit.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let cos_theta = (-unit_direction).dot(&hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction = if cannot_refract || Self::reflectance(cos_theta, ri) > random_f64() {
            Vec3::reflect(&unit_direction, &hit.normal)
        } else {
            Vec3::refract(&unit_direction, &hit.normal, ri)
        };

        scattered.origin = hit.point;
        scattered.direction = direction;

        true
    }
}
