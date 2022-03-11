extern crate dmg;
extern crate image;
extern crate minifb;

use std::env;
use std::time::{Duration, Instant};

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

    let mut prev_time = Instant::now();
    let delta = Duration::from_micros(16600);

    let mut core = Core::load("./dmg_boot.bin", game_rom);



    let mut title = String::new();
    for i in 0x134..0x143 {
        let i1 = core.cpu.mem.read_byte(i);
        if i1 == 0 {
            break;
        }

        title += &(i1 as char).to_string();
    }

    window.set_title(&title);

    let mut running = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if running {
            let elapsed = core.cpu.next();
            let should_render = core.cpu.mem.gpu.next(elapsed, &mut buffer);

            if should_render {
                let current_time = Instant::now();
                if current_time > (prev_time + delta) {
                    prev_time = current_time;
                    // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
                    window
                        .update_with_buffer(&buffer, WIDTH, HEIGHT)
                        .unwrap();
                }
            }
        }

        if window.is_key_down(Key::LeftSuper) && window.is_key_pressed(Key::S, KeyRepeat::Yes) {
            write_buffer_to_file(&buffer);
        }

        if running && window.is_key_down(Key::P) {
            running = false;
            println!("Stopped");
        }

        if !running && window.is_key_down(Key::S) {
            running = true;
            println!("Started");
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
