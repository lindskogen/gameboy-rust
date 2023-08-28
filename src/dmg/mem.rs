// 32K memory
// 0x0000 - 0x7FFF

// 0x0000 - 0x00FF: Destination address for RST instructions and starting address for interrupts
// 0x0100 - 0x014F: ROM area for storing data such as the name of the game
//          0x0150: Starting address of the user program

// 0x8000 - 0x9FFF: RAM for LCD display
//                  Only 8KB is used for DMG

use std::fmt;

use crate::dmg::gpu::GPU;
use crate::dmg::input::Joypad;
use crate::dmg::intf::InterruptFlag;
use crate::dmg::mbc::MBCWrapper;
use crate::dmg::serial::Serial;
use serde::{Serialize, Deserialize};
use crate::dmg::sound::apu::Apu;
use crate::dmg::sound::traits::Mem;

const WRAM_SIZE: usize = 0x8000;
const ZRAM_SIZE: usize = 0x7F;


pub type RomBuffer = Vec<u8>;

#[derive(Serialize, Deserialize)]
pub struct MemoryBus {
    #[serde(with = "serde_arrays")]
    wram: [u8; WRAM_SIZE],
    #[serde(with = "serde_arrays")]
    zram: [u8; ZRAM_SIZE],
    boot_rom_disabled: bool,
    mbc: MBCWrapper,
    serial: Serial,
    wram_bank: usize,

    #[serde(with = "serde_arrays")]
    boot_rom: [u8; 256],
    pub input: Joypad,
    pub ppu: GPU,

    #[serde(skip)]
    pub apu: Apu,
    pub interrupt_enable: InterruptFlag,
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus {
            wram: [0x00; WRAM_SIZE],
            zram: [0x00; ZRAM_SIZE],
            wram_bank: 1,
            serial: Serial::default(),
            mbc: MBCWrapper::default(),
            ppu: GPU::new(),
            apu: Apu::default(),
            boot_rom: [0x00; 256],
            input: Joypad::default(),
            boot_rom_disabled: false,
            interrupt_enable: InterruptFlag::empty(),
        }
    }
}

impl MemoryBus {
    pub fn new_without_boot_rom(rom: Option<RomBuffer>) -> MemoryBus {
        let mut bus = Self::new(None, rom);

        // Initialize LCDC
        bus.write_byte(0xff40, 0x91);

        bus
    }
    pub fn new(bootloader: Option<[u8; 256]>, rom: Option<RomBuffer>) -> MemoryBus {
        let mut boot_rom = [0x00; 256];

        if let Some(rom) = bootloader {
            boot_rom[..256].copy_from_slice(&rom);
        }


        let mbc = rom.map(|r| MBCWrapper::new(r)).unwrap_or_default();


        MemoryBus {
            wram: [0x00; WRAM_SIZE],
            zram: [0x00; ZRAM_SIZE],
            wram_bank: 1,
            mbc,
            serial: Serial::default(),
            boot_rom,
            boot_rom_disabled: bootloader.is_none(),
            input: Joypad::default(),
            ppu: GPU::new(),
            apu: Apu::default(),
            interrupt_enable: InterruptFlag::empty(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;

        match address {
            0x0000..=0x7fff => self.mbc.write_rom(address, value),
            0xc000..=0xcfff | 0xe000..=0xefff => self.wram[address & 0x0fff] = value,
            0xd000..=0xdfff | 0xf000..=0xfdff => self.wram[(self.wram_bank * 0x1000) | address & 0x0fff] = value,
            0xff4d | 0xff7f => {}
            0xff00 => self.input.write(value),
            0xff01..=0xff02 => self.serial.write(addr, value),
            0xa000..=0xbfff => self.mbc.write_ram(address, value),
            0x8000..=0x9fff => self.ppu.write_vram(addr, value),
            0xfe00..=0xfe9f => self.ppu.write_vram(addr, value),
            0xff46 => self.dma_transfer(value),
            0xff40..=0xff4f => self.ppu.write_vram(addr, value),
            0xff68..=0xff6b => self.ppu.write_vram(addr, value),
            0xff04..=0xff07 => self.ppu.write_vram(addr, value),
            0xff10..=0xff3f => self.apu.write(addr, value),
            0xff0f => self.ppu.write_vram(addr, value), // TODO: move interrupt flags here
            0xff50 => {
                self.boot_rom_disabled = value == 1;
            }
            0xfea0..=0xfeff => { /* Unusable */ }
            0xff80..=0xfffe => self.zram[address & 0x007f] = value,
            0xffff => {
                self.interrupt_enable = InterruptFlag::from_bits_truncate(value);
                // eprintln!("++ interrupts_enabled: {:?}", self.interrupts_enabled);
            }

            _ => unreachable!("MEM: Write to unmapped address: {:04X}", address)
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
            0xd000..=0xdfff | 0xf000..=0xfdff => self.wram[(self.wram_bank * 0x1000) | address & 0x0fff],
            0xff4d | 0xff4f | 0xff51..=0xff55 | 0xff6c | 0xff70 | 0xff7f => { 0xff }
            0xff00 => { self.input.read() }
            0xff01..=0xff02 => self.serial.read(addr),
            0x8000..=0x9fff => self.ppu.read_vram(addr),
            0xfe00..=0xfe9f => self.ppu.read_vram(addr),
            0xff40..=0xff4f => self.ppu.read_vram(addr),
            0xff68..=0xff6b => self.ppu.read_vram(addr),
            0xff04..=0xff07 => self.ppu.read_vram(addr),
            0xff10..=0xff3f => self.apu.read(addr),
            0xfea0..=0xfeff => { /* Unusable */ 0xff }
            0xff80..=0xfffe => self.zram[address & 0x007f],
            0xff0f => self.ppu.read_vram(addr), // TODO: move interrupt flags here
            0xffff => self.interrupt_enable.bits(),
            _ => unreachable!("MEM: Read from unmapped address: {:04X}", address)
        };

        val
    }

    fn dma_transfer(&mut self, addr: u8) {
        let address_block: u16 = (addr as u16) << 8;
        for i in 0..=0x9f {
            self.write_byte(0xfe00 + i, self.read_byte(address_block + i));
        }
    }

    pub fn check_interrupt(&self) -> bool {
        self.interrupt_enable
            .intersects(self.ppu.interrupt_flag)
    }
}

impl fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bootloader: {:02x?}", &self.boot_rom[..256])
    }
}



