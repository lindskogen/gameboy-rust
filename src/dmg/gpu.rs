use bitflags::bitflags;
use std::fmt::Debug;
use std::usize;

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
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
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
        }
    }


    fn set_stat_lyc(&mut self, value: bool) {
        self.stat.enable_ly_interrupt = value;
    }

    pub fn next(&mut self, elapsed: u32) -> bool {
        self.cycles += elapsed;

        if !self.lcdc.lcd_display_enable() {
            return false;
        }

        let mut should_render = false;

        let mut interrupt_vblank = false;

        match self.stat.mode {
            StatMode::Hblank => {
                if self.cycles >= 204 {
                    self.cycles -= 204;
                    self.ly += 1;
                    if self.ly == 144 {
                        self.stat.mode = StatMode::Vblank;
                        interrupt_vblank = true;
                        should_render = true;
                    } else {
                        self.stat.mode = StatMode::Oam;
                    }
                }
            }
            StatMode::Vblank => {
                if self.cycles >= 456 {
                    self.cycles -= 456;
                    self.ly += 1;
                    if self.ly > 153 {
                        self.ly = 0;
                        self.stat.mode = StatMode::Oam;
                    }
                }
            }
            StatMode::Oam => {
                if self.cycles >= 80 {
                    self.cycles -= 80;
                    self.stat.mode = StatMode::Hblank;
                }
            }
            StatMode::Transfer => {
                if self.cycles >= 172 {
                    self.cycles -= 172;
                    // draw scanline
                    self.stat.mode = StatMode::Hblank;
                }
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

    fn draw_tile_at(&self, x: u8, y: u8) -> u32 {
        let tile_base = self.lcdc.bg_and_window_tile_data_select();
        let tx = (u16::from(x) >> 3) & 31;
        let ty = (u16::from(y) >> 3) & 31;

        let bg_base = self.lcdc.bg_tile_map_display_select();
        let title_addr = bg_base + ty * 32 + tx;

        let tile_number = self.read_vram(title_addr);

        let tile_offset = if self.lcdc.contains(Lcdc::BG_AND_WINDOW_TILE_DATA_SELECT) {
            i16::from(tile_number)
        } else {
            i16::from(tile_number as i8) + 128
        } as u16
            * 16;

        let tile_location = tile_base + tile_offset;

        let tile_y = y % 8;

        let tile_y_data: [u8; 2] = {
            let a = self.read_vram(tile_location + u16::from(tile_y * 2));
            let b = self.read_vram(tile_location + u16::from(tile_y * 2) + 1);
            [a, b]
        };

        let tile_x = x % 8;

        // Palettes
        let color_l = if tile_y_data[0] & (0x80 >> tile_x) != 0 { 1 } else { 0 };
        let color_h = if tile_y_data[1] & (0x80 >> tile_x) != 0 { 2 } else { 0 };
        let color = (color_h | color_l) as u8;

        TilePixelValue::from_u8(color).to_rgb()
    }

    pub fn copy_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        let render_debug_grid = false;
        let mut i = 0usize;
        for y in 0..144u16 {
            for x in 0..160u16 {
                buffer[i] = self.draw_tile_at(((x + self.scx as u16) % 160) as u8, ((y + self.scy as u16) % 144) as u8);
                if render_debug_grid && (x % 8 == 0 || y % 8 == 0) {
                    buffer[i] = buffer[i] >> 4;
                }
                i += 1;
            }
        }
    }

    pub fn debug_print(&self) {
        for i in 0..100 {
            let from = i * 0x10;
            let buffer = &self.vram[from..(from + 0x10)];

            if buffer.iter().any(|n| { *n != 0 }) {
                println!("{:x}: {:02x?}", from + 0x8000, buffer);
            }
        }
    }
}


fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}
