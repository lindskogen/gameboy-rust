use std::env;
use std::time::Duration;

use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

use dmg::dmg::core::Core;
use dmg::dmg::input::JoypadInput;
use dmg::emulator::audio::setup_audio_device;
use dmg::emulator::state::restore_state;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    let game_rom = env::args().nth(1);

    if let Some(name) = &game_rom {
        eprintln!("Loading {}", name);
    }

    let mut display_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];


    let mut options = WindowOptions::default();
    options.scale = Scale::X4;
    options.resize = true;

    let mut window = Window::new("gameboy", WIDTH, HEIGHT, options).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);


    let (mut audio_player, audio_stream) = setup_audio_device();

    let new_core = Core::load_without_boot_rom(game_rom);

    let old_core = restore_state();

    let mut core = match old_core {
        Some(c) if c.read_rom_name() == new_core.read_rom_name() => {
            c
        }
        _ => new_core
    };

    // core.initialize_gameboy_doctor();

    let title = core.read_rom_name();

    window.set_title(&title);


    while window.is_open() && !window.is_key_down(Key::Escape) {
        let keys_pressed = detect_keys(&window);

        let should_render = core.step(&mut display_buffer, &mut audio_player, keys_pressed);

        if should_render {
            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window.update_with_buffer(&display_buffer, WIDTH, HEIGHT).unwrap();
        }

        if window.is_key_down(Key::LeftSuper) && window.is_key_pressed(Key::S, KeyRepeat::Yes) {
            write_buffer_to_file(&display_buffer);
        }
    }

    // let _ = save_state(&core);
}

fn detect_keys(window: &Window) -> JoypadInput {
    let mut keys_pressed = JoypadInput::empty();

    if window.is_key_down(Key::Up) { keys_pressed |= JoypadInput::UP; }
    if window.is_key_down(Key::Left) { keys_pressed |= JoypadInput::LEFT; }
    if window.is_key_down(Key::Down) { keys_pressed |= JoypadInput::DOWN; }
    if window.is_key_down(Key::Right) { keys_pressed |= JoypadInput::RIGHT; }
    if window.is_key_down(Key::Enter) { keys_pressed |= JoypadInput::START; }
    if window.is_key_down(Key::RightShift) { keys_pressed |= JoypadInput::SELECT; }
    if window.is_key_down(Key::Z) { keys_pressed |= JoypadInput::A; }
    if window.is_key_down(Key::X) { keys_pressed |= JoypadInput::B; }

    keys_pressed
}

fn write_buffer_to_file(buffer: &Vec<u32>) {
    let mut slice: Vec<u8> = Vec::new();
    for num in buffer.iter() {
        slice.append(&mut num.to_ne_bytes().to_vec());
    }
    let result = image::save_buffer(
        "image.png",
        &slice,
        WIDTH as u32,
        HEIGHT as u32,
        image::ColorType::Rgba8,
    );

    match result {
        Ok(_) => println!("Saved image to {}", "image.png"),
        Err(e) => eprintln!("Failed saving image: {}", e),
    }
}
