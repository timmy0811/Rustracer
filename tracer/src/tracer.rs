use crate::camera::Camera;
use crate::hittable::Hit;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::vec3::Vec3;
use utility::color::{Color, LinearColor};
use utility::random::{degrees_to_radians, random_f64};

#[derive(Clone)]
pub struct Tracer<'a> {
    viewport_width: f64,
    viewport_height: f64,

    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    viewport_u: Vec3,
    viewport_v: Vec3,

    pixel_zero: Vec3,

    camera: Camera,
    scene: &'a Scene,

    pixel_samples_scale: f64,
    samples_per_pixel: u32,
    max_bounces: u32,
}

impl<'a> Tracer<'a> {
    pub fn new(
        image_width: u32,
        image_height: u32,
        samples_per_pixel: u32,
        max_bounces: u32,
        camera: Camera,
        scene: &'a Scene,
    ) -> Self {
        let aspect_ratio = image_width as f64 / image_height as f64;

        let theta = degrees_to_radians(camera.fov);
        let h = f64::tan(theta / 2.0);
        let viewport_height = 2.0 * h * camera.focus_dist;
        let viewport_width = viewport_height * aspect_ratio;

        let viewport_u = viewport_width * camera.u;
        let viewport_v = viewport_height * -camera.v;

        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let vp_upper_left =
            camera.position - camera.focus_dist * camera.w - viewport_u * 0.5 - viewport_v * 0.5;
        let pxl_0 = vp_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Tracer {
            viewport_width,
            viewport_height,
            pixel_delta_u,
            pixel_delta_v,
            viewport_u,
            viewport_v,
            pixel_zero: pxl_0,
            camera,
            scene,
            pixel_samples_scale: 1.0 / samples_per_pixel as f64,
            samples_per_pixel,
            max_bounces,
        }
    }

    pub fn raytrace_pixel(&self, x: u32, y: u32, interval: &Interval) -> Color {
        let mut pixel_color = Vec3::zero();
        for _ in 0..self.samples_per_pixel {
            let ray = self.gen_ray(x, y);
            pixel_color += self.trace_ray(ray, interval, self.max_bounces);
        }

        (pixel_color * self.pixel_samples_scale)
            .as_linear_color()
            .gamma_correct()
            .to_color()
    }

    fn sample_square() -> Vec3 {
        Vec3(random_f64() - 0.5, random_f64() - 0.5, 0.0)
    }

    fn defocus_disk_sample(&self) -> Vec3 {
        let p = Vec3::random_in_unit_disk();
        self.camera.position + p.0 * self.camera.defocus_disk_u + p.1 * self.camera.defocus_disk_v
    }

    fn gen_ray(&self, x: u32, y: u32) -> Ray {
        let offset = Self::sample_square();
        let pixel_sample = self.pixel_zero
            + ((x as f64 + offset.0) * self.pixel_delta_u)
            + ((y as f64 + offset.1) * self.pixel_delta_v);

        let origin = if self.camera.defocus_angle <= 0.0 {
            self.camera.position
        } else {
            self.defocus_disk_sample()
        };

        Ray {
            origin,
            direction: pixel_sample - origin,
        }
    }

    fn trace_ray(&self, ray: Ray, interval: &Interval, depth: u32) -> Vec3 {
        if depth == 0 {
            return Vec3::zero();
        }

        let mut hr = Hit::default();
        let mut closest_so_far = interval.max;
        let mut hit_anything = false;

        for obj in self.scene {
            if obj.hit(&ray, &Interval::new(interval.min, closest_so_far), &mut hr) {
                hit_anything = true;
                closest_so_far = hr.t;
            }
        }

        if hit_anything {
            let mut scattered = Ray::new();
            let mut attenuation = LinearColor::black();

            if let Some(material) = hr.material.as_ref() {
                if material.scatter(&ray, &hr, &mut attenuation, &mut scattered) {
                    return self.trace_ray(scattered, interval, depth - 1) * attenuation;
                }
            }

            return Vec3::zero();
        }

        let unit_vec = ray.direction.normalized();
        let a = 0.5 * (unit_vec.1 + 1.0);

        (1.0 - a) * Vec3(1.0, 1.0, 1.0) + a * Vec3(0.5, 0.7, 1.0)
    }
}
