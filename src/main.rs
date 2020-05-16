use std::{env, process};

mod cartridge;
mod cpu;
mod nes;
mod mapper;
mod ppu;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} [path to rom file]", args[0]);
        process::exit(1);
    }

    let rom_path: &str = &args[1];
    println!("Loading rom at path: {}", rom_path);

    let rom = cartridge::Cartridge::load(rom_path);
    let nes = nes::Nes::new_from_rom(rom);
}
