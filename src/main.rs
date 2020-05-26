extern crate sdl2;

use std::time::{Duration, Instant};

mod nes;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;
const SCALE: u32 = 2;

fn run_emulator(mut nes: nes::nes::Nes) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("NES", WIDTH * SCALE, HEIGHT * SCALE)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::ABGR8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;

    let mut frame_counter = 0;
    let mut frame_timer = Instant::now();
    let mut paused = false;

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => { break 'running; }
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Space), .. } => {
                    paused = !paused;
                }
                _ => {}
            }
        }

        // Update
        let frame_start = Instant::now();

        if !paused {
            nes.emulate_frame();
            let buf = nes.get_frame_buffer();
            texture
                .update(None, buf, (WIDTH * 4) as usize)
                .map_err(|e| e.to_string())?;
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }

        // FPS display
        frame_counter += 1;
        if Instant::now() - frame_timer > Duration::from_secs(1) {
            canvas.window_mut()
                .set_title(&format!("NES - FPS: {}", frame_counter))
                .map_err(|e| e.to_string())?;
            frame_counter = 0;
            frame_timer = Instant::now();
        }

        let frame_end = Instant::now();
        let frame_time = frame_end.duration_since(frame_start);
        let period = Duration::from_nanos(1_000_000_000 / 60);
        if period > frame_time {
            std::thread::sleep(period - frame_time);
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} [path to rom file]", args[0]);
        std::process::exit(1);
    }

    let rom_path: &str = &args[1];
    println!("[main] Loading rom at path: {}", rom_path);

    let cart = nes::cartridge::Cartridge::load(rom_path);
    let nes = nes::nes::Nes::new(cart);
    run_emulator(nes).unwrap();
}
