use std::{env, process};

mod nes;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} [path to rom file]", args[0]);
        process::exit(1);
    }

    let rom_path: &str = &args[1];
    println!("[main] Loading rom at path: {}", rom_path);

    let cart = nes::cartridge::Cartridge::load(rom_path);
    let mut nes = nes::nes::Nes::new(cart);
    nes.run();
}
