use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicU8, Ordering};
use rand::RngExt;
use crate::interval::Interval;
use crate::scene::Scene;
use crate::tracer::{Tracer};

const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug)]
pub enum RenderError {
    InvalidBufferLength { expected: usize, actual: usize },
    EmptyDimensions,
}

pub struct ThreadWorkPackage<'a> {
    pub scene: &'a Scene,
    pub data_region: &'a [AtomicU8],
    pub start_pixel: u32,
    pub end_pixel: u32,
    pub tracer: Tracer<'a>
}

impl<'a> ThreadWorkPackage<'a> {
    fn render(self, width: u32, height: u32) {
        let mut rng = rand::rng();

        let n = (self.end_pixel - self.start_pixel) as usize;
        if n == 0 {
            return;
        }

        let offset = (rng.random::<u64>() as usize) % n;

        // Pick a random step that is coprime with n.
        let mut step = (n as f32 * 0.453).max(1.0) as usize;
        while gcd(step, n) != 1 {
            step = (step + 1) % n;
            if step == 0 {
                step = 1;
            }
        }

        let width_r = 1.0 / width as f32;
        let height_r = 1.0 / width as f32;
        
        let clipping_interval = Interval::new(0.001, 9999999.0);

        for k in 0..n {
            let local_pixel = (offset + k * step) % n;
            let global_pixel_index = self.start_pixel as usize + local_pixel;
            let x = (global_pixel_index % width as usize) as u32;
            let y = (global_pixel_index / width as usize) as u32;
            let rand_val: u32 = rng.random();

            let color = self.tracer.raytrace_pixel(x, y, &clipping_interval);

            let base = local_pixel * BYTES_PER_PIXEL;
            self.data_region[base + 0].store(color.r, Ordering::Relaxed);
            self.data_region[base + 1].store(color.g, Ordering::Relaxed);
            self.data_region[base + 2].store(color.b, Ordering::Relaxed);
            self.data_region[base + 3].store(color.a, Ordering::Relaxed);
        }
    }
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

pub fn render_packages_in_parallel_atomic(
    scene: &Scene,
    buffer: &[AtomicU8],
    width: u32,
    height: u32,
    threads: usize,
    tracer: Tracer) -> Result<(), RenderError>
{
    if width == 0 || height == 0 {
        return Err(RenderError::EmptyDimensions);
    }

    let total_pixels = width as usize * height as usize;
    let expected_len = total_pixels * BYTES_PER_PIXEL;
    if buffer.len() != expected_len {
        return Err(RenderError::InvalidBufferLength {
            expected: expected_len,
            actual: buffer.len(),
        });
    }

    let mut packages = split_into_work_packages_atomic(scene, buffer, width, height, threads.max(1), tracer);

    thread::scope(|scope| {
        for package in packages.drain(..) {
            scope.spawn(move || {
                package.render(width, height);
            });
        }
    });

    Ok(())
}

pub fn raycast_scene_parallel(
    scene: &Scene,
    buffer: &[AtomicU8],
    width: u32,
    height: u32,
    threads: usize,
    tracer: Tracer
) -> Result<(), RenderError> {
    render_packages_in_parallel_atomic(scene, buffer, width, height, threads, tracer)
}

fn split_into_work_packages_atomic<'a>(
    scene: &'a Scene,
    buffer: &'a [AtomicU8],
    width: u32,
    height: u32,
    threads: usize,
    tracer: Tracer<'a>
) -> Vec<ThreadWorkPackage<'a>> {
    let worker_count = threads.min(height as usize).max(1);
    let height_usize = height as usize;
    let rows_per_worker = height_usize / worker_count;
    let extra_rows = height_usize % worker_count;

    let mut packages = Vec::with_capacity(worker_count);
    let mut remaining = buffer;
    let mut start_pixel = 0_u32;

    for worker in 0..worker_count {
        let rows = rows_per_worker + usize::from(worker < extra_rows);
        let pixels = rows * width as usize;
        let bytes = pixels * BYTES_PER_PIXEL;
        
        let (region, rest) = remaining.split_at(bytes);
        remaining = rest;

        let end_pixel = start_pixel + pixels as u32;
        packages.push(ThreadWorkPackage {
            scene,
            data_region: region,
            start_pixel,
            end_pixel,
            tracer: tracer.clone()
        });
        start_pixel = end_pixel;
    }

    packages
}