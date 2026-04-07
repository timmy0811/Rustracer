use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Color{
    pub fn x(&self) -> u8 {self.r}
    pub fn y(&self) -> u8 {self.g}
    pub fn z(&self) -> u8 {self.b}
    pub fn w(&self) -> u8 {self.a}

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self{r, g, b, a}
    }
    pub fn rgba_f(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self{
            r : (255.0 * r.clamp(0.0, 1.0)) as u8,
            g : (255.0 * g.clamp(0.0, 1.0)) as u8,
            b : (255.0 * b.clamp(0.0, 1.0)) as u8,
            a : (255.0 * a.clamp(0.0, 1.0)) as u8
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self{r, g, b, a: 255}
    }
    pub fn rgb_f(r: f32, g: f32, b: f32) -> Self {
        Self{
            r : (255.0 * r.clamp(0.0, 1.0)) as u8,
            g : (255.0 * g.clamp(0.0, 1.0)) as u8,
            b : (255.0 * b.clamp(0.0, 1.0)) as u8,
            a: 255
        }
    }

    fn channel_to_f32(channel: u8) -> f32 {
        channel as f32 / 255.0
    }

    fn channel_from_f32(channel: f32) -> u8 {
        (255.0 * channel.clamp(0.0, 1.0)) as u8
    }
    
    fn linear_to_gamma(linear_component: f64) -> f64{
        if linear_component > 0.0 {
            return linear_component.sqrt()
        }
        
        0.0
    }

    pub fn gamma_correct(mut self) -> Self {
        let r = Self::channel_to_f32(self.r);
        let g = Self::channel_to_f32(self.g);
        let b = Self::channel_to_f32(self.b);

        self.r = Self::channel_from_f32(Self::linear_to_gamma(r as f64) as f32);
        self.g = Self::channel_from_f32(Self::linear_to_gamma(g as f64) as f32);
        self.b = Self::channel_from_f32(Self::linear_to_gamma(b as f64) as f32);

        self
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
        Color::rgba_f(
            Self::channel_to_f32(self.r) * rhs,
            Self::channel_to_f32(self.g) * rhs,
            Self::channel_to_f32(self.b) * rhs,
            Self::channel_to_f32(self.a) * rhs,
        )
    }
}

impl Div<f32> for Color {
    type Output = Color;

    fn div(self, rhs: f32) -> Color {
        Color::rgba_f(
            Self::channel_to_f32(self.r) / rhs,
            Self::channel_to_f32(self.g) / rhs,
            Self::channel_to_f32(self.b) / rhs,
            Self::channel_to_f32(self.a) / rhs,
        )
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
        Color::rgba(
            Color::channel_from_f32(self * Color::channel_to_f32(rhs.r)),
            Color::channel_from_f32(self * Color::channel_to_f32(rhs.g)),
            Color::channel_from_f32(self * Color::channel_to_f32(rhs.b)),
            Color::channel_from_f32(self * Color::channel_to_f32(rhs.a)),
        )
    }
}
