use super::nes::State;

fn pack_status_flags(s: &State, status_b: bool) -> u8 {
    0
        | (s.cpu.status_c as u8) << 0
        | (s.cpu.status_z as u8) << 1
        | (s.cpu.status_i as u8) << 2
        | (s.cpu.status_d as u8) << 3
        | (status_b as u8) << 4
        | (1) << 5
        | (s.cpu.status_v as u8) << 6
        | (s.cpu.status_n as u8) << 7
}

// Absolute addressing: 2 bytes describe a full 16-bit address to use.
fn address_absolute(s: &mut State) -> u16 {
    let lo = s.cpu_read(s.cpu.pc);
    let hi = s.cpu_read(s.cpu.pc + 1);
    s.cpu.pc += 2;
    (hi as u16) << 8 | (lo as u16)
}

// Indirect addressing: 2 bytes describe an address that contains the full 16-bit address to use.
fn address_indirect(s: &mut State) -> u16 {
    let address = address_absolute(s);
    let lo = s.cpu_read(address);
    let hi = s.cpu_read(address + 1);
    s.cpu.pc += 2;
    (hi as u16) << 8 | (lo as u16)
}

// Immediate addressing: 1 byte contains the *value* to use.
fn address_immediate(s: &mut State) -> u8 {
    let data = s.cpu_read(s.cpu.pc);
    s.cpu.pc += 1;
    data
}

// Zero page addressing: 1 byte contains the address on the zero page to use.
fn address_zero_page(s: &mut State) -> u16 {
    let address = s.cpu_read(s.cpu.pc) as u16;
    s.cpu.pc += 1;
    address
}

// Zero page indexed: 1 byte (+ index) contains the address *on the zero page* to use.
fn address_zero_page_indexed(s: &mut State, index: u8) -> u16 {
    let address = s.cpu_read(s.cpu.pc) as u16;
    s.cpu.pc += 1;
    s.cpu_read(address); // Dummy read
    (address + (index as u16)) & 0xFF
}

// Absolute indexed addressing: 2 bytes describe an address, plus the index.
fn address_absolute_indexed(s: &mut State, index: u8) -> (u16, u16) {
    let base = address_absolute(s);
    let fixed = base + (index as u16);
    let initial = (base & 0xFF00) | (fixed & 0xFF);
    (initial, fixed)
}

// Set status registers for the loaded value.
fn set_status_load(s: &mut State, val: u8) {
    s.cpu.status_z = val == 0;
    s.cpu.status_n = val & 0x80 > 0;
}

pub fn emulate(s: &mut State, min_cycles: u64) -> u64 {
    // Read instruction: args: addressing mode, destination register, arithmetic expression
    macro_rules! inst_read {
        (imm; $data:ident, $reg:ident, $expr:block) => {
            {
                let $data = address_immediate(s);
                s.cpu.$reg = $expr;
            }
        };
        (zero; $data:ident, $reg:ident, $expr:block) => {
            {
                let addr = address_zero_page(s);
                let $data = s.cpu_read(addr);
                s.cpu.$reg = $expr;
            }
        };
        (zero, $idx_reg:ident; $data:ident, $reg:ident, $expr:block) => {
            {
                let addr = address_zero_page_indexed(s, s.cpu.$idx_reg);
                let $data = s.cpu_read(addr);
                s.cpu.$reg = $expr;
            }
        };
        (abs; $data:ident, $reg:ident, $expr:block) => {
            {
                let addr = address_absolute(s);
                let $data = s.cpu_read(addr);
                s.cpu.$reg = $expr;
            }
        };
        (abs, $idx_reg:ident; $data:ident, $reg:ident, $expr:block) => {
            {
                let (initial, fixed) = address_absolute_indexed(s, s.cpu.$idx_reg);
                let $data = if initial == fixed {
                    s.cpu_read(initial)
                } else {
                    s.cpu_read(initial);
                    s.cpu_read(fixed)
                };
                s.cpu.$reg = $expr;
            }
        };
    }

    macro_rules! inst_write {
        (zero; $expr:block) => {
            {
                let addr = address_zero_page(s);
                let data = $expr;
                s.cpu_write(addr, data);
            }
        };
        (zero, $idx_reg:ident; $expr:block) => {
            {
                let addr = address_zero_page_indexed(s, s.cpu.$idx_reg);
                let data = $expr;
                s.cpu_write(addr, data);
            }
        };
        (abs; $expr:block) => {
            {
                let addr = address_absolute(s);
                let data = $expr;
                s.cpu_write(addr, data);
            }
        };
    }

    let start_cycles = s.cpu.cycles;
    let end_cycles = start_cycles + min_cycles;
    while s.cpu.cycles < end_cycles {
        let cycle = s.cpu.cycles;
        let opcode = s.cpu_read(s.cpu.pc);
        println!(
            "{:04X}  {:02X} ______________________________________ A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:_______ CYC:{}",
            s.cpu.pc, opcode, s.cpu.a, s.cpu.x, s.cpu.y, pack_status_flags(s, false), s.cpu.sp, cycle
        );
        s.cpu.pc += 1;

        match opcode {
            // JMP - Jump
            0x4C => s.cpu.pc = address_absolute(s),
            0x6C => s.cpu.pc = address_indirect(s),
            // LDX - Load X Register
            0xA2 => inst_read!(imm; data, x, { set_status_load(s, data); data }),
            0xA6 => inst_read!(zero; data, x, { set_status_load(s, data); data }),
            0xB6 => inst_read!(zero, y; data, x, { set_status_load(s, data); data }),
            0xAE => inst_read!(abs; data, x, { set_status_load(s, data); data }),
            0xBE => inst_read!(abs, y; data, x, { set_status_load(s, data); data }),
            // STX - Store X Register
            0x86 => inst_write!(zero; { s.cpu.x }),
            0x96 => inst_write!(zero, y; { s.cpu.x }),
            0x8E => inst_write!(abs; { s.cpu.x }),

            _ => panic!("invalid instruction: 0x{:02X}", opcode)
        }
    }
    s.cpu.cycles - start_cycles
}