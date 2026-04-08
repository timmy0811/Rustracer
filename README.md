# Rustracer

A simple Sphere-Raytracer written in Rust rendering a random generated Scene.
- **Preview mode** for interactive camera movement and fast feedback.
- **Final mode** for higher quality still renders.

The app uses `winit` + `wgpu` for windowing/display and a CPU path tracer backend for ray casting.

## Workspace layout

- `runtime/` - windowing, GPU upload/present path, input handling, scene bootstrap.
- `tracer/` - ray/path tracing logic (camera, materials, hittables, workers).
- `utility/` - shared config, color, and random utilities.

## Run

From the workspace root:

```bash
cargo run -p runtime
```

The runtime reads settings from `config.yml`.

## Controls

> While a final render is in progress, keyboard input is ignored until it completes.

| Key | Action |
| --- | --- |
| `W` | Move camera forward |
| `S` | Move camera backward |
| `A` | Move camera left |
| `D` | Move camera right |
| `Arrow Up` | Look up |
| `Arrow Down` | Look down |
| `Arrow Left` | Look left |
| `Arrow Right` | Look right |
| `R` | Increase camera defocus angle |
| `F` | Decrease camera defocus angle |
| `T` | Increase camera focus distance |
| `G` | Decrease camera focus distance |
| `Space` | Start one final-quality render |
| `Esc` | Exit |

## Render modes

- **Preview mode**
  - Lower resolution and sample settings (`preview_*` fields in `config.yml`).
  - Used for interactive navigation.
- **Final mode**
  - Uses final settings (`width`, `samples_per_pixel`, `max_bounces`).
  - Triggered by `Space`.
  - After camera/input changes, the app switches back to preview mode.

## Configuration (`config.yml`)

Main fields:

- `min_viewport_width`: minimum window width.
- `aspect_ratio`: image ratio (supports values like `1.7777` or `16/9`).
- `threads`: worker thread count (`1..=32`).
- `frame_time`: upload interval in milliseconds.
- `camera_move_step`: movement step for `W/A/S/D`.
- `camera_look_step_radians`: turn step for arrow keys.
- `preview_width`, `preview_samples_per_pixel`, `preview_max_bounces`: preview quality.
- `width`, `samples_per_pixel`, `max_bounces`: final quality.

Heights are computed automatically from width and `aspect_ratio`.

## Notes

- The default scene is generated at startup (ground + randomized spheres + three large spheres).
- Final render time is logged when a final frame completes.
