use std::fs;

pub struct RomHeader {
    prg_rom_size: u8,  // in 16KB units
    chr_rom_size: u8,  // in  8KB units
    flags6: u8,
    flags7: u8,
    flags_ext: Vec<u8>,
}

pub struct Cartridge {
    header: RomHeader,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper_id: u8,
    mirror_mode: u8,
}

impl Cartridge {
    pub fn load(path: &str) -> Cartridge {
        let data = fs::read(path).expect("Error reading rom file");
        assert!(data.len() >= 16);
        assert!(&data[0..4] == [0x4E, 0x45, 0x53, 0x1A], "not an iNES file");

        let header = RomHeader {
            prg_rom_size: data[4],
            chr_rom_size: data[5],
            flags6: data[6],
            flags7: data[7],
            flags_ext: data[8..16].to_vec(),
        };

        let mut index: usize = 16;
        let prg_rom = &data[index..(index + (16384 * header.prg_rom_size as usize))];
        index += 16384 * header.prg_rom_size as usize;
        let chr_rom = &data[index..(index + (8192 * header.chr_rom_size as usize))];
        index += 8192 * header.chr_rom_size as usize;

        Cartridge {
            prg_rom: prg_rom.to_vec(),
            chr_rom: chr_rom.to_vec(),
            mapper_id: (header.flags7 & 0xF0) | (header.flags6 >> 4),
            mirror_mode: (header.flags6 & 0x1) | (header.flags6 & 0x8 >> 2),
            header
        }
    }
}
