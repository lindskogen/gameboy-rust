use bit_field::BitField;

use serde::{Serialize, Deserialize};
use crate::dmg::traits::Mem;

#[derive(Serialize, Deserialize)]
pub struct Serial {
    value: Option<u8>,
    debug_print: bool,
}

impl Default for Serial {
    fn default() -> Self {
        Self {
            debug_print: false,
            value: None,
        }
    }
}

impl Mem for Serial {
    fn read_byte(&self, addr: u16) -> u8 {
        // TODO: implement Serial transfers some day
        match addr {
            0xff01 | 0xff02 => 0x00,
            _ => unreachable!("SERIAL: Read from unmapped address: {:04X}", addr)
        }
    }

    fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff01 => self.value = Some(v),
            0xff02 if v.get_bit(7) => {
                if let Some(value) = self.value {
                    if self.debug_print {
                        eprint!("{}", value as char)
                    }
                }
            }
            0xff02 => {
                self.value = None;
            }
            _ => unreachable!("SERIAL: Write to unmapped address: {:04X}", addr)
        }
    }
}
