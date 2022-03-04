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

    fn draw_tile_at(&self, buffer: &mut Vec<u32>, x: u8, y: u8) {
        let map_offset = 0;
        let tile_x = x / 8;
        let tile_y = y / 8;

        let title_id_addr = map_offset + tile_y.wrapping_mul(32).wrapping_add(tile_x);

        let tile_map_id = self.vram[title_id_addr as usize] as u16;

        let addr: usize = (tile_map_id as usize) * 16;


        let dx: u8 = 7 - (x % 8);
        let dy: u16 = y.wrapping_mul(8).wrapping_mul(2) as u16;

        let a = self.vram[addr + dy as usize];
        let b = self.vram[addr + (dy as usize) + 1];

        let lsb = (a & (1 << dx)) >> dx;
        let msb = (b & (1 << dx)) >> dx;

        let tile_data = match (lsb != 0, msb != 0) {
            (true, true) => TilePixelValue::Black,
            (false, true) => TilePixelValue::DarkGray,
            (true, false) => TilePixelValue::LightGray,
            (false, false) => TilePixelValue::White,
        };

        buffer[(x as usize) + ((y as usize * 144))] = tile_data.to_rgb();

    }

    pub fn copy_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        for x in 0..144 {
            for y in 0..160 {
                self.draw_tile_at(buffer, x, y);
            }
        }
    }
}


fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}
