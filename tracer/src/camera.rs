use crate::vec3::Vec3;
use utility::random::degrees_to_radians;

#[derive(Clone)]
pub struct Camera {
    pub focal_length: f64,
    pub position: Vec3,
    pub fov: f64,
    pub lookfrom: Vec3,
    pub lookat: Vec3,
    pub up: Vec3,

    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,

    pub defocus_angle: f64,
    pub focus_dist: f64,

    pub defocus_disk_u: Vec3,
    pub defocus_disk_v: Vec3,
}

impl Camera {
    pub fn default() -> Self {
        Self {
            focal_length: 1.0,
            position: Vec3(0.0, 0.0, 0.0),
            fov: 130.0,
            lookfrom: Vec3(0.0, 0.0, 0.0),
            lookat: Vec3(0.0, 0.0, -1.0),
            up: Vec3(0.0, 1.0, 0.0),

            u: Vec3(0.0, 0.0, 0.0),
            v: Vec3(0.0, 0.0, 0.0),
            w: Vec3(0.0, 0.0, 0.0),

            defocus_angle: 6.0,
            focus_dist: 3.5,

            defocus_disk_u: Vec3::zero(),
            defocus_disk_v: Vec3::zero(),
        }
    }

    pub fn init(&mut self) {
        self.position = self.lookfrom;

        self.w = (self.lookfrom - self.lookat).normalized();
        self.u = self.up.cross(&self.w).normalized();
        self.v = self.w.cross(&self.u);

        let defocus_radius =
            self.focus_dist * f64::tan(degrees_to_radians(self.defocus_angle) / 2.0);
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }
}
