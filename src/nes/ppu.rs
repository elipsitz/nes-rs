use super::nes::State;

pub struct PpuState {
    pub frames: u64,
}

impl PpuState {
    pub fn new() -> PpuState {
        PpuState {
            frames: 0,
        }
    }
}

pub fn emulate(s: &mut State, cycles: u64) {

}
