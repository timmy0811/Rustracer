use crate::framebuffer::Framebuffer;
use crate::gpu::GpuState;
use log::set_max_level;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracer::camera::Camera;
use tracer::hittable::sphere::Sphere;
use tracer::material::{Dielectric, Lambertian, Material, Metal};
use tracer::scene::Scene;
use tracer::tracer::Tracer;
use tracer::vec3::Vec3;
use tracer::{RenderError, raycast_scene_parallel};
use utility::color::{Color, LinearColor};
use utility::config::{AppConfig, ConfigError, DEFAULT_CONFIG_PATH};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use utility::random::random_f64;

pub struct App {
    window: Option<Arc<Window>>,
    g_context: Option<GraphicsContext>,
    g_data: GraphicsData,
    timestep: f32,
    config: AppConfig,
    update_scene: bool,
    shared_render_buffer: Arc<Vec<AtomicU8>>,
    worker_job: Option<JoinHandle<Result<(), RenderError>>>,
    upload_interval: Duration,
    last_upload: Instant,

    camera: Camera,
    scene: Arc<Scene>,

    start_duration: Duration,
}

impl App {
    pub fn new() -> Result<Self, ConfigError> {
        env_logger::init();

        log::info!("Welcome to Rustracer!");
        log::info!("Initializing App...");

        let config = AppConfig::load_from_file(DEFAULT_CONFIG_PATH)?;

        let mut framebuffer = Framebuffer::new(config.width, config.height, 4);
        framebuffer.fill(Color::rgb(20, 30, 200));
        let shared_render_buffer = Arc::new(
            (0..framebuffer.data.len())
                .map(|_| AtomicU8::new(0))
                .collect::<Vec<_>>(),
        );

        let data = GraphicsData { framebuffer };

        let frametime = config.frame_time;

        let mut camera = Camera::default();

        camera.lookfrom = Vec3(13.0, 2.0, 3.0);
        camera.lookat = Vec3(0.0, 0.0, 0.0);
        camera.up = Vec3(0.0, 1.0, 0.0);
        camera.fov = 20.0;

        camera.defocus_angle = 0.6;
        camera.focus_dist = 10.0;

        camera.init();

        Ok(Self {
            window: None,
            g_context: None,
            g_data: data,
            timestep: 1.0 / 16.0,
            config,
            update_scene: false,
            shared_render_buffer,
            worker_job: None,
            upload_interval: Duration::from_millis(frametime),
            last_upload: Instant::now() - Duration::from_millis(frametime),
            camera,
            scene: Arc::new(Scene::new(32)),
            start_duration: Duration::ZERO,
        })
    }

    pub fn run(self, event_loop: EventLoop<()>) -> Result<(), winit::error::EventLoopError> {
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = self;
        event_loop.run_app(&mut app)
    }

    pub fn update(&mut self) {
        self.poll_worker_completion();

        if self.update_scene && self.worker_job.is_none() {
            self.spawn_worker_batch();
        }
    }

    pub fn render(&mut self) {
        if self.last_upload.elapsed() < self.upload_interval {
            return;
        }

        self.sync_framebuffer_from_shared();

        if let Some(context) = self.g_context.as_mut() {
            context.state.upload_buffer(&self.g_data.framebuffer);
            context.state.render();
            self.last_upload = Instant::now();
        }
    }

    fn spawn_worker_batch(&mut self) {
        let width = self.g_data.framebuffer.width;
        let height = self.g_data.framebuffer.height;
        let threads = self.config.threads as usize;
        let shared = Arc::clone(&self.shared_render_buffer);
        let scene = Arc::clone(&self.scene);
        let camera = self.camera.clone();
        let samples_per_pixel = self.config.samples_per_pixel;
        let max_bounces = self.config.max_bounces;

        // Show unfinished work as black until the worker batch finishes.
        self.g_data.framebuffer.data.fill(0);
        for byte in shared.iter() {
            byte.store(0, Ordering::Relaxed);
        }

        self.start_duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        self.worker_job = Some(std::thread::spawn(move || {
            let scene_ref = scene.as_ref();
            let tracer = Tracer::new(
                width,
                height,
                samples_per_pixel,
                max_bounces,
                camera,
                scene_ref,
            );

            raycast_scene_parallel(scene_ref, shared.as_slice(), width, height, threads, tracer)
        }));
    }

    fn poll_worker_completion(&mut self) {
        let Some(worker_job) = self.worker_job.as_ref() else {
            return;
        };

        if !worker_job.is_finished() {
            return;
        }

        let worker_job = self.worker_job.take().expect("worker job must exist");
        match worker_job.join() {
            Ok(Ok(())) => {
                self.update_scene = false;
                let frame_time =
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - self.start_duration;
                log::error!("frame rendered in {frame_time:?}");
            }
            Ok(Err(err)) => {
                log::error!("render error: {err:?}");
                self.update_scene = false;
            }
            Err(_) => {
                log::error!("worker batch panicked");
                self.update_scene = false;
            }
        }
    }

    fn sync_framebuffer_from_shared(&mut self) {
        for (dst, src) in self
            .g_data
            .framebuffer
            .data
            .iter_mut()
            .zip(self.shared_render_buffer.iter())
        {
            *dst = src.load(Ordering::Relaxed);
        }
    }

    pub fn setup_scene(&mut self) {
        let Some(scene) = Arc::get_mut(&mut self.scene) else {
            return;
        };

        let mat_ground = Lambertian::new(Color::rgb_f(0.5, 0.5, 0.5));
        scene.add_object(Sphere::new(Vec3(0.0, -1000.0, -1.0), 1000.00, Arc::new(mat_ground)));

        for i in -11..11 {
            for j in -11..11 {
                let random_mat = random_f64();
                let center = Vec3(i as f64 + 0.9 * random_f64(), 0.2, j as f64 + 0.9 * random_f64());

                if (center - Vec3(4.0, 0.2, 0.0)).length() > 0.9 {
                    let mat: Arc<dyn Material>;

                    if random_mat < 0.8 {
                        let albedo = LinearColor::random().to_color();
                        mat = Arc::new(Lambertian::new(albedo));
                        scene.add_object(Sphere::new(center, 0.2, mat));
                    }
                    else if random_mat < 0.95 {
                        let albedo = LinearColor::random_limit(0.5, 1.0).to_color();
                        let fuzz = random_f64() * 0.5;
                        mat = Arc::new(Metal::new(albedo, fuzz));
                        scene.add_object(Sphere::new(center, 0.2, mat));
                    }
                    else {
                        mat = Arc::new(Dielectric::new(1.5));
                        scene.add_object(Sphere::new(center, 0.2, mat));
                    }
                }
            }
        }

        let mat = Dielectric::new(1.5);
        scene.add_object(Sphere::new(Vec3(0.0, 1.0, 0.0), 1.0, Arc::new(mat)));

        let mat = Lambertian::new(Color::rgb_f(0.1, 0.2, 0.5));
        scene.add_object(Sphere::new(Vec3(-4.0, 1.0, 0.0), 1.0, Arc::new(mat)));

        let mat = Metal::new(Color::rgb_f(0.7, 0.6, 0.5), 0.0);
        scene.add_object(Sphere::new(Vec3(4.0, 1.0, 0.0), 1.0, Arc::new(mat)));
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Rustracer")
            .with_inner_size(PhysicalSize::new(self.config.width, self.config.height));

        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                log::error!("failed to create window: {err}");
                event_loop.exit();
                return;
            }
        };

        let state = pollster::block_on(GpuState::new(window.clone()));
        let context = GraphicsContext { state };

        self.g_context = Some(context);
        self.window = Some(window.clone());

        if let Some(context) = self.g_context.as_mut() {
            context.state.upload_buffer(&self.g_data.framebuffer);
        }

        self.setup_scene();
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let Some(context) = self.g_context.as_mut() {
                    context.state.resize(new_size);
                }

                self.update_scene = true; // Resize Buffer???
            }
            WindowEvent::RedrawRequested => {
                Self::update(self);
                Self::render(self);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

struct GraphicsContext {
    state: GpuState,
}

struct GraphicsData {
    framebuffer: Framebuffer,
}
