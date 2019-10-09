use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} [path to rom file]", args[0]);
        process::exit(1);
    }

    let rom: &str = &args[1];
    println!("Loading rom: {}", rom);
}
