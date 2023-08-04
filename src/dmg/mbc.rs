use std::iter;
use crate::dmg::mem::RomBuffer;

#[derive(Debug, Copy, Clone)]
enum MBC {
    NoMBC,
    MBC1,
    Mbc1ExternalRam,
    Mbc1BatteryExternalRam,
}

impl Default for MBC {
    fn default() -> Self {
        MBC::NoMBC
    }
}

impl TryFrom<u8> for MBC {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(MBC::NoMBC),
            0x01 => Ok(MBC::MBC1),
            0x02 => Ok(MBC::Mbc1ExternalRam),
            0x03 => Ok(MBC::Mbc1BatteryExternalRam),
            _ => Err(()),
        }
    }
}


struct MBC0 {
    rom: RomBuffer,
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
            variant: MBCType::Mbc0(MBC0 { rom: Vec::default() })
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
    }
    else {
        0
    }
}

impl MBCWrapper {
    pub fn new(rom: RomBuffer) -> Self {
        let mbc = rom.get(0x147).and_then(|&v| v.try_into().ok()).unwrap_or_default();
        let num_rom_banks = rom_banks(*rom.get(0x148).unwrap_or(&0u8));

        let num_ram_banks = ram_banks(*rom.get(0x0149).unwrap_or(&0u8));
        let ram_size = num_ram_banks * 0x2000;


        if num_ram_banks > 0 {
            eprintln!("RAM size: {}", num_ram_banks);
        }

        match mbc {
            MBC::NoMBC => {
                Self {
                    variant: MBCType::Mbc0(MBC0 { rom })
                }
            }
            MBC::MBC1 => {
                Self {
                    variant: MBCType::Mbc1(MBC1 {
                        rom,
                        ram: iter::repeat(0u8).take(ram_size).collect(),
                        mode: MBC1Mode::RomMode,
                        ram_on: false,
                        rom_bank: 1,
                        ram_bank: 0,
                        num_ram_banks,
                        num_rom_banks,

                    })
                }
            }
            _ => panic!("No support for cartridge type: {:?}", mbc),
        }
    }

    pub fn read_rom(&self, addr: usize) -> u8 {
        match self.variant {
            MBCType::Mbc0(MBC0 { ref rom }) => rom[addr],
            MBCType::Mbc1(MBC1 { ref rom, ref rom_bank, ref mode, .. }) => {
                let bank = if addr < 0x4000 {
                    if *mode == MBC1Mode::RomMode {
                        rom_bank & 0xe0
                    } else { 0 }
                } else {
                    *rom_bank
                };
                let idx = bank * 0x4000 | (addr & 0x3fff);
                *rom.get(idx).unwrap_or(&0xff)
            }
        }
    }


    pub fn read_ram(&self, addr: usize) -> u8 {
        match self.variant {
            MBCType::Mbc0(_) => 0x00,
            MBCType::Mbc1(MBC1 { ref ram, ref ram_on, ref ram_bank, ref mode, .. }) => {
                if !ram_on { return 0xff; }
                let bank = if *mode == MBC1Mode::RamMode { *ram_bank } else { 0 };

                ram[(bank * 0x2000) | (addr & 0x1fff)]
            }
        }
    }

    pub fn write_ram(&mut self, addr: usize, value: u8) {
        match &mut self.variant {
            MBCType::Mbc0(_) => {}
            MBCType::Mbc1(ref mut a) => {
                if a.ram_on {
                    let bank = if a.mode == MBC1Mode::RamMode { a.ram_bank } else { 0 };
                    let idx = (bank * 0x2000) | (addr & 0x1fff);

                    if idx < a.ram.len() {
                        a.ram[idx] = value;
                    }
                }
            }
        }
    }

    pub fn write_rom(&mut self, addr: usize, value: u8) {
        match &mut self.variant {
            MBCType::Mbc0(_) => {}
            MBCType::Mbc1(ref mut a) => {
                match addr {
                    0x0000..=0x1fff => {
                        a.ram_on = value & 0xf == 0xa;
                    }
                    0x2000..=0x3fff => {
                        let lower_bits = match (value as usize) & 0x1f {
                            0 => 1,
                            n => n,
                        };
                        a.rom_bank = ((a.rom_bank & 0x60) | lower_bits) % a.num_rom_banks
                    }
                    0x4000..=0x5fff => {
                        if a.num_rom_banks > 0x20 {
                            let upper_bits = (value as usize & 0x03) % (a.num_rom_banks >> 5);
                            a.rom_bank = (a.rom_bank & 0x1f) | (upper_bits << 5);
                        }
                        if a.num_ram_banks > 1 {
                            a.ram_bank = (value as usize) & 0x03;
                        }
                    }
                    0x6000..=0x7fff => {
                        a.mode = MBC1Mode::try_from(value).expect("Set unexpected bank_mode");
                    }
                    _ => unreachable!("MBC1 invalid address, {:04X}", addr)
                }
            }
        }
    }
}
