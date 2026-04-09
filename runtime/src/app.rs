use crate::framebuffer::Framebuffer;
use crate::gpu::GpuState;
use crate::input::{InputAction, InputState};
use env_logger::Env;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;
use std::f64::consts::FRAC_PI_2;
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
use utility::random::random_f64;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderMode {
    Preview,
    Final,
}

const CAMERA_DEFOCUS_STEP: f64 = 0.05;
const CAMERA_FOCUS_DIST_STEP: f64 = 0.5;
const MIN_FOCUS_DIST: f64 = 0.01;

pub struct App {
    window: Option<Arc<Window>>,
    g_context: Option<GraphicsContext>,
    g_data: GraphicsData,
    config: AppConfig,
    update_scene: bool,
    shared_render_buffer: Arc<Vec<AtomicU8>>,
    worker_job: Option<JoinHandle<Result<(), RenderError>>>,
    upload_interval: Duration,
    last_upload: Instant,

    camera: Camera,
    scene: Arc<Scene>,
    render_mode: RenderMode,
    min_viewport_size: PhysicalSize<u32>,
    preview_render_size: PhysicalSize<u32>,
    final_render_size: PhysicalSize<u32>,

    start_duration: Duration,
    is_final_render: bool,
}

impl App {
    pub fn new() -> Result<Self, ConfigError> {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

        log::info!("Welcome to Rustracer!");
        log::info!("Initializing App...");

        let config = AppConfig::load_from_file(DEFAULT_CONFIG_PATH)?;

        let min_viewport_size =
            PhysicalSize::new(config.min_viewport_width, config.min_viewport_height());
        let preview_render_size = PhysicalSize::new(config.preview_width, config.preview_height());
        let final_render_size = PhysicalSize::new(config.width, config.final_height());

        let mut framebuffer =
            Framebuffer::new(preview_render_size.width, preview_render_size.height, 4);
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
            config,
            update_scene: true,
            shared_render_buffer,
            worker_job: None,
            upload_interval: Duration::from_millis(frametime),
            last_upload: Instant::now() - Duration::from_millis(frametime),
            camera,
            scene: Arc::new(Scene::new(32)),
            render_mode: RenderMode::Preview,
            min_viewport_size,
            preview_render_size,
            final_render_size,
            start_duration: Duration::ZERO,
            is_final_render: false,
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
        let (samples_per_pixel, max_bounces) = match self.render_mode {
            RenderMode::Preview => (
                self.config.preview_samples_per_pixel,
                self.config.preview_max_bounces,
            ),
            RenderMode::Final => (self.config.samples_per_pixel, self.config.max_bounces),
        };

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
                if self.is_final_render {
                    let frame_time =
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - self.start_duration;
                    log::info!("frame rendered in {frame_time:?}");
                }

                self.is_final_render = false;
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

    fn resize_render_target(&mut self, new_size: PhysicalSize<u32>) {
        if self.g_data.framebuffer.width == new_size.width
            && self.g_data.framebuffer.height == new_size.height
        {
            return;
        }

        self.g_data.framebuffer = Framebuffer::new(new_size.width, new_size.height, 4);
        self.g_data.framebuffer.fill(Color::rgb(0, 0, 0));
        self.shared_render_buffer = Arc::new(
            (0..self.g_data.framebuffer.data.len())
                .map(|_| AtomicU8::new(0))
                .collect::<Vec<_>>(),
        );
    }

    fn request_final_render(&mut self) {
        if self.worker_job.is_some() {
            return;
        }

        self.render_mode = RenderMode::Final;
        self.resize_render_target(self.final_render_size);
        self.ensure_window_size(self.final_render_size);
        self.update_scene = true;
        self.is_final_render = true;
    }

    fn switch_to_preview_mode(&mut self) {
        self.render_mode = RenderMode::Preview;
        self.resize_render_target(self.preview_render_size);
        self.ensure_window_size(self.min_viewport_size);

        self.update_scene = true;
    }

    fn ensure_window_size(&mut self, target_size: PhysicalSize<u32>) {
        let Some(window) = &self.window else {
            return;
        };

        let current_size = window.inner_size();
        if current_size == target_size {
            if let Some(context) = self.g_context.as_mut() {
                context.state.resize(target_size);
            }
            return;
        }

        // Request exact target dimensions; many platforms apply this asynchronously.
        let requested_size = window.request_inner_size(target_size).unwrap_or(current_size);
        if let Some(context) = self.g_context.as_mut() {
            context.state.resize(requested_size);
        }
    }

    fn mark_scene_dirty_preview(&mut self) {
        if self.render_mode == RenderMode::Preview {
            self.update_scene = true;
        } else {
            self.switch_to_preview_mode();
        }
    }

    fn move_camera(&mut self, movement: Vec3) {
        self.camera.lookfrom += movement;
        self.camera.lookat += movement;
        self.camera.init();
        self.mark_scene_dirty_preview();
    }

    fn rotate_camera(&mut self, yaw_delta: f64, pitch_delta: f64) {
        let forward = (self.camera.lookat - self.camera.lookfrom).normalized();
        if forward.near_zero() {
            return;
        }

        let mut yaw = forward.z().atan2(forward.x());
        let mut pitch = forward.y().asin();
        let pitch_limit = FRAC_PI_2 - 0.01;

        yaw += yaw_delta;
        pitch = (pitch + pitch_delta).clamp(-pitch_limit, pitch_limit);

        let look_dir = Vec3(
            pitch.cos() * yaw.cos(),
            pitch.sin(),
            pitch.cos() * yaw.sin(),
        )
        .normalized();

        self.camera.lookat = self.camera.lookfrom + look_dir;
        self.camera.init();
        self.mark_scene_dirty_preview();
    }

    fn adjust_defocus_angle(&mut self, delta: f64) {
        self.camera.defocus_angle = (self.camera.defocus_angle + delta).max(0.0);
        log::info!("defocus_angle adjusted to {:.3}", self.camera.defocus_angle);
        self.camera.init();
        self.mark_scene_dirty_preview();
    }

    fn adjust_focus_dist(&mut self, delta: f64) {
        self.camera.focus_dist = (self.camera.focus_dist + delta).max(MIN_FOCUS_DIST);
        log::info!("focus_dist adjusted to {:.3}", self.camera.focus_dist);
        self.camera.init();
        self.mark_scene_dirty_preview();
    }

    fn snapshot_current_render_buffer(&self) -> Vec<u8> {
        self.shared_render_buffer
            .iter()
            .map(|byte| byte.load(Ordering::Relaxed))
            .collect()
    }

    fn save_current_buffer_to_file(&self) {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(err) => {
                log::error!("failed to build timestamp for screenshot: {err}");
                return;
            }
        };

        let output_dir = Path::new("screenshots");
        if let Err(err) = fs::create_dir_all(output_dir) {
            log::error!("failed to create screenshot directory: {err}");
            return;
        }

        let filename = format!("render_{timestamp_ms}.png");
        let output_path = output_dir.join(filename);

        let file = match File::create(&output_path) {
            Ok(file) => file,
            Err(err) => {
                log::error!("failed to create screenshot file: {err}");
                return;
            }
        };

        let pixels = self.snapshot_current_render_buffer();
        let encoder = PngEncoder::new(BufWriter::new(file));
        if let Err(err) = encoder.write_image(
            &pixels,
            self.g_data.framebuffer.width,
            self.g_data.framebuffer.height,
            ColorType::Rgba8.into(),
        ) {
            log::error!("failed to write screenshot: {err}");
            return;
        }

        log::info!("saved screenshot: {}", output_path.display());
    }

    fn apply_input_action(&mut self, action: InputAction, event_loop: &ActiveEventLoop) {
        match action {
            InputAction::Exit => event_loop.exit(),
            InputAction::RenderFinalOnce => self.request_final_render(),
            InputAction::SaveFrameBuffer => self.save_current_buffer_to_file(),
            InputAction::MoveForward => {
                let forward = (self.camera.lookat - self.camera.lookfrom).normalized();
                if !forward.near_zero() {
                    self.move_camera(forward * self.config.camera_move_step);
                }
            }
            InputAction::MoveBackward => {
                let forward = (self.camera.lookat - self.camera.lookfrom).normalized();
                if !forward.near_zero() {
                    self.move_camera(-forward * self.config.camera_move_step);
                }
            }
            InputAction::MoveLeft => {
                let right = self.camera.u.normalized();
                if !right.near_zero() {
                    self.move_camera(-right * self.config.camera_move_step);
                }
            }
            InputAction::MoveRight => {
                let right = self.camera.u.normalized();
                if !right.near_zero() {
                    self.move_camera(right * self.config.camera_move_step);
                }
            }
            InputAction::LookUp => {
                self.rotate_camera(0.0, self.config.camera_look_step_radians);
            }
            InputAction::LookDown => {
                self.rotate_camera(0.0, -self.config.camera_look_step_radians);
            }
            InputAction::LookLeft => {
                self.rotate_camera(-self.config.camera_look_step_radians, 0.0);
            }
            InputAction::LookRight => {
                self.rotate_camera(self.config.camera_look_step_radians, 0.0);
            }
            InputAction::IncreaseDefocusAngle => {
                self.adjust_defocus_angle(CAMERA_DEFOCUS_STEP);
            }
            InputAction::DecreaseDefocusAngle => {
                self.adjust_defocus_angle(-CAMERA_DEFOCUS_STEP);
            }
            InputAction::IncreaseFocusDist => {
                self.adjust_focus_dist(CAMERA_FOCUS_DIST_STEP);
            }
            InputAction::DecreaseFocusDist => {
                self.adjust_focus_dist(-CAMERA_FOCUS_DIST_STEP);
            }
        }
    }

    pub fn setup_scene(&mut self) {
        let Some(scene) = Arc::get_mut(&mut self.scene) else {
            return;
        };

        let mat_ground = Lambertian::new(Color::rgb_f(0.5, 0.5, 0.5));
        scene.add_object(Sphere::new(
            Vec3(0.0, -1000.0, -1.0),
            1000.00,
            Arc::new(mat_ground),
        ));

        for i in -11..11 {
            for j in -11..11 {
                let random_mat = random_f64();
                let center = Vec3(
                    i as f64 + 0.9 * random_f64(),
                    0.2,
                    j as f64 + 0.9 * random_f64(),
                );

                if (center - Vec3(4.0, 0.2, 0.0)).length() > 0.9 {
                    let mat: Arc<dyn Material>;

                    if random_mat < 0.8 {
                        let albedo = LinearColor::random().to_color();
                        mat = Arc::new(Lambertian::new(albedo));
                        scene.add_object(Sphere::new(center, 0.2, mat));
                    } else if random_mat < 0.95 {
                        let albedo = LinearColor::random_limit(0.5, 1.0).to_color();
                        let fuzz = random_f64() * 0.5;
                        mat = Arc::new(Metal::new(albedo, fuzz));
                        scene.add_object(Sphere::new(center, 0.2, mat));
                    } else {
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
            .with_inner_size(self.min_viewport_size)
            .with_min_inner_size(self.min_viewport_size)
            .with_resizable(false);

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

                self.update_scene = true;
            }
            WindowEvent::RedrawRequested => {
                Self::update(self);
                Self::render(self);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Some(action) = InputState::key_action(keycode) {
                    if self.is_final_render && !matches!(action, InputAction::SaveFrameBuffer) {
                        return;
                    }

                    self.apply_input_action(action, event_loop);
                }
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
