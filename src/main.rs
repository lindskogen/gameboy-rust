extern crate dmg;
extern crate minifb;
extern crate image;

use std::env;

use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

use dmg::dmg::Core;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    let game_rom = env::args().nth(1);

    if let Some(name) = &game_rom {
        println!("Loading {}", name);
    }

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut options = WindowOptions::default();
    options.scale = Scale::X4;
    options.resize = true;

    let mut window = Window::new(
        "gameboy",
        WIDTH,
        HEIGHT,
        options,
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });


    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut core = Core::load("./dmg_boot.bin", game_rom);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let elapsed = core.cpu.next();
        let should_render = core.cpu.mem.gpu.next(elapsed);

        if window.is_key_down(Key::LeftSuper) && window.is_key_pressed(Key::S, KeyRepeat::No) {
            write_buffer_to_file(&buffer);
        }

        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            core.cpu.mem.gpu.debug_print();
        }

        if should_render {
            core.cpu.mem.copy_vram_into_buffer(&mut buffer);


            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window
                .update_with_buffer(&buffer, WIDTH, HEIGHT)
                .unwrap();
        }
    }
}

fn write_buffer_to_file(buffer: &Vec<u32>) {
    let mut slice: Vec<u8> = Vec::new();
    for num in buffer.iter() {
        slice.append(&mut num.to_ne_bytes().to_vec());
    }
    let result = image::save_buffer("image.png", &slice, WIDTH as u32, HEIGHT as u32, image::ColorType::Rgba8);

    match result {
        Ok(_) => println!("Saved image to {}", "image.png"),
        Err(e) => eprintln!("Failed saving image: {}", e)
    }
}
