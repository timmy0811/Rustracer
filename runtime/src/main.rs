use runtime::app::App;
use utility::config::ConfigError;
use winit::event_loop::EventLoop;

fn main() {
    let app = match App::new() {
        Ok(app) => app,
        Err(config_err) => {
            match config_err {
                ConfigError::Read { path, source } => {
                    log::error!("failed to read config file `{path}`: {source}");
                }
                ConfigError::Parse { path, source } => {
                    log::error!("failed to parse yaml config `{path}`: {source}");
                }
                ConfigError::Validation(message) => {
                    log::error!("invalid config value: {message}");
                }
            }

            std::process::exit(1);
        }
    };

    let event_loop = EventLoop::new().expect("failed to create event loop");

    if let Err(err) = app.run(event_loop) {
        log::error!("event loop error: {err}");
        std::process::exit(1);
    }
}
