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
const ROM_BEGIN: usize = 0x0000;
const ROM_END: usize = 0x7fff;

const ERAM_BEGIN: usize = 0xa000;
const ERAM_END: usize = 0xbfff;

const ERAM_SIZE: usize = ERAM_END - ERAM_BEGIN + 1;

const MEM_SIZE: usize = 0xffff + 1;

pub type RomBuffer = Vec<u8>;

pub struct MemoryBus {
    memory: [u8; MEM_SIZE],
    external_memory: [u8; ERAM_SIZE],
    boot_rom_disabled: bool,
    mbc: MBCWrapper,
    pub ppu: GPU,
    pub interrupt_enable: InterruptFlag,
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus {
            memory: [0x00; MEM_SIZE],
            external_memory: [0x00; ERAM_SIZE],
            mbc: MBCWrapper::default(),
            ppu: GPU::new(),
            boot_rom_disabled: false,
            interrupt_enable: InterruptFlag::empty(),
        }
    }
}

impl MemoryBus {
    pub fn new(bootloader: [u8; 256], rom: Option<RomBuffer>) -> MemoryBus {
        let mut memory = [0x00; MEM_SIZE];

        memory[..256].copy_from_slice(&bootloader);


        let mbc = rom.map(|r| MBCWrapper::new(r)).unwrap_or_default();


        MemoryBus {
            memory,
            mbc,
            external_memory: [0x00; ERAM_SIZE],
            ppu: GPU::new(),
            interrupt_enable: InterruptFlag::empty(),
            boot_rom_disabled: false,
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;

        match address {
            0x0000..=0x7fff => self.mbc.write_rom(address, value),
            0xa000..=0xbfff => self.mbc.write_ram(address, value),
            0xff46 => {
                self.dma_transfer(value);
            }
            VRAM_BEGIN..=VRAM_END => self.ppu.write_vram(addr, value),
            0xff50 => {
                self.boot_rom_disabled = value == 1;
            }
            0xffff => {
                self.interrupt_enable = InterruptFlag::from_bits_truncate(value);
                // eprintln!("++ interrupts_enabled: {:?}", self.interrupts_enabled);
            }
            _ => self.memory[address] = value,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let address = addr as usize;

        let val = match address {
            0x0000..=0x7fff => self.mbc.read_rom(address),
            0xa000..=0xbfff => self.mbc.read_ram(address),
            VRAM_BEGIN..=VRAM_END => self.ppu.read_vram(addr),
            0xffff => self.interrupt_enable.bits(),
            _ => self.memory[address]
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
        write!(f, "Bootloader: {:02x?}", &self.memory[..256])
    }
}



