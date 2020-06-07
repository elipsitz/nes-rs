extern crate clap;
extern crate sdl2;

use std::time::{Duration, Instant};

use sdl2::keyboard::Keycode;
use crate::nes::controller::ControllerState;

mod nes;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;
const SCALE: u32 = 2;

fn get_controller_state(event_pump: &sdl2::EventPump) -> (ControllerState, ControllerState) {
    let mut controller1 = ControllerState::default();
    let controller2 = ControllerState::default();
    let keyboard_state = event_pump.keyboard_state();
    let keys = keyboard_state.pressed_scancodes().filter_map(Keycode::from_scancode);
    for key in keys {
        match key {
            Keycode::Z => { controller1.a = true; }
            Keycode::X => { controller1.b = true; }
            Keycode::RShift => { controller1.select = true; }
            Keycode::Return => { controller1.start = true; }
            Keycode::Up => { controller1.up = true; }
            Keycode::Down => { controller1.down = true; }
            Keycode::Left => { controller1.left = true; }
            Keycode::Right => { controller1.right = true; }
            _ => {}
        }
    }
    (controller1, controller2)
}

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
    let mut single_step = false;

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        // Update
        let frame_start = Instant::now();

        // Check events.
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => { break 'running; }
                sdl2::event::Event::KeyDown { keycode: Some(code), .. } => match code {
                    Keycode::Space => { paused = !paused; }
                    Keycode::Tab => {
                        paused = true;
                        single_step = true;
                    }
                    Keycode::Escape => { break 'running; }
                    _ => {}
                }
                _ => {}
            }
        }

        let (controller1, controller2) = get_controller_state(&event_pump);
        nes.set_controller1_state(controller1);
        nes.set_controller2_state(controller2);

        if !paused || single_step {
            single_step = false;
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
    let args = clap::App::new("nes_rs")
        .author("Eli Lipsitz <eli.lipsitz@gmail.com>")
        .arg(clap::Arg::with_name("rom")
            .help("Path to the rom file to use")
            .required(true)
            .index(1))
        .get_matches();

    let rom_path: &str = args.value_of("rom").unwrap();
    println!("[main] Loading rom at path: {}", rom_path);

    let cart = nes::cartridge::Cartridge::load(rom_path);
    let nes = nes::nes::Nes::new(cart);
    run_emulator(nes).unwrap();
}
