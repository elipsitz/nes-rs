pub struct RomHeader {
    prg_rom_size: u8, // in 16KB units
    chr_rom_size: u8, // in  8KB units
    flags6: u8,
    flags7: u8,
    _flags_ext: Vec<u8>,
}

pub struct Cartridge {
    pub(crate) _header: RomHeader,
    pub(crate) prg_rom: Vec<u8>,
    pub(crate) chr_rom: Vec<u8>,
    pub(crate) mapper_id: u8,
    pub(crate) mirror_mode: u8,
    pub(crate) _extra_data: Vec<u8>,
}

impl Cartridge {
    pub fn load(data: &[u8]) -> Cartridge {
        assert!(data.len() >= 16);
        assert!(&data[0..4] == [0x4E, 0x45, 0x53, 0x1A], "not an iNES file");

        let header = RomHeader {
            prg_rom_size: data[4],
            chr_rom_size: data[5],
            flags6: data[6],
            flags7: data[7],
            _flags_ext: data[8..16].to_vec(),
        };

        let mut index: usize = 16;
        let prg_rom = &data[index..(index + (16384 * header.prg_rom_size as usize))];
        let prg_rom = prg_rom.to_vec();
        index += 16384 * header.prg_rom_size as usize;
        let chr_rom = &data[index..(index + (8192 * header.chr_rom_size as usize))];
        let mut chr_rom = chr_rom.to_vec();
        index += 8192 * header.chr_rom_size as usize;
        let extra_data = &data[index..data.len()];

        if header.chr_rom_size == 0 {
            // 8KB of CHR RAM
            chr_rom = vec![0; 8192];
        }

        Cartridge {
            prg_rom,
            chr_rom,
            mapper_id: (header.flags7 & 0xF0) | (header.flags6 >> 4),
            mirror_mode: (header.flags6 & 0x1) | (header.flags6 & 0x8 >> 2),
            _header: header,
            _extra_data: extra_data.to_vec(),
        }
    }
}
