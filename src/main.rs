extern crate dmg;
extern crate minifb;

use std::env;

use minifb::{Key, Window, WindowOptions};

use dmg::dmg::Core;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {

    let game_rom = env::args().nth(1);

    if let Some(name) = &game_rom {
        println!("Loading {}", name);
    }

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "gameboy",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });


    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut core = Core::load("./dmg_boot.bin", game_rom);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for _ in 0..100 {
            core.cpu.next();
        }
        for _ in 0..100 {
            core.cpu.step_ppu();
        }

        core.cpu.mem.copy_vram_into_buffer(&mut buffer);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
