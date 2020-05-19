use super::nes::State;

fn status_pack(s: &State, status_b: bool) -> u8 {
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

fn status_unpack(s: &mut State, packed: u8) {
    s.cpu.status_c = packed & (1 << 0) > 0;
    s.cpu.status_z = packed & (1 << 1) > 0;
    s.cpu.status_i = packed & (1 << 2) > 0;
    s.cpu.status_d = packed & (1 << 3) > 0;
    s.cpu.status_v = packed & (1 << 6) > 0;
    s.cpu.status_n = packed & (1 << 7) > 0;
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
    s.cpu.cycles += 1; // Dummy read
    (address + (index as u16)) & 0xFF
}

// Absolute indexed: 2 bytes describe an address, plus the index.
fn address_absolute_indexed(s: &mut State, index: u8) -> (u16, u16) {
    let base = address_absolute(s);
    let fixed = base + (index as u16);
    let initial = (base & 0xFF00) | (fixed & 0xFF);
    (initial, fixed)
}

// Indexed indirect: 1 byte (+ X) is pointer to zero page, that address is used.
fn address_indexed_indirect(s: &mut State) -> u16 {
    let base = s.cpu_read(s.cpu.pc) as u16;
    s.cpu.pc += 1;
    s.cpu.cycles += 1; // Dummy read of base.
    let address = (base + (s.cpu.x as u16)) & 0x00FF;
    let lo = s.cpu_read(address);
    let hi = s.cpu_read(address + 1);
    (hi as u16) << 8 | (lo as u16)
}

// Indirect indexed: 1 byte is pointer to zero page, that address (+ Y) is used.
fn address_indirect_indexed(s: &mut State) -> (u16, u16) {
    let ptr = s.cpu_read(s.cpu.pc) as u16;
    s.cpu.pc += 1;

    let lo = s.cpu_read(ptr);
    let hi = s.cpu_read(ptr + 1);
    let base = (hi as u16) << 8 | (lo as u16);
    let fixed = base + (s.cpu.y as u16);
    let initial = (base & 0xFF00) | (fixed & 0xFF);
    (initial, fixed)
}

// Set status registers for the loaded value.
fn set_status_load(s: &mut State, val: u8) {
    s.cpu.status_z = val == 0;
    s.cpu.status_n = val & 0x80 > 0;
}

// Compute Add with Carry
fn compute_adc(s: &mut State, data: u8) -> u8 {
    let a = s.cpu.a as u16;
    let b = data as u16;
    let c = s.cpu.status_c as u16;
    let result = a + b + c;
    s.cpu.status_c = result > 0xFF;
    s.cpu.status_v = (a ^ b) & 0x80 == 0 && ( a ^ result) & 0x80 != 0;
    (result & 0xFF) as u8
}

// Compute Bit Test
fn compute_bit(s: &mut State, data: u8) {
    s.cpu.status_z = (s.cpu.a & data) == 0;
    s.cpu.status_v = (data & 0x40) > 0;
    s.cpu.status_n = (data & 0x80) > 0;
}

fn do_branch(s: &mut State, condition: bool) {
    let offset = address_immediate(s) as i8;
    if condition {
        let old_pc = s.cpu.pc;
        let new_pc = ((old_pc as i32) + (offset as i32)) as u16;
        s.cpu.cycles += 1;
        if (old_pc & 0xFF00) != (new_pc & 0xFF00) {
            s.cpu.cycles += 1;
        }
        s.cpu.pc = new_pc;
    }
}

fn stack_push(s: &mut State, data: u8) {
    s.cpu_write(0x0100 | (s.cpu.sp as u16), data);
    s.cpu.sp -= 1;
}

fn stack_pull(s: &mut State) -> u8 {
    s.cpu.sp += 1;
    s.cpu_read(0x0100 | (s.cpu.sp as u16))
}

pub fn emulate(s: &mut State, min_cycles: u64) -> u64 {
    macro_rules! inst_fetch {
        (imm; $data:ident, $expr:block) => {
            {
                let $data = address_immediate(s);
                $expr
            }
        };
        (zero; $data:ident, $expr:block) => {
            {
                let addr = address_zero_page(s);
                let $data = s.cpu_read(addr);
                $expr
            }
        };
        (zero, $idx_reg:ident; $data:ident, $expr:block) => {
            {
                let addr = address_zero_page_indexed(s, s.cpu.$idx_reg);
                let $data = s.cpu_read(addr);
                $expr
            }
        };
        (abs; $data:ident, $expr:block) => {
            {
                let addr = address_absolute(s);
                let $data = s.cpu_read(addr);
                $expr
            }
        };
        (abs, $idx_reg:ident; $data:ident, $expr:block) => {
            {
                let (initial, fixed) = address_absolute_indexed(s, s.cpu.$idx_reg);
                let $data = if initial == fixed {
                    s.cpu_read(initial)
                } else {
                    s.cpu_read(initial);
                    s.cpu_read(fixed)
                };
                $expr
            }
        };
        // Indexed Indirect (Indirect,X)
        (indirect, x; $data:ident, $expr:block) => {
            {
                let address = address_indexed_indirect(s);
                let $data = s.cpu_read(address);
                $expr
            }
        };
        // Indirect Indexed (Indirect),Y
        (indirect, y; $data:ident, $expr:block) => {
            {
                let (initial, fixed) = address_indirect_indexed(s);
                let $data = if initial == fixed {
                    s.cpu_read(initial)
                } else {
                    s.cpu_read(initial);
                    s.cpu_read(fixed)
                };
                $expr
            }
        };
    }

    macro_rules! inst_load {
        ($mode:tt; $data:ident, $reg:ident, $expr:block) => {
            {
                let result = inst_fetch!($mode; $data, $expr);
                s.cpu.$reg = result;
                set_status_load(s, result);
            }
        };
        ($mode:tt, $idx_reg:tt; $data:ident, $reg:ident, $expr:block) => {
            {
                let result = inst_fetch!($mode, $idx_reg; $data, $expr);
                s.cpu.$reg = result;
                set_status_load(s, result);
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
        (abs, $idx_reg:ident; $expr:block) => {
            {
                let (initial, fixed) = address_absolute_indexed(s, s.cpu.$idx_reg);
                s.cpu_read(initial);
                let data = $expr;
                s.cpu_write(fixed, data);
            }
        };
        // Indexed Indirect (Indirect,X)
        (indirect, x; $expr:block) => {
            {
                let address = address_indexed_indirect(s);
                let data = $expr;
                s.cpu_write(address, data);
            }
        };
        // Indirect Indexed (Indirect),Y
        (indirect, y; $expr:block) => {
            {
                let (initial, fixed) = address_indirect_indexed(s);
                s.cpu_read(initial);
                let data = $expr;
                s.cpu_write(fixed, data);
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
            s.cpu.pc, opcode, s.cpu.a, s.cpu.x, s.cpu.y, status_pack(s, false), s.cpu.sp, cycle
        );
        s.cpu.pc += 1;

        match opcode {
            // ADC - Add with Carry
            0x69 => inst_load!(imm; data, a, { compute_adc(s, data) }),
            0x65 => inst_load!(zero; data, a, { compute_adc(s, data) }),
            0x75 => inst_load!(zero, x; data, a, { compute_adc(s, data) }),
            0x6D => inst_load!(abs; data, a, { compute_adc(s, data) }),
            0x7D => inst_load!(abs, x; data, a, { compute_adc(s, data) }),
            0x79 => inst_load!(abs, y; data, a, { compute_adc(s, data) }),
            0x61 => inst_load!(indirect, x; data, a, { compute_adc(s, data) }),
            0x71 => inst_load!(indirect, y; data, a, { compute_adc(s, data) }),
            // AND - Logical AND
            0x29 => inst_load!(imm; data, a, { s.cpu.a & data }),
            0x25 => inst_load!(zero; data, a, { s.cpu.a & data }),
            0x35 => inst_load!(zero, x; data, a, { s.cpu.a & data }),
            0x2D => inst_load!(abs; data, a, { s.cpu.a & data }),
            0x3D => inst_load!(abs, x; data, a, { s.cpu.a & data }),
            0x39 => inst_load!(abs, y; data, a, { s.cpu.a & data }),
            0x21 => inst_load!(indirect, x; data, a, { s.cpu.a & data }),
            0x31 => inst_load!(indirect, y; data, a, { s.cpu.a & data }),
            // TODO: ASL - Arithmetic Shift Left
            // BCC - Branch if Carry Clear
            0x90 => do_branch(s, !s.cpu.status_c),
            // BCS - Branch if Carry Set
            0xB0 => do_branch(s, s.cpu.status_c),
            // BEQ - Branch if Equal
            0xF0 => do_branch(s, s.cpu.status_z),
            // BIT - Bit Test
            0x24 => inst_fetch!(zero; data, { compute_bit(s, data) }),
            0x2C => inst_fetch!(abs; data, { compute_bit(s, data) }),
            // BMI - Branch if Minus
            0x30 => do_branch(s, s.cpu.status_n),
            // BNE - Branch if Not Equal
            0xD0 => do_branch(s, !s.cpu.status_z),
            // BPL - Branch if Positive
            0x10 => do_branch(s, !s.cpu.status_n),
            // TODO: BRK - Force Interrupt
            // BVC - Branch if Overflow Clear
            0x50 => do_branch(s, !s.cpu.status_v),
            // BVS - Branch if Overflow Set
            0x70 => do_branch(s, s.cpu.status_v),
            // CLC - Clear Carry Flag
            0x18 => { s.cpu.status_c = false; s.cpu.cycles += 1; }
            // CLD - Clear Decimal Mode
            0xD8 => { s.cpu.status_d = false; s.cpu.cycles += 1; }
            // CLI - Clear Interrupt Disable
            0x58 => { s.cpu.status_i = false; s.cpu.cycles += 1; }
            // CLV - Clear Overflow Flag
            0xB8 => { s.cpu.status_v = false; s.cpu.cycles += 1; }
            // TODO: CMP - Compare
            // TODO: CPX - Compare X Register
            // TODO: CPY - Compare Y Register
            // TODO: DEC - Decrement Memory
            // TODO: DEX - Decrement X Register
            // TODO: DEY - Decrement Y Register
            // TODO: EOR - Exclusive OR
            // TODO: INC - Increment Memory
            // TODO: INX - Increment X Register
            // TODO: INY - Increment Y Register
            // JMP - Jump
            0x4C => s.cpu.pc = address_absolute(s),
            0x6C => s.cpu.pc = address_indirect(s),
            // JSR - Jump to Subroutine
            0x20 => {
                let addr = address_absolute(s);
                let pc_store = s.cpu.pc - 1;
                let hi = (pc_store >> 8) & 0xFF;
                let lo = pc_store & 0xFF;
                stack_push(s, hi as u8);
                stack_push(s, lo as u8);
                s.cpu.cycles += 1;
                s.cpu.pc = addr;
            }
            // LDA - Load Accumulator
            0xA9 => inst_load!(imm; data, a, { data }),
            0xA5 => inst_load!(zero; data, a, { data }),
            0xB5 => inst_load!(zero, x; data, a, { data }),
            0xAD => inst_load!(abs; data, a, { data }),
            0xBD => inst_load!(abs, x; data, a, { data }),
            0xB9 => inst_load!(abs, y; data, a, { data }),
            0xA1 => inst_load!(indirect, x; data, a, { data }),
            0xB1 => inst_load!(indirect, y; data, a, { data }),
            // LDX - Load X Register
            0xA2 => inst_load!(imm; data, x, { data }),
            0xA6 => inst_load!(zero; data, x, { data }),
            0xB6 => inst_load!(zero, y; data, x, { data }),
            0xAE => inst_load!(abs; data, x, { data }),
            0xBE => inst_load!(abs, y; data, x, { data }),
            // LDY - Load Y Register
            0xA0 => inst_load!(imm; data, y, { data }),
            0xA4 => inst_load!(zero; data, y, { data }),
            0xB4 => inst_load!(zero, x; data, y, { data }),
            0xAC => inst_load!(abs; data, y, { data }),
            0xBC => inst_load!(abs, x; data, y, { data }),
            // TODO: LSR - Logical Shift Right
            // NOP - No Operation
            0xEA => { s.cpu.cycles += 1; }
            // TODO: ORA - Logical Inclusive OR
            // PHA - Push Accumulator
            0x48 => {
                s.cpu_read(s.cpu.pc); // Dummy read.
                stack_push(s, s.cpu.a);
            }
            // PHP - Push Processor Status
            0x08 => {
                s.cpu_read(s.cpu.pc); // Dummy read.
                stack_push(s, status_pack(s, true));
            }
            // PLA - Pull Accumulator
            0x68 => {
                s.cpu_read(s.cpu.pc); // Dummy read.
                s.cpu.cycles += 1;
                s.cpu.a = stack_pull(s);
                set_status_load(s, s.cpu.a);
            }
            // PLP - Pull Processor Status
            0x28 => {
                s.cpu_read(s.cpu.pc); // Dummy read.
                s.cpu.cycles += 1;
                let status = stack_pull(s);
                status_unpack(s, status);
            }
            // TODO: ROL - Rotate Left
            // TODO: ROR - Rotate Right
            // TODO: RTI - Return from Interrupt
            // RTS - Return from Subroutine
            0x60 => {
                s.cpu_read(s.cpu.pc); // Dummy read.
                s.cpu.cycles += 1;
                let lo = stack_pull(s) as u16;
                let hi = stack_pull(s) as u16;
                s.cpu.pc = (hi << 8) | lo;
                s.cpu_read(s.cpu.pc); // Dummy read.
                s.cpu.pc += 1;
            }
            // TODO: SBC - Subtract with Carry
            // SEC - Set Carry Flag
            0x38 => { s.cpu.status_c = true; s.cpu.cycles += 1; }
            // SED - Set Decimal Flag
            0xF8 => { s.cpu.status_d = true; s.cpu.cycles += 1; }
            // SEI - Set Interrupt Disable
            0x78 => { s.cpu.status_i = true; s.cpu.cycles += 1; }
            // STA - Store Accumulator
            0x85 => inst_write!(zero; { s.cpu.a }),
            0x95 => inst_write!(zero, x; { s.cpu.a }),
            0x8D => inst_write!(abs; { s.cpu.a }),
            0x9D => inst_write!(abs, x; { s.cpu.a }),
            0x99 => inst_write!(abs, y; { s.cpu.a }),
            0x81 => inst_write!(indirect, x; { s.cpu.a }),
            0x91 => inst_write!(indirect, y; { s.cpu.a }),
            // STX - Store X Register
            0x86 => inst_write!(zero; { s.cpu.x }),
            0x96 => inst_write!(zero, y; { s.cpu.x }),
            0x8E => inst_write!(abs; { s.cpu.x }),
            // STY - Store Y Register
            0x84 => inst_write!(zero; { s.cpu.y }),
            0x94 => inst_write!(zero, x; { s.cpu.y }),
            0x8C => inst_write!(abs; { s.cpu.y }),
            // TAX - Transfer Accumulator to X
            0xAA => { s.cpu.x = s.cpu.a; s.cpu.cycles += 1; set_status_load(s, s.cpu.x) }
            // TAY - Transfer Accumulator to Y
            0xA8 => { s.cpu.y = s.cpu.a; s.cpu.cycles += 1; set_status_load(s, s.cpu.y) }
            // TSX - Transfer Stack Pointer to X
            0xBA => { s.cpu.x = s.cpu.sp; s.cpu.cycles += 1; set_status_load(s, s.cpu.x) }
            // TXA - Transfer X to Accumulator
            0x8A => { s.cpu.a = s.cpu.x; s.cpu.cycles += 1; set_status_load(s, s.cpu.a) }
            // TXS - Transfer X to Stack Pointer
            0x9A => { s.cpu.sp = s.cpu.x; s.cpu.cycles += 1 }
            // TYA - Transfer Y to Accumulator
            0x98 => { s.cpu.a = s.cpu.y; s.cpu.cycles += 1; set_status_load(s, s.cpu.a) }
            _ => panic!("invalid instruction: 0x{:02X}", opcode)
        }
    }
    s.cpu.cycles - start_cycles
}