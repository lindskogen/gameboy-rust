// 32K memory
// 0x0000 - 0x7FFF

// 0x0000 - 0x00FF: Destination address for RST instructions and starting address for interrupts
// 0x0100 - 0x014F: ROM area for storing data such as the name of the game
//          0x0150: Starting address of the user program

// 0x8000 - 0x9FFF: RAM for LCD display
//                  Only 8KB is used for DMG

use std::fmt;

use crate::dmg::gpu::{GPU, VRAM_BEGIN, VRAM_END};
use crate::dmg::intf::InterruptFlag;
use crate::dmg::mbc::MBCWrapper;

const WRAM_SIZE: usize = 0x8000;
const ZRAM_SIZE: usize = 0x7F;


pub type RomBuffer = Vec<u8>;

pub struct MemoryBus {
    wram: [u8; WRAM_SIZE],
    zram: [u8; ZRAM_SIZE],
    boot_rom_disabled: bool,
    mbc: MBCWrapper,
    boot_rom: [u8; 256],
    pub ppu: GPU,
    pub interrupt_enable: InterruptFlag,
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus {
            wram: [0x00; WRAM_SIZE],
            zram: [0x00; ZRAM_SIZE],
            mbc: MBCWrapper::default(),
            ppu: GPU::new(),
            boot_rom: [0x00; 256],
            boot_rom_disabled: false,
            interrupt_enable: InterruptFlag::empty(),
        }
    }
}

impl MemoryBus {
    pub fn new(bootloader: [u8; 256], rom: Option<RomBuffer>) -> MemoryBus {
        let mut boot_rom = [0x00; 256];

        boot_rom[..256].copy_from_slice(&bootloader);


        let mbc = rom.map(|r| MBCWrapper::new(r)).unwrap_or_default();


        MemoryBus {
            wram: [0x00; WRAM_SIZE],
            zram: [0x00; ZRAM_SIZE],
            mbc,
            boot_rom,
            boot_rom_disabled: false,
            ppu: GPU::new(),
            interrupt_enable: InterruptFlag::empty(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;

        match address {
            0x0000..=0x7fff => self.mbc.write_rom(address, value),
            0xc000..=0xcfff | 0xe000..=0xefff => self.wram[address & 0x0fff] = value,
            0xa000..=0xbfff => self.mbc.write_ram(address, value),
            0xff46 => {
                self.dma_transfer(value);
            }
            VRAM_BEGIN..=VRAM_END => self.ppu.write_vram(addr, value),
            0xff50 => {
                self.boot_rom_disabled = value == 1;
            }
            0xff80..=0xfffe => self.zram[address & 0x007f] = value,
            0xffff => {
                self.interrupt_enable = InterruptFlag::from_bits_truncate(value);
                // eprintln!("++ interrupts_enabled: {:?}", self.interrupts_enabled);
            }

            0xff4d |
            0xff7f => {}
            _ => unreachable!("Write to unmapped address: {:04X}", address)
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let address = addr as usize;

        if address < 0x100 && !self.boot_rom_disabled {
            return self.boot_rom[address];
        }

        let val = match address {
            0x0000..=0x7fff => self.mbc.read_rom(address),
            0xa000..=0xbfff => self.mbc.read_ram(address),
            0xc000..=0xcfff | 0xe000..=0xefff => self.wram[address & 0x0fff],
            VRAM_BEGIN..=VRAM_END => self.ppu.read_vram(addr),
            0xff80..=0xfffe => self.zram[address & 0x007f],
            0xff4d | 0xff4f | 0xff51..=0xff55 | 0xff6c | 0xff70 | 0xff7f => { 0xff }
            0xffff => self.interrupt_enable.bits(),
            _ => unreachable!("Read from unmapped address: {:04X}", address)
        };

        val
    }

    fn dma_transfer(&mut self, addr: u8) {
        let address_block: u16 = (addr as u16) << 8;
        for i in 0..=0x9f {
            self.write_byte(0xfe00 + i, self.read_byte(address_block + i));
        }
    }
}

impl fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bootloader: {:02x?}", &self.boot_rom[..256])
    }
}



