use crate::vec3::Vec3;

pub struct Ray{
    pub origin: Vec3,
    pub direction: Vec3
}

impl Ray{
    pub fn new() -> Self{
        Self{
            origin: Vec3(0.0, 0.0, 0.0),
            direction: Vec3(0.0, 0.0, 1.0)
        }
    }
    
    pub fn at(&self, t: f64) -> Vec3{
        self.origin + t * self.direction
    }
}