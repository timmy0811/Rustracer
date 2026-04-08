use winit::keyboard::KeyCode;

pub enum InputAction {
    Exit,
    RenderFinalOnce,
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    LookUp,
    LookDown,
    LookLeft,
    LookRight,
    IncreaseDefocusAngle,
    DecreaseDefocusAngle,
    IncreaseFocusDist,
    DecreaseFocusDist,
}

pub struct InputState;

impl InputState {
    pub fn key_action(keycode: KeyCode) -> Option<InputAction> {
        match keycode {
            KeyCode::Escape => Some(InputAction::Exit),
            KeyCode::Space => Some(InputAction::RenderFinalOnce),
            KeyCode::KeyW => Some(InputAction::MoveForward),
            KeyCode::KeyS => Some(InputAction::MoveBackward),
            KeyCode::KeyA => Some(InputAction::MoveLeft),
            KeyCode::KeyD => Some(InputAction::MoveRight),
            KeyCode::ArrowUp => Some(InputAction::LookUp),
            KeyCode::ArrowDown => Some(InputAction::LookDown),
            KeyCode::ArrowLeft => Some(InputAction::LookLeft),
            KeyCode::ArrowRight => Some(InputAction::LookRight),
            KeyCode::KeyR => Some(InputAction::IncreaseDefocusAngle),
            KeyCode::KeyF => Some(InputAction::DecreaseDefocusAngle),
            KeyCode::KeyT => Some(InputAction::IncreaseFocusDist),
            KeyCode::KeyG => Some(InputAction::DecreaseFocusDist),
            _ => None,
        }
    }
}
