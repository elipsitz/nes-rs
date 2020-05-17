use super::nes::State;

pub fn emulate(s: &mut State, min_cycles: u64) -> u64 {
    let start_cycles = s.cpu.cycles;
    let end_cycles = start_cycles + min_cycles;
    while s.cpu.cycles < end_cycles {
        let opcode = s.cpu_read(s.cpu.pc);
        println!(
            "{:04X}  {:02X} ______________________________________ A:{:02X} X:{:02X} Y:{:02X} P:__ SP:{:02X} PPU:_______ CYC:{}",
            s.cpu.pc, opcode, s.cpu.a, s.cpu.x, s.cpu.y, s.cpu.sp, s.cpu.cycles
        );
        // TODO: execute opcode
        s.cpu.cycles += 1;
    }
    s.cpu.cycles - start_cycles
}