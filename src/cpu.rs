pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,

    status_c: bool,
    status_z: bool,
    status_i: bool,
    status_d: bool,
    status_v: bool,
    status_n: bool,

    cycles: u64,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,

            status_c: false,
            status_z: false,
            status_i: false,
            status_d: false,
            status_v: false,
            status_n: false,

            cycles: 0
        }
    }
}
