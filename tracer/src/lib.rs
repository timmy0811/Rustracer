extern crate core;

mod worker;
pub mod tracer;
pub mod vec3;
mod ray;
pub mod camera;
pub mod hittable;
pub mod scene;
mod interval;

pub use worker::{raycast_scene_parallel, render_packages_in_parallel_atomic, RenderError, ThreadWorkPackage};
