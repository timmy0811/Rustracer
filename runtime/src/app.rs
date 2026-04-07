use crate::framebuffer::Framebuffer;
use crate::gpu::GpuState;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tracer::camera::Camera;
use tracer::hittable::sphere::Sphere;
use tracer::scene::Scene;
use tracer::tracer::Tracer;
use tracer::vec3::Vec3;
use tracer::{RenderError, raycast_scene_parallel};
use utility::color::Color;
use utility::config::{AppConfig, ConfigError, DEFAULT_CONFIG_PATH};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

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
            camera: Camera::default(),
            scene: Arc::new(Scene::new(32)),
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

        self.update_scene = false;

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
                // Keep rendering batches running; later this can be driven by scene/input events.
                self.update_scene = false;
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

        let sph1 = Sphere::new(Vec3(0.0, 0.0, -1.0), 0.5);
        let sph2 = Sphere::new(Vec3(0.0, -100.5, -1.0), 100.0);
        scene.add_object(sph1);
        scene.add_object(sph2);
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
