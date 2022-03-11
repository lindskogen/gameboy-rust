use std::fmt::Debug;
use std::usize;

use bitflags::bitflags;

use dmg::intf::InterruptFlag;

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0xFF6B;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

bitflags! {
    pub struct Lcdc: u8 {
        const LCD_DISPLAY_ENABLE = 0b1000_0000;
        const WINDOW_TILE_MAP_DISPLAY_SELECT = 0b0100_0000;
        const WINDOW_DISPLAY_ENABLE = 0b0010_0000;
        const BG_AND_WINDOW_TILE_DATA_SELECT = 0b0001_0000;
        const BG_TILE_MAP_DISPLAY_SELECT = 0b0000_1000;
        const OBJ_SIZE = 0b0000_0100;
        const OBJ_DISPLAY_ENABLE = 0b0000_0010;
        const BG_AND_WINDOW_DISPLAY = 0b0000_0001;
    }
}

impl Lcdc {
    fn new() -> Self { Self { bits: 0b0100_1000 } }

    fn lcd_display_enable(&self) -> bool { self.contains(Lcdc::LCD_DISPLAY_ENABLE) }

    fn bg_and_window_tile_data_select(&self) -> u16 { if self.contains(Lcdc::BG_AND_WINDOW_TILE_DATA_SELECT) { 0x8000 } else { 0x8800 } }

    fn bg_tile_map_display_select(&self) -> u16 { if self.contains(Lcdc::BG_TILE_MAP_DISPLAY_SELECT) { 0x9c00 } else { 0x9800 } }

    fn obj_size(&self) -> u16 { if self.contains(Lcdc::OBJ_SIZE) { 16 } else { 8 } }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum TilePixelValue {
    Black = 0b00,
    LightGray = 0b01,
    DarkGray = 0b10,
    White = 0b11,
}

impl TilePixelValue {
    fn from_u8(value: u8) -> TilePixelValue {
        match value {
            0b00 => TilePixelValue::Black,
            0b01 => TilePixelValue::LightGray,
            0b10 => TilePixelValue::DarkGray,
            0b11 => TilePixelValue::White,
            _ => panic!("Unmapped color {:b}", value)
        }
    }

    fn to_rgb(&self) -> u32 {
        match self {
            TilePixelValue::Black => 0xff091820,
            TilePixelValue::LightGray => 0xff88C070,
            TilePixelValue::DarkGray => 0xff356856,
            TilePixelValue::White => 0xffE0F8D0,
        }
    }

    fn to_unicode(&self) -> char {
        match self {
            TilePixelValue::Black => '▓',
            TilePixelValue::LightGray => '░',
            TilePixelValue::DarkGray => '▒',
            TilePixelValue::White => ' '
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Stat {
    // Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
    enable_ly_interrupt: bool,
    // Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    enable_m2_interrupt: bool,
    // Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    enable_m1_interrupt: bool,
    // Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    enable_m0_interrupt: bool,
    mode: StatMode,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            enable_ly_interrupt: false,
            enable_m2_interrupt: false,
            enable_m1_interrupt: false,
            enable_m0_interrupt: false,
            mode: StatMode::Hblank,
        }
    }
}

pub struct GPU {
    lcdc: Lcdc,
    stat: Stat,
    vram: [u8; VRAM_SIZE],

    scy: u8,
    scx: u8,
    wy: u8,
    wx: u8,
    ly: u8,
    lc: u8,

    cycles: u32,
    dots: u32,
    pub intf: InterruptFlag,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum StatMode {
    Hblank = 0x00,
    Vblank = 0x01,
    Oam = 0x02,
    Transfer = 0x03,
}


impl GPU {
    pub fn new() -> GPU {
        GPU {
            lcdc: Lcdc::new(),
            stat: Stat::new(),
            vram: [0; VRAM_SIZE],

            scy: 0x00,
            scx: 0x00,
            wx: 0x00,
            wy: 0x00,
            ly: 0x00,
            lc: 0x00,

            cycles: 0,
            dots: 0,
            intf: InterruptFlag::empty(),
        }
    }

    pub fn next(&mut self, elapsed: u32) -> bool {
        if elapsed == 0 || !self.lcdc.lcd_display_enable() {
            return false;
        }

        let mut should_render = false;

        let c = (elapsed - 1) / 80 + 1;

        for i in 0..c {
            if i == (c - 1) {
                self.dots += elapsed % 80
            } else {
                self.dots += 80
            }
            let d = self.dots;
            self.dots %= 456;
            if d != self.dots {
                self.ly = (self.ly + 1) % 154;
                if self.stat.enable_ly_interrupt && self.ly == self.lc {
                    self.intf.insert(InterruptFlag::LCD_STAT);
                }
            }
            if self.ly >= 144 {
                if self.stat.mode == StatMode::Vblank {
                    continue;
                }
                self.stat.mode = StatMode::Vblank;
                // self.v_blank = true;
                should_render = true;
                self.intf.insert(InterruptFlag::V_BLANK);
                if self.stat.enable_m1_interrupt {
                    self.intf.insert(InterruptFlag::LCD_STAT);
                }
            } else if self.dots <= 80 {
                if self.stat.mode == StatMode::Oam {
                    continue;
                }
                self.stat.mode = StatMode::Oam;
                if self.stat.enable_m2_interrupt {
                    self.intf.insert(InterruptFlag::LCD_STAT);
                }
            } else if self.dots <= (80 + 172) {
                self.stat.mode = StatMode::Transfer;
            } else {
                if self.stat.mode == StatMode::Hblank {
                    continue;
                }
                self.stat.mode = StatMode::Hblank;
                // self.h_blank = true;
                if self.stat.enable_m0_interrupt {
                    self.intf.insert(InterruptFlag::LCD_STAT);
                }
                // Render scanline

                // Draw sprites
            }
        }
        return should_render;
    }


    pub fn read_vram(&self, address: u16) -> u8 {
        match address {
            0xff40 => self.lcdc.bits,
            0xff41 => {
                let bit6 = if self.stat.enable_ly_interrupt { 0x40 } else { 0x00 };
                let bit5 = if self.stat.enable_m2_interrupt { 0x20 } else { 0x00 };
                let bit4 = if self.stat.enable_m1_interrupt { 0x10 } else { 0x00 };
                let bit3 = if self.stat.enable_m0_interrupt { 0x08 } else { 0x00 };
                let bit2 = if self.ly == self.lc { 0x04 } else { 0x00 };
                bit6 | bit5 | bit4 | bit3 | bit2 | (self.stat.mode as u8)
            }
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lc,
            0xff4a => self.wx,
            0xff4b => self.wy,
            _ => self.vram[address as usize - VRAM_BEGIN]
        }
    }

    pub fn write_vram(&mut self, index: u16, value: u8) {
        match index {
            0xff40 => self.lcdc.bits = value,
            0xff41 => {
                self.stat.enable_ly_interrupt = value & 0x40 != 0x00;
                self.stat.enable_m2_interrupt = value & 0x20 != 0x00;
                self.stat.enable_m1_interrupt = value & 0x10 != 0x00;
                self.stat.enable_m0_interrupt = value & 0x08 != 0x00;
            }
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            0xff44 => self.ly = value,
            0xff45 => self.lc = value,
            0xff4a => self.wx = value,
            0xff4b => self.wy = value,
            _ => self.vram[(index as usize) - VRAM_BEGIN] = value,
        }
    }

    fn get_pixel_color(&self, tile_location: u16, tile_y: u8, tile_x: u8) -> u32 {
        let tile_y_data: [u8; 2] = {
            let a = self.read_vram(tile_location + (tile_y as u16 * 2));
            let b = self.read_vram(tile_location + (tile_y as u16 * 2) + 1);
            [a, b]
        };

        // Palettes
        let color_l = if tile_y_data[0] & (0x80 >> tile_x) != 0 { 1 } else { 0 };
        let color_h = if tile_y_data[1] & (0x80 >> tile_x) != 0 { 2 } else { 0 };
        let color = (color_h | color_l) as u8;

        TilePixelValue::from_u8(color).to_rgb()
    }

    fn get_tile_location(&self, x: u8, y: u8) -> u16 {
        let tile_base = self.lcdc.bg_and_window_tile_data_select();
        let tx = x as u16 / 8;
        let ty = y as u16 / 8;

        let bg_base = self.lcdc.bg_tile_map_display_select();
        let title_addr = bg_base + ty * 32 + tx;

        let tile_number = self.read_vram(title_addr);

        let tile_offset = if self.lcdc.contains(Lcdc::BG_AND_WINDOW_TILE_DATA_SELECT) {
            i16::from(tile_number)
        } else {
            i16::from(tile_number as i8) + 128
        } as u16
            * 16;

        tile_base + tile_offset
    }

    fn draw_tile_at(&self, x: u8, y: u8) -> u32 {
        let tile_location = self.get_tile_location(x, y);

        let tile_y = y % 8;
        let tile_x = x % 8;

        self.get_pixel_color(tile_location, tile_y, tile_x)
    }

    pub fn copy_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        let mut i = 0usize;
        for y in 0..144u16 {
            for x in 0..160u16 {
                buffer[i] = self.draw_tile_at(((x + self.scx as u16) % 256) as u8, ((y + self.scy as u16) % 256) as u8);
                i += 1;
            }
        }
    }

    pub fn debug_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        for i in 0usize..(144 / 8) * (160 / 8) {
            for y in 0..8 {
                for x in 0..8 {
                    buffer[(i * 8 % 160) + x + (y + i * 8 / 160 * 8) * 160] = self.get_pixel_color((0x8000 + i * 16) as u16, y as u8, x as u8)
                }
            }
        }
    }

    pub fn debug_print(&self) {
        for i in 0..0x20 {
            let from = i * 0x10;
            let buffer = &self.vram[from..(from + 0x10)];

            let absolute_addr = from + VRAM_BEGIN;
            if buffer.iter().any(|n| { *n != 0 }) {
                println!("{:x}: {:02x?}", absolute_addr, buffer);
            } else {
                println!("{:x}: ", absolute_addr);
            }
        }
    }
}

