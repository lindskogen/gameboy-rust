use bit_field::BitField;

pub struct Serial {
    value: Option<u8>,
    debug_print: bool,
}

impl Default for Serial {
    fn default() -> Self {
        Self {
            debug_print: true,
            value: None,
        }
    }
}

impl Serial {
    pub fn read(&self, _addr: u16) -> u8 {
        // TODO: implement Serial transfers some day

        0xff
    }

    pub fn write(&mut self, addr: u16, v: u8) {
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
