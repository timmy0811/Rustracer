use crate::vec3::Vec3;

#[derive(Clone)]
pub struct Camera{
    pub focal_length: f64,
    pub position: Vec3,
}

impl Camera{
    pub fn default() -> Self {
        Self{
            focal_length: 1.0,
            position: Vec3(0.0, 0.0, 0.0),
        }
    }
}