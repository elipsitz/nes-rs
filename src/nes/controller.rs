#[derive(Default, Debug)]
pub struct ControllerState {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl ControllerState {
    pub fn new() -> ControllerState {
        ControllerState::default()
    }
}