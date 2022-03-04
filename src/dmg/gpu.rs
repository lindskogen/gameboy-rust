pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;


#[derive(Copy, Clone)]
#[repr(u8)]
enum TilePixelValue {
    Black = 0b00,
    LightGray = 0b01,
    DarkGray = 0b10,
    White = 0b11,
}

impl TilePixelValue {
    fn to_rgb(&self) -> u32 {
        match self {
            TilePixelValue::Black => 0x00000000,
            TilePixelValue::LightGray => 0xccccccff,
            TilePixelValue::DarkGray => 0x999999ff,
            TilePixelValue::White => 0xffffffff,
        }
    }
}

type Tile = [[TilePixelValue; 8]; 8];

fn empty_tile() -> Tile {
    [[TilePixelValue::Black; 8]; 8]
}


pub struct GPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
}


impl GPU {
    pub fn new() -> GPU {
        GPU {
            vram: [0; VRAM_SIZE],
            tile_set: [empty_tile(); 384],
        }
    }


    pub fn read_vram(&self, address: usize) -> &u8 {
        &self.vram[address]
    }

    pub fn write_vram(&mut self, index: usize, value: u8) {
        self.vram[index] = value;

        if index >= 0x1800 {
            return;
        }

        let normalized_index = index & 0xFFFE;

        let byte1 = self.vram[normalized_index];
        let byte2 = self.vram[normalized_index + 1];

        let tile_index = index / 16;
        let row_index = (index % 16) / 2;


        for pixel_index in 0..8 {
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;

            let value = match (lsb != 0, msb != 0) {
                (true, true) => TilePixelValue::Black,
                (false, true) => TilePixelValue::DarkGray,
                (true, false) => TilePixelValue::LightGray,
                (false, false) => TilePixelValue::White,
            };

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
    }

    pub fn copy_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        for i in 0..8 {
            for x in 0..8 {
                for y in 0..8 {
                    let c = self.tile_set[i][x][y];
                    buffer[i * 8 + x + y * 160] = c.to_rgb();
                }
            }
        }
    }
}


fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}
