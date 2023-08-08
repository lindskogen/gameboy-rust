use std::fmt::Debug;
use std::usize;
use bit_field::BitField;

use bitflags::bitflags;

use crate::dmg::intf::InterruptFlag;

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9fff;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;
pub const OAM_SIZE: usize = 0xA0;

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
    fn new() -> Self {
        Self { bits: 0b0100_1000 }
    }

    fn lcd_display_enable(&self) -> bool {
        self.contains(Lcdc::LCD_DISPLAY_ENABLE)
    }

    fn bg_and_window_tile_data_select(&self) -> u16 {
        if self.contains(Lcdc::BG_AND_WINDOW_TILE_DATA_SELECT) {
            0x8000
        } else {
            0x8800
        }
    }

    fn bg_and_window_display_enable(&self) -> bool {
        self.contains(Lcdc::BG_AND_WINDOW_DISPLAY)
    }

    fn bg_tile_map_display_select(&self) -> u16 {
        if self.contains(Lcdc::BG_TILE_MAP_DISPLAY_SELECT) {
            0x9c00
        } else {
            0x9800
        }
    }

    fn obj_display_enable(&self) -> bool {
        self.contains(Lcdc::OBJ_DISPLAY_ENABLE)
    }

    fn obj_size(&self) -> u16 {
        if self.contains(Lcdc::OBJ_SIZE) {
            16
        } else {
            8
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum TilePixelValue {
    Black = 0b11,
    LightGray = 0b01,
    DarkGray = 0b10,
    White = 0b00,
}

impl TilePixelValue {
    fn from_palette_and_u8(palette: u8, value: u8) -> TilePixelValue {
        match palette >> (2 * value) & 0x03 {
            0b00 => TilePixelValue::White,
            0b01 => TilePixelValue::LightGray,
            0b10 => TilePixelValue::DarkGray,
            _ => TilePixelValue::Black,
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
            mode: StatMode::HBlank0,
        }
    }
}

pub struct GPU {
    lcdc: Lcdc,
    stat: Stat,
    vram: [u8; VRAM_SIZE],

    oam: [u8; OAM_SIZE],

    vram_bank: usize,

    scy: u8,
    scx: u8,
    wy: u8,
    wx: u8,
    ly: u8,
    lc: u8,
    bgp: u8,
    pal0: u8,
    pal1: u8,

    /** FF04 - DIV - Divider Register (R/W) */
    div: u8,
    /** FF05 - TIMA - Timer counter (R/W) */
    tima: u8,

    /** FF06 - TMA - Timer Modulo (R/W) */
    tma: u8,

    /** FF07 - TAC - Timer Control (R/W) */
    tac: u8,

    cycles: u32,
    timer_cycles: u32,
    timer_clock: u32,
    enable_debug_override: bool,
    pub interrupt_flag: InterruptFlag,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum StatMode {
    HBlank0 = 0x00,
    VBlank1 = 0x01,
    OamRead2 = 0x02,
    Transfer3 = 0x03,
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            lcdc: Lcdc::new(),
            stat: Stat::new(),
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],

            vram_bank: 0,

            scy: 0x00,
            scx: 0x00,
            wx: 0x00,
            wy: 0x00,
            ly: 0x00,
            lc: 0x00,
            bgp: 0x00,
            pal0: 0x00,
            pal1: 0x00,

            div: 0x00,

            tima: 0x00,
            tma: 0x00,
            tac: 0x00,
            enable_debug_override: false,

            cycles: 0,
            timer_cycles: 0,
            timer_clock: 0,
            interrupt_flag: InterruptFlag::empty(),
        }
    }

    pub fn initialize_gameboy_doctor(&mut self) {
        self.enable_debug_override = true;
    }

    fn handle_timer(&mut self, cycles: u32) {
        self.cycles = self.cycles.wrapping_add(cycles);

        self.timer_cycles += cycles;


        if self.timer_cycles >= 256 {
            self.div = self.div.wrapping_add(1);
            self.timer_cycles -= 256;
        }

        let timer_enabled = (self.tac >> 2 & 0x01) != 0;

        if timer_enabled {
            self.timer_clock = self.timer_clock.wrapping_add(cycles * 4);

            let timer_freq = match self.tac & 0b11 {
                1 => 262144,
                2 => 65536,
                3 => 16384,
                _ => 4096
            };

            while self.timer_clock >= (4194304 / timer_freq) {
                let tima = self.tima;
                let (new_tima, tima_overflow) = tima.overflowing_add(1);
                self.tima = new_tima;
                if tima_overflow {
                    self.interrupt_flag.insert(InterruptFlag::TIMER);
                    self.tima = self.tma;
                }
                self.timer_clock -= 4194304 / timer_freq;
            }
        }
    }

    pub fn next(&mut self, elapsed: u32, buffer: &mut Vec<u32>) -> bool {
        self.handle_timer(elapsed);


        if !self.lcdc.lcd_display_enable() {
            return false;
        }


        let mut should_render = false;

        match self.stat.mode {
            StatMode::OamRead2 => {
                if self.cycles >= 80 {
                    self.cycles = 80;
                    self.stat.mode = StatMode::Transfer3;
                }
            }
            StatMode::Transfer3 => {
                if self.cycles >= 172 {
                    self.cycles = 0;
                    self.stat.mode = StatMode::HBlank0;
                    if self.stat.enable_m0_interrupt {
                        self.interrupt_flag.insert(InterruptFlag::LCD_STAT);
                    }
                    self.render_line_into_buffer(buffer);
                }
            }
            StatMode::HBlank0 => {
                if self.cycles >= 204 {
                    self.cycles = 0;
                    self.ly += 1;

                    if self.stat.enable_ly_interrupt && self.ly == self.lc {
                        self.interrupt_flag.insert(InterruptFlag::LCD_STAT);
                    }

                    if self.ly == 143 {
                        self.stat.mode = StatMode::VBlank1;
                        self.interrupt_flag.insert(InterruptFlag::V_BLANK);
                        // eprintln!("!! Interrupt {:?}", InterruptFlag::V_BLANK);
                        if self.stat.enable_m1_interrupt {
                            self.interrupt_flag.insert(InterruptFlag::LCD_STAT);
                        }
                        should_render = true;
                    } else {
                        self.stat.mode = StatMode::OamRead2;
                        if self.stat.enable_m2_interrupt {
                            self.interrupt_flag.insert(InterruptFlag::LCD_STAT);
                        }
                    }
                }
            }
            StatMode::VBlank1 => {
                if self.cycles >= 456 {
                    self.cycles = 0;
                    self.ly += 1;

                    if self.stat.enable_ly_interrupt && self.ly == self.lc {
                        self.interrupt_flag.insert(InterruptFlag::LCD_STAT);
                    }

                    if self.ly > 153 {
                        self.interrupt_flag.remove(InterruptFlag::V_BLANK);
                        self.stat.mode = StatMode::OamRead2;
                        if self.stat.enable_m2_interrupt {
                            self.interrupt_flag.insert(InterruptFlag::LCD_STAT);
                        }
                        self.ly = 0;
                    }
                }
            }
        }
        return should_render;
    }

    /// LY should be set to 0 when the LCD is off.
    fn read_ly(&self) -> u8 {
        if self.enable_debug_override {
            // Hard coded value for gameboy-doctor
            0x90
        } else if !self.lcdc.lcd_display_enable() {
            0
        } else {
            self.ly
        }
    }

    /// While the LCD is disabled, the mode flag (bits 1-0 of STAT) is set to 0, indicating that it is safe to write to all parts of RAM.
    fn read_stat_mode(&self) -> StatMode {
        if !self.lcdc.lcd_display_enable() {
            StatMode::HBlank0
        } else {
            self.stat.mode
        }
    }

    pub fn read_vram(&self, adr: u16) -> u8 {
        let address = adr as usize;

        match address {
            VRAM_BEGIN..=VRAM_END => self.vram[(self.vram_bank * 0x2000) | (address & 0x1fff)],
            0xfe00..=0xfe9f => self.oam[address - 0xfe00],
            0xff40 => self.lcdc.bits,
            0xff41 => {
                let bit6 = if self.stat.enable_ly_interrupt {
                    0x40
                } else {
                    0x00
                };
                let bit5 = if self.stat.enable_m2_interrupt {
                    0x20
                } else {
                    0x00
                };
                let bit4 = if self.stat.enable_m1_interrupt {
                    0x10
                } else {
                    0x00
                };
                let bit3 = if self.stat.enable_m0_interrupt {
                    0x08
                } else {
                    0x00
                };
                let bit2 = if self.ly == self.lc { 0x04 } else { 0x00 };
                let mode = self.read_stat_mode() as u8;

                bit6 | bit5 | bit4 | bit3 | bit2 | mode
            }
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.read_ly(),
            0xff45 => self.lc,
            0xff47 => self.bgp,
            0xff48 => self.pal0,
            0xff49 => self.pal1,
            0xff4a => self.wx,
            0xff4b => self.wy,
            0xff4f => self.vram_bank as u8 | 0xfe,
            0xff04 => self.div as u8,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            0xff0f => self.interrupt_flag.bits(),
            _ => unreachable!("MEM: Read from unmapped address: {:04X}", address)
            // _ => { 0xff },
        }
    }

    pub fn write_vram(&mut self, adr: u16, value: u8) {
        let address = adr as usize;

        match address {
            VRAM_BEGIN..=VRAM_END => self.vram[(self.vram_bank * 0x2000) | (address & 0x1fff)] = value,
            0xfe00..=0xfe9f => self.oam[address - 0xfe00] = value,
            0xff40 => self.lcdc = Lcdc::from_bits_truncate(value),
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
            0xff47 => self.bgp = value,
            0xff48 => self.pal0 = value,
            0xff49 => self.pal1 = value,
            0xff4a => self.wx = value,
            0xff4b => self.wy = value,
            0xff4f => self.vram_bank = (value & 0x01) as usize,
            0xff04 => self.div = 0, // Write to ff04 resets it to 0
            0xff05 => self.tima = value,
            0xff06 => self.tma = value,
            0xff07 => self.tac = value,

            0xff0f => self.interrupt_flag = InterruptFlag::from_bits_truncate(value),
            _ => unreachable!("PPU: Write to unmapped address: {:04X}", address)
        }
    }

    fn get_pixel_color(&self, tile_location: u16, tile_y: u8, tile_x: u8) -> TilePixelValue {
        let tile_y_data: [u8; 2] = {
            let a = self.read_vram(tile_location + (tile_y as u16 * 2));
            let b = self.read_vram(tile_location + (tile_y as u16 * 2) + 1);
            [a, b]
        };

        // Palettes
        let color_l = if tile_y_data[0] & (0x80 >> tile_x) != 0 {
            1
        } else {
            0
        };
        let color_h = if tile_y_data[1] & (0x80 >> tile_x) != 0 {
            2
        } else {
            0
        };
        let color = (color_h | color_l) as u8;

        TilePixelValue::from_palette_and_u8(self.bgp, color)
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

    fn draw_tile_at(&self, x: u8, y: u8) -> TilePixelValue {
        let tile_location = self.get_tile_location(x, y);

        let tile_y = y % 8;
        let tile_x = x % 8;

        self.get_pixel_color(tile_location, tile_y, tile_x)
    }

    fn render_line_into_buffer(&self, buffer: &mut Vec<u32>) {
        let y = self.ly as u16;

        let (sprites_to_draw, len) = self.populate_sprites_to_render(y);

        for x in 0..160u16 {
            let index = y as usize * 160 + x as usize;

            let mut tile_pixel_color = TilePixelValue::White;

            if self.lcdc.bg_and_window_display_enable() {
                tile_pixel_color = self.draw_tile_at(
                    ((x + self.scx as u16) % 256) as u8,
                    ((y + self.scy as u16) % 256) as u8,
                );
            }


            if self.lcdc.obj_display_enable() && len > 0 {
                if let Some(color) = self.draw_sprite_at(&sprites_to_draw[..len], x as u8, y as u8, tile_pixel_color == TilePixelValue::White) {
                    tile_pixel_color = color;
                }
            }

            buffer[index] = tile_pixel_color.to_rgb();
        }
    }

    fn populate_sprites_to_render(&self, line: u16) -> ([(i32, i32, u16); 10], usize) {
        let mut sprites = [(0, 0, 0); 10];
        let mut index = 0usize;
        let line = line as i32;
        let sprite_size = self.lcdc.obj_size() as i32;


        for i in 0..40u16 {
            let addr = 0xfe00 + (i * 4);
            let sprite_y = self.read_vram(addr + 0) as u16 as i32 - 16;

            if line < sprite_y || line >= sprite_y + sprite_size {
                continue;
            }

            let sprite_x = self.read_vram(addr + 1) as u16 as i32 - 8;
            sprites[index] = (sprite_x, sprite_y, i);
            index += 1;
            if index >= 10 {
                break;
            }
        }

        if index > 0 {
            sprites[..index].sort_by(|a, b| {
                a.0.cmp(&b.0)
            })
        }


        (sprites, index)
    }

    fn draw_sprite_at(&self, sprites: &[(i32, i32, u16)], x: u8, y: u8, bg_color_is_white: bool) -> Option<TilePixelValue> {
        let sprite_size = self.lcdc.obj_size();
        for &(sprite_x, sprite_y, i) in sprites {
            let tile_x = x as i32 - sprite_x;

            if tile_x < 0 || tile_x > 7 {
                continue;
            }

            let addr = 0xfe00 + i * 4;
            let tile_num = (self.read_vram(addr + 2) as u16) & (if sprite_size == 16 { 0xfe } else { 0xff } as u16);
            let flags = self.read_vram(addr + 3);
            let use_pal1 = flags.get_bit(4);
            let x_flip = flags.get_bit(5);
            let y_flip = flags.get_bit(6);
            let behind_non_white_bg = flags.get_bit(7);

            if y as i32 - sprite_y > (sprite_size as i32 - 1) { continue; }


            let tile_y = if y_flip {
                (sprite_size - 1) - (y as i32 - sprite_y) as u16
            } else {
                (y as i32 - sprite_y) as u16
            };

            let tile_addr: u16 = 0x8000 + tile_num * 16 + tile_y * 2;

            let (b1, b2) = (self.read_vram(tile_addr), self.read_vram(tile_addr + 1));


            let x_bit = 1 << (if x_flip { tile_x } else { 7 - tile_x } as u32);

            let color = (if b1 & x_bit != 0 { 1 } else { 0 }) | (if b2 & x_bit != 0 { 2 } else { 0 });

            if color == 0 {
                return None;
            }

            if !bg_color_is_white && behind_non_white_bg {
                return None
            }

            let palette = if use_pal1 { self.pal1 } else { self.pal0 };

            return Some(TilePixelValue::from_palette_and_u8(palette, color));
        }

        return None;
    }

    pub fn debug_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        for i in 0usize..(144 / 8) * (160 / 8) {
            for y in 0..8 {
                for x in 0..8 {
                    buffer[(i * 8 % 160) + x + (y + i * 8 / 160 * 8) * 160] =
                        self.get_pixel_color((0x8000 + i * 16) as u16, y as u8, x as u8).to_rgb()
                }
            }
        }
    }

    pub fn debug_print(&self) {
        for i in 0..0x20 {
            let from = i * 0x10;
            let buffer = &self.vram[from..(from + 0x10)];

            let absolute_addr = from + VRAM_BEGIN;
            if buffer.iter().any(|n| *n != 0) {
                println!("{:x}: {:02x?}", absolute_addr, buffer);
            } else {
                println!("{:x}: ", absolute_addr);
            }
        }
    }
}

