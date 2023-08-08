use std::iter;
use crate::dmg::mem::RomBuffer;

#[derive(Debug, Copy, Clone)]
enum MBC {
    NoMbc,
    Mbc1,
    Mbc1ExternalRam,
    Mbc1BatteryExternalRam,
    Mbc2,
    Mbc2ExternalRam,
    RomExternatRam,
    RomBatteryExternatRam,
}

impl Default for MBC {
    fn default() -> Self {
        MBC::NoMbc
    }
}

impl TryFrom<u8> for MBC {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(MBC::NoMbc),
            0x01 => Ok(MBC::Mbc1),
            0x02 => Ok(MBC::Mbc1ExternalRam),
            0x03 => Ok(MBC::Mbc1BatteryExternalRam),
            0x05 => Ok(MBC::Mbc2),
            0x06 => Ok(MBC::Mbc2ExternalRam),
            0x08 => Ok(MBC::RomExternatRam),
            0x09 => Ok(MBC::RomBatteryExternatRam),
            _ => Err(()),
        }
    }
}


struct MBC0 {
    rom: RomBuffer,
}

impl MBC0 {
    fn new(rom: RomBuffer) -> Self {
        Self {
            rom
        }
    }
}

#[derive(Eq, PartialEq)]
enum MBC1Mode {
    RomMode,
    RamMode,
}


impl TryFrom<u8> for MBC1Mode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0x01 {
            0x00 => Ok(MBC1Mode::RomMode),
            0x01 => Ok(MBC1Mode::RamMode),
            _ => Err(()),
        }
    }
}

struct MBC1 {
    rom: RomBuffer,
    ram: Vec<u8>,

    // Selected ROM bank
    rom_bank: usize,

    // Selected RAM bank
    ram_bank: usize,

    // RAM enable switch
    ram_on: bool,

    num_rom_banks: usize,
    num_ram_banks: usize,

    // ROM/RAM expansion mode
    mode: MBC1Mode,
}


impl MBC1 {
    fn new(rom: RomBuffer) -> Self {
        let num_rom_banks = rom_banks(*rom.get(0x148).unwrap_or(&0u8));

        let num_ram_banks = ram_banks(*rom.get(0x0149).unwrap_or(&0u8));
        let ram_size = num_ram_banks * 0x2000;

        if num_ram_banks > 0 {
            eprintln!("RAM size: {}", num_ram_banks);
        }

        Self {
            rom,
            ram: iter::repeat(0u8).take(ram_size).collect(),
            mode: MBC1Mode::RomMode,
            ram_on: false,
            rom_bank: 1,
            ram_bank: 0,
            num_ram_banks,
            num_rom_banks,

        }
    }

    pub fn read_rom(&self, addr: usize) -> u8 {
        let bank = if addr < 0x4000 {
            if self.mode == MBC1Mode::RomMode {
                self.rom_bank & 0xe0
            } else { 0 }
        } else {
            self.rom_bank
        };
        let idx = bank * 0x4000 | (addr & 0x3fff);

        *self.rom.get(idx).unwrap_or(&0xff)
    }

    pub fn read_ram(&self, addr: usize) -> u8 {
        if !self.ram_on { return 0xff; }
        let bank = if self.mode == MBC1Mode::RamMode { self.ram_bank } else { 0 };

        self.ram[(bank * 0x2000) | (addr & 0x1fff)]
    }

    pub fn write_ram(&mut self, addr: usize, value: u8) {
        if self.ram_on {
            let bank = if self.mode == MBC1Mode::RamMode { self.ram_bank } else { 0 };
            let idx = (bank * 0x2000) | (addr & 0x1fff);

            if idx < self.ram.len() {
                self.ram[idx] = value;
            }
        }
    }

    pub fn write_rom(&mut self, addr: usize, value: u8) {
        match addr {
            0x0000..=0x1fff => {
                self.ram_on = value & 0xf == 0xa;
            }
            0x2000..=0x3fff => {
                let lower_bits = match (value as usize) & 0x1f {
                    0 => 1,
                    n => n,
                };
                self.rom_bank = ((self.rom_bank & 0x60) | lower_bits) % self.num_rom_banks
            }
            0x4000..=0x5fff => {
                if self.num_rom_banks > 0x20 {
                    let upper_bits = (value as usize & 0x03) % (self.num_rom_banks >> 5);
                    self.rom_bank = (self.rom_bank & 0x1f) | (upper_bits << 5);
                }
                if self.num_ram_banks > 1 {
                    self.ram_bank = (value as usize) & 0x03;
                }
            }
            0x6000..=0x7fff => {
                self.mode = MBC1Mode::try_from(value).expect("Set unexpected bank_mode");
            }
            _ => unreachable!("MBC1 invalid address, {:04X}", addr)
        }
    }
}


enum MBCType {
    Mbc0(MBC0),
    Mbc1(MBC1),
}

pub struct MBCWrapper {
    variant: MBCType,
}

impl Default for MBCWrapper {
    fn default() -> Self {
        Self {
            variant: MBCType::Mbc0(MBC0::new(iter::repeat(0x00).take(8000).collect()))
        }
    }
}

fn ram_banks(v: u8) -> usize {
    match v {
        1 | 2 => 1,
        3 => 4,
        4 => 16,
        5 => 8,
        _ => 0,
    }
}

fn rom_banks(v: u8) -> usize {
    if v <= 8 {
        2 << v
    } else {
        0
    }
}

impl MBCWrapper {
    pub fn new(rom: RomBuffer) -> Self {
        let mbc = rom.get(0x147).and_then(|&v| v.try_into().ok()).unwrap_or_default();


        match mbc {
            MBC::NoMbc => {
                Self {
                    variant: MBCType::Mbc0(MBC0::new(rom))
                }
            }
            MBC::Mbc1 | MBC::Mbc1BatteryExternalRam | MBC::Mbc1ExternalRam => {
                Self {
                    variant: MBCType::Mbc1(MBC1::new(rom))
                }
            }
            _ => panic!("No support for cartridge type: {:?}", mbc),
        }
    }

    pub fn read_rom(&self, addr: usize) -> u8 {
        match self.variant {
            MBCType::Mbc0(MBC0 { ref rom }) => rom[addr],
            MBCType::Mbc1(ref m) => m.read_rom(addr),
        }
    }


    pub fn read_ram(&self, addr: usize) -> u8 {
        match self.variant {
            MBCType::Mbc0(_) => 0x00,
            MBCType::Mbc1(ref m) => m.read_ram(addr)
        }
    }

    pub fn write_ram(&mut self, addr: usize, value: u8) {
        match &mut self.variant {
            MBCType::Mbc0(_) => {}
            MBCType::Mbc1(ref mut a) => a.write_ram(addr, value)
        }
    }

    pub fn write_rom(&mut self, addr: usize, value: u8) {
        match &mut self.variant {
            MBCType::Mbc0(_) => {}
            MBCType::Mbc1(ref mut a) => a.write_rom(addr, value)
        }
    }
}
