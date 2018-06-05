use super::mem::Memory;

const REG_P1_ADDR: usize = 0xFF00;
const REG_SB_ADDR: usize = 0xFF01;
const REG_SC_ADDR: usize = 0xFF02;
const REG_DIV_ADDR: usize = 0xFF04;

#[derive(Debug)]
pub struct ProcessingUnit {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    mem: Memory,
}

impl ProcessingUnit {
    pub fn new(mem: Memory) -> ProcessingUnit {
        ProcessingUnit {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            pc: 0x100,
            sp: 0xFFFE,
            mem,
        }
    }

    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn next(&mut self) {
        let pc = self.pc;

        match self.mem[pc] {
            0x06 => self.ld_b(self.mem[pc + 1]),
            0x0E => self.ld_c(self.mem[pc + 1]),
            0x16 => self.ld_d(self.mem[pc + 1]),
            0x1E => self.ld_e(self.mem[pc + 1]),
            0x26 => self.ld_h(self.mem[pc + 1]),
            0x2E => self.ld_l(self.mem[pc + 1]),

            0x7F => self.ld_a(self.a),
            0x78 => self.ld_a(self.b),
            0x79 => self.ld_a(self.c),
            0x7A => self.ld_a(self.d),
            0x7B => self.ld_a(self.e),
            0x7C => self.ld_a(self.h),
            0x7E => self.ld_a(self.mem[self.get_hl()]),

            _ => {
                println!("{}", self.mem[pc]);
                unimplemented!()
            }
        }
    }


    fn ld_a(&mut self, n: u8) {
        self.a = n;
    }

    fn ld_b(&mut self, n: u8) {
        self.b = n;
    }

    fn ld_c(&mut self, n: u8) {
        self.c = n;
    }

    fn ld_d(&mut self, n: u8) {
        self.d = n;
    }

    fn ld_e(&mut self, n: u8) {
        self.e = n;
    }

    fn ld_h(&mut self, n: u8) {
        self.h = n;
    }

    fn ld_l(&mut self, n: u8) {
        self.l = n;
    }
}


