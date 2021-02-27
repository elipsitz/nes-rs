extern crate clap;
extern crate sdl2;

use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
    time::{Duration, Instant},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use nes_core::ControllerState;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;
const SCALE: u32 = 2;

fn get_controller_state(event_pump: &sdl2::EventPump) -> (ControllerState, ControllerState) {
    let mut controller1 = ControllerState::default();
    let controller2 = ControllerState::default();
    let keyboard_state = event_pump.keyboard_state();
    let keys = keyboard_state
        .pressed_scancodes()
        .filter_map(Keycode::from_scancode);
    for key in keys {
        match key {
            Keycode::Z => {
                controller1.a = true;
            }
            Keycode::X => {
                controller1.b = true;
            }
            Keycode::RShift => {
                controller1.select = true;
            }
            Keycode::Return => {
                controller1.start = true;
            }
            Keycode::Up => {
                controller1.up = true;
            }
            Keycode::Down => {
                controller1.down = true;
            }
            Keycode::Left => {
                controller1.left = true;
            }
            Keycode::Right => {
                controller1.right = true;
            }
            _ => {}
        }
    }
    (controller1, controller2)
}

fn run_emulator(
    nes: &mut nes_core::Nes,
    mut audio_out: Option<hound::WavWriter<BufWriter<File>>>,
    save_state_path: &str,
) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let window = video_subsystem
        .window("NES", WIDTH * SCALE, HEIGHT * SCALE)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::ABGR8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let mut debug_texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::ABGR8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;
    debug_texture.set_blend_mode(BlendMode::Blend);

    let audio_spec_desired = sdl2::audio::AudioSpecDesired {
        freq: Some(nes_core::AUDIO_SAMPLE_RATE as i32),
        channels: Some(1),
        samples: None,
    };
    let audio_device = audio_subsystem.open_queue::<f32, _>(None, &audio_spec_desired)?;
    audio_device.resume();

    let mut frame_counter = 0;
    let mut frame_timer = Instant::now();
    let mut paused = false;
    let mut single_step = false;
    // Was paused before focus was lost.
    let mut was_paused = paused;

    let mut event_pump = sdl_context.event_pump()?;
    let mut last_event: Option<sdl2::event::Event> = None;
    'running: loop {
        // Handle events.
        loop {
            // Allow for events we waited for previously.
            if last_event.is_none() {
                last_event = event_pump.poll_event();
                if last_event.is_none() {
                    break;
                }
            }
            match last_event.take().unwrap() {
                sdl2::event::Event::Quit { .. } => {
                    break 'running;
                }
                sdl2::event::Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::FocusGained => {
                        paused = was_paused;
                    }
                    sdl2::event::WindowEvent::FocusLost => {
                        was_paused = paused;
                        paused = true;
                    }
                    _ => {}
                },
                sdl2::event::Event::KeyDown {
                    keycode: Some(code),
                    keymod,
                    ..
                } => match code {
                    Keycode::Space => {
                        paused = !paused;
                    }
                    Keycode::Tab => {
                        paused = true;
                        single_step = true;
                    }
                    Keycode::Escape => {
                        break 'running;
                    }
                    Keycode::Backquote => {
                        nes.debug_toggle_overlay();
                    }
                    Keycode::S if keymod == sdl2::keyboard::Mod::LGUIMOD => {
                        // Save
                        let state = nes.get_state();
                        let f = std::fs::File::create(save_state_path).unwrap();
                        let mut writer = ZlibEncoder::new(f, Compression::default());
                        writer.write_all(&state).unwrap();
                        println!("Saved state to {}", save_state_path);
                    }
                    Keycode::L if keymod == sdl2::keyboard::Mod::LGUIMOD => {
                        // Load
                        let mut output = vec![];
                        let result = std::fs::File::open(save_state_path)
                            .map_err(|_| ())
                            .map(|f| ZlibDecoder::new(f))
                            .and_then(|mut d| d.read_to_end(&mut output).map_err(|_| ()))
                            .and_then(|_| nes.set_state(&mut output));
                        match result {
                            Ok(_) => println!("Loaded state from {}", save_state_path),
                            Err(_) => println!("Nothing to load."),
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let (controller1, controller2) = get_controller_state(&event_pump);
        nes.set_controller1_state(controller1);
        nes.set_controller2_state(controller2);

        if !paused || single_step {
            single_step = false;
            nes.emulate_frame();
            frame_counter += 1;
            let buf = nes.get_frame_buffer();
            texture
                .update(None, buf, (WIDTH * 4) as usize)
                .map_err(|e| e.to_string())?;
            canvas.copy(&texture, None, None)?;

            // Target maximum of 8 frames of samples in the buffer.
            let samples_queued = (audio_device.size() as usize) / 4;
            let samples_max = 8 * nes_core::AUDIO_SAMPLE_RATE / 60;
            if samples_queued < samples_max {
                let buffer = nes.get_audio_buffer();
                let to_add = usize::min(buffer.len(), samples_max - samples_queued);
                audio_device.queue(&buffer[..to_add]);
            }
            if let Some(f) = &mut audio_out {
                for &sample in nes.get_audio_buffer() {
                    f.write_sample(sample).unwrap();
                }
            }

            if nes.debug_render_enabled() {
                let buf = nes.debug_get_overlay_buffer();
                debug_texture
                    .update(None, buf, (WIDTH * 4) as usize)
                    .map_err(|e| e.to_string())?;
                canvas.copy(&debug_texture, None, None)?;
            }

            canvas.present();
        } else {
            last_event = Some(event_pump.wait_event());
        }

        // FPS display
        if Instant::now() - frame_timer > Duration::from_secs(1) {
            canvas
                .window_mut()
                .set_title(&format!("NES - FPS: {}", frame_counter))
                .map_err(|e| e.to_string())?;
            frame_counter = 0;
            frame_timer = Instant::now();
        }
    }

    Ok(())
}

fn main() {
    let args = clap::App::new("nes_rs")
        .author("Eli Lipsitz <eli.lipsitz@gmail.com>")
        .arg(
            clap::Arg::with_name("rom")
                .help("Path to the rom file to use")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("cpu-log")
                .long("cpu-log")
                .help("Print CPU execution log"),
        )
        .arg(
            clap::Arg::with_name("audio-output")
                .long("audio-output")
                .takes_value(true)
                .help("Output audio to a WAV file"),
        )
        .get_matches();

    let rom_path: &str = args.value_of("rom").unwrap();
    println!("[main] Loading rom at path: {}", rom_path);

    let mut debug = nes_core::Debug::default();
    debug.cpu_log = args.is_present("cpu-log");

    let audio_out = args.value_of("audio-output").map(|filename| {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: nes_core::AUDIO_SAMPLE_RATE as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        hound::WavWriter::create(filename, spec).unwrap()
    });

    let rom_filename = Path::new(rom_path).file_name().unwrap().to_str().unwrap();
    let save_state_path = format!("state_{}.nes_state", rom_filename);

    let cartridge_data = std::fs::read(rom_path).expect("Error reading rom file");
    let cart = nes_core::Cartridge::load(&cartridge_data);
    let mut nes = Box::new(nes_core::Nes::new(debug, cart));
    run_emulator(nes.as_mut(), audio_out, &save_state_path).unwrap();
}
