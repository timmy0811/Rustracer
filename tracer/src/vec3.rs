use std::ops::*;
use utility::color::{Color, LinearColor};
use utility::random;
use utility::random::{random_f64, random_f64_limit};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3(pub f64, pub f64, pub f64);

impl Vec3 {
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
    pub fn z(&self) -> f64 {
        self.2
    }

    pub fn zero() -> Self {
        Self(0.0, 0.0, 0.0)
    }

    pub fn dot(self, rhs: &Self) -> f64 {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }

    pub fn cross(self, rhs: &Self) -> Self {
        Self(
            self.1 * rhs.2 - self.2 * rhs.1,
            self.2 * rhs.0 - self.0 * rhs.2,
            self.0 * rhs.1 - self.1 * rhs.0,
        )
    }

    pub fn length_squared(self) -> f64 {
        self.dot(&self)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            *self / len
        }
    }

    pub fn near_zero(&self) -> bool {
        const S: f64 = 1e-8;
        self.0.abs() < S && self.1.abs() < S && self.2.abs() < S
    }

    pub fn as_linear_color(self) -> LinearColor {
        LinearColor::rgb_unclamped(self.0 as f32, self.1 as f32, self.2 as f32)
    }

    pub fn as_color(self) -> Color {
        self.as_linear_color().to_color()
    }

    pub fn random() -> Self {
        Vec3(random_f64(), random_f64(), random_f64())
    }

    pub fn random_clamped(min: f64, max: f64) -> Self {
        Vec3(
            random_f64_limit(min, max),
            random_f64_limit(min, max),
            random_f64_limit(min, max),
        )
    }

    pub fn random_unit_vector() -> Self {
        while true {
            let p = Self::random_clamped(-1.0, 1.0);
            let lensq = p.length_squared();
            if 1e-160 < lensq && lensq <= 1.0 {
                return p / lensq.sqrt();
            }
        }

        Vec3::zero()
    }

    pub fn random_on_hemisphere(normal: &Vec3) -> Self {
        let on_unit_sphere = Self::random_unit_vector();
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
        *v - 2.0 * v.dot(&n) * *n
    }

    pub fn refract(v: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
        let cos_theta = (-*v).dot(&n).min(1.0);
        let r_out_perp = etai_over_etat * (*v + cos_theta * *n);
        let r_out_parallel = -((1.0 - r_out_perp.length_squared()).max(0.0).sqrt()) * *n;

        r_out_perp + r_out_parallel
    }

    pub fn random_in_unit_disk() -> Self {
        while true {
            let p = Self(
                random::random_f64_limit(-1.0, 1.0),
                random::random_f64_limit(-1.0, 1.0),
                0.0,
            );
            if p.length_squared() < 1.0 {
                return p;
            }
        }

        Vec3::zero()
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Add<f64> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: f64) -> Vec3 {
        Vec3(self.0 + rhs, self.1 + rhs, self.2 + rhs)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3(-self.0, -self.1, -self.2)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Vec3 {
        Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Mul<Color> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Color) -> Vec3 {
        self * rhs.to_linear()
    }
}

impl Mul<LinearColor> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: LinearColor) -> Vec3 {
        Vec3(
            self.0 * rhs.r as f64,
            self.1 * rhs.g as f64,
            self.2 * rhs.b as f64,
        )
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Vec3 {
        Vec3(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

// Optional: allows 2.0 * v
impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}
