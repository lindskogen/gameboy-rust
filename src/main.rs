use std::env;
use std::time::Duration;

use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

use dmg::dmg::core::Core;
use dmg::dmg::input::JoypadInput;
use dmg::state::{restore_state, save_state};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    let game_rom = env::args().nth(1);

    if let Some(name) = &game_rom {
        eprintln!("Loading {}", name);
    }

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut options = WindowOptions::default();
    options.scale = Scale::X4;
    options.resize = true;

    let mut window = Window::new("gameboy", WIDTH, HEIGHT, options).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(Duration::from_micros(16600)));


    let new_core = Core::load("./dmg_boot.bin", game_rom);

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
        let mut keys_pressed = JoypadInput::empty();

        if window.is_key_down(Key::Up) { keys_pressed |= JoypadInput::UP; }
        if window.is_key_down(Key::Left) { keys_pressed |= JoypadInput::LEFT; }
        if window.is_key_down(Key::Down) { keys_pressed |= JoypadInput::DOWN; }
        if window.is_key_down(Key::Right) { keys_pressed |= JoypadInput::RIGHT; }
        if window.is_key_down(Key::Enter) { keys_pressed |= JoypadInput::START; }
        if window.is_key_down(Key::RightShift) { keys_pressed |= JoypadInput::SELECT; }
        if window.is_key_down(Key::Z) { keys_pressed |= JoypadInput::A; }
        if window.is_key_down(Key::X) { keys_pressed |= JoypadInput::B; }

        let should_render = core.step(&mut buffer, keys_pressed);

        if should_render {
            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        }

        if window.is_key_down(Key::LeftSuper) && window.is_key_pressed(Key::S, KeyRepeat::Yes) {
            write_buffer_to_file(&buffer);
        }
    }

    let _ = save_state(&core);
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
