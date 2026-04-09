use std::cell::RefCell;
use rand::RngExt;

thread_local! {
    static RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::rng());
}

pub fn random_f64() -> f64 {
    RNG.with(|rng| rng.borrow_mut().random())
}

pub fn random_f32() -> f32 {
    RNG.with(|rng| rng.borrow_mut().random())
}
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

pub fn random_f64_limit(min: f64, max: f64) -> f64 {
    min + (max - min) * random_f64()
}

pub fn random_f32_limit(min: f32, max: f32) -> f32 {
    min + (max - min) * random_f32()
}
