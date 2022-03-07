extern crate dmg;
extern crate minifb;

use std::env;
use std::hint::spin_loop;

use minifb::{Key, Scale, Window, WindowOptions};

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

        if window.is_key_down(Key::Space) {
            core.cpu.mem.gpu.debug_print();
            loop {}
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
