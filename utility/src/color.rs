use crate::random;
use rand::RngExt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LinearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl LinearColor {
    pub fn x(&self) -> f32 {
        self.r
    }
    pub fn y(&self) -> f32 {
        self.g
    }
    pub fn z(&self) -> f32 {
        self.b
    }
    pub fn w(&self) -> f32 {
        self.a
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();
        Self {
            r: rng.random(),
            g: rng.random(),
            b: rng.random(),
            a: 1.0,
        }
    }

    pub fn random_limit(min: f32, max: f32) -> Self {
        Self {
            r: random::random_f32_limit(min, max),
            g: random::random_f32_limit(min, max),
            b: random::random_f32_limit(min, max),
            a: 1.0,
        }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    pub fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }

    fn linear_to_gamma(linear_component: f32) -> f32 {
        if linear_component > 0.0 {
            return linear_component.sqrt();
        }

        0.0
    }

    pub fn gamma_correct(mut self) -> Self {
        self.r = Self::linear_to_gamma(self.r);
        self.g = Self::linear_to_gamma(self.g);
        self.b = Self::linear_to_gamma(self.b);
        self
    }

    pub fn to_color(self) -> Color {
        Color::rgba_f(self.r, self.g, self.b, self.a)
    }
}

impl Add for LinearColor {
    type Output = LinearColor;

    fn add(self, rhs: LinearColor) -> LinearColor {
        LinearColor::rgba(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Sub for LinearColor {
    type Output = LinearColor;

    fn sub(self, rhs: LinearColor) -> LinearColor {
        LinearColor::rgba(
            self.r - rhs.r,
            self.g - rhs.g,
            self.b - rhs.b,
            self.a - rhs.a,
        )
    }
}

impl Mul<f32> for LinearColor {
    type Output = LinearColor;

    fn mul(self, rhs: f32) -> LinearColor {
        LinearColor::rgba(self.r * rhs, self.g * rhs, self.b * rhs, self.a * rhs)
    }
}

impl Div<f32> for LinearColor {
    type Output = LinearColor;

    fn div(self, rhs: f32) -> LinearColor {
        LinearColor::rgba(self.r / rhs, self.g / rhs, self.b / rhs, self.a / rhs)
    }
}

impl AddAssign for LinearColor {
    fn add_assign(&mut self, rhs: LinearColor) {
        self.r = (self.r + rhs.r).clamp(0.0, 1.0);
        self.g = (self.g + rhs.g).clamp(0.0, 1.0);
        self.b = (self.b + rhs.b).clamp(0.0, 1.0);
        self.a = (self.a + rhs.a).clamp(0.0, 1.0);
    }
}

impl Color {
    pub fn x(&self) -> u8 {
        self.r
    }
    pub fn y(&self) -> u8 {
        self.g
    }
    pub fn z(&self) -> u8 {
        self.b
    }
    pub fn w(&self) -> u8 {
        self.a
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub fn rgba_f(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (255.0 * r.clamp(0.0, 1.0)) as u8,
            g: (255.0 * g.clamp(0.0, 1.0)) as u8,
            b: (255.0 * b.clamp(0.0, 1.0)) as u8,
            a: (255.0 * a.clamp(0.0, 1.0)) as u8,
        }
    }

    pub fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub fn rgb_f(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: (255.0 * r.clamp(0.0, 1.0)) as u8,
            g: (255.0 * g.clamp(0.0, 1.0)) as u8,
            b: (255.0 * b.clamp(0.0, 1.0)) as u8,
            a: 255,
        }
    }

    fn channel_to_f32(channel: u8) -> f32 {
        channel as f32 / 255.0
    }

    pub fn to_linear(self) -> LinearColor {
        LinearColor::rgba(
            Self::channel_to_f32(self.r),
            Self::channel_to_f32(self.g),
            Self::channel_to_f32(self.b),
            Self::channel_to_f32(self.a),
        )
    }

    pub fn gamma_correct(self) -> Self {
        self.to_linear().gamma_correct().to_color()
    }
}

impl From<Color> for LinearColor {
    fn from(value: Color) -> Self {
        value.to_linear()
    }
}

impl From<LinearColor> for Color {
    fn from(value: LinearColor) -> Self {
        value.to_color()
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color::rgba(
            self.r.saturating_add(rhs.r),
            self.g.saturating_add(rhs.g),
            self.b.saturating_add(rhs.b),
            self.a.saturating_add(rhs.a),
        )
    }
}

impl Add<u8> for Color {
    type Output = Color;

    fn add(self, rhs: u8) -> Color {
        Color::rgba(
            self.r.saturating_add(rhs),
            self.g.saturating_add(rhs),
            self.b.saturating_add(rhs),
            self.a.saturating_add(rhs),
        )
    }
}

impl Sub for Color {
    type Output = Color;

    fn sub(self, rhs: Color) -> Color {
        Color::rgba(
            self.r.saturating_sub(rhs.r),
            self.g.saturating_sub(rhs.g),
            self.b.saturating_sub(rhs.b),
            self.a.saturating_sub(rhs.a),
        )
    }
}

impl Neg for Color {
    type Output = Color;

    fn neg(self) -> Color {
        // Unary minus for colors behaves like channel inversion.
        Color::rgba(255 - self.r, 255 - self.g, 255 - self.b, 255 - self.a)
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Color {
        (self.to_linear() * rhs).to_color()
    }
}

impl Div<f32> for Color {
    type Output = Color;

    fn div(self, rhs: f32) -> Color {
        (self.to_linear() / rhs).to_color()
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r = self.r.saturating_add(rhs.r);
        self.g = self.g.saturating_add(rhs.g);
        self.b = self.b.saturating_add(rhs.b);
        self.a = self.a.saturating_add(rhs.a);
    }
}

impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Color {
        (rhs.to_linear() * self).to_color()
    }
}
