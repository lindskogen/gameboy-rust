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
            pc: 0x0,
            sp: 0xFFFE,
            mem,
        }
    }

    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn get_intermediate_u8(&mut self) -> u8 {
        let v = self.mem[self.pc + 1];
        self.pc += 1;
        v
    }

    fn get_intermediate_u16(&mut self) -> u16 {
        let (n1, n2) = self.get_intermediate_u16_tuple();

        ((n1 as u16) << 8) | (n2 as u16)
    }

    fn get_intermediate_u16_tuple(&mut self) -> (u8, u8) {
        let v = (self.mem[self.pc + 2], self.mem[self.pc + 1]);
        self.pc += 2;
        v
    }

    pub fn next(&mut self) {
        let pc = self.pc;

        match self.mem[pc] {

            // LD nn,n

            0x06 => {
                let n = self.get_intermediate_u8();
                self.ld_b(n);
            },
            0x0E => {
                let n = self.get_intermediate_u8();
                self.ld_c(n);
            },
            0x16 => {
                let n = self.get_intermediate_u8();
                self.ld_d(n);
            },
            0x1E => {
                let n = self.get_intermediate_u8();
                self.ld_e(n);
            },
            0x26 => {
                let n = self.get_intermediate_u8();
                self.ld_h(n);
            },
            0x2E => {
                let n = self.get_intermediate_u8();
                self.ld_l(n);
            },

            // LD r1, r2

            0x7F => self.ld_a(self.a),
            0x78 => self.ld_a(self.b),
            0x79 => self.ld_a(self.c),
            0x7A => self.ld_a(self.d),
            0x7B => self.ld_a(self.e),
            0x7C => self.ld_a(self.h),
            0x7D => self.ld_a(self.l),
            0x7E => self.ld_a(self.mem[self.get_hl()]),

            0x40 => self.ld_b(self.b),
            0x41 => self.ld_b(self.c),
            0x42 => self.ld_b(self.d),
            0x43 => self.ld_b(self.e),
            0x44 => self.ld_b(self.h),
            0x45 => self.ld_b(self.l),
            0x46 => self.ld_b(self.mem[self.get_hl()]),

            0x48 => self.ld_c(self.b),
            0x49 => self.ld_c(self.c),
            0x4A => self.ld_c(self.d),
            0x4B => self.ld_c(self.e),
            0x4C => self.ld_c(self.h),
            0x4D => self.ld_c(self.l),
            0x4E => self.ld_c(self.mem[self.get_hl()]),

            0x50 => self.ld_d(self.b),
            0x51 => self.ld_d(self.c),
            0x52 => self.ld_d(self.d),
            0x53 => self.ld_d(self.e),
            0x54 => self.ld_d(self.h),
            0x55 => self.ld_d(self.l),
            0x56 => self.ld_d(self.mem[self.get_hl()]),

            0x58 => self.ld_e(self.b),
            0x59 => self.ld_e(self.c),
            0x5A => self.ld_e(self.d),
            0x5B => self.ld_e(self.e),
            0x5C => self.ld_e(self.h),
            0x5D => self.ld_e(self.l),
            0x5E => self.ld_e(self.mem[self.get_hl()]),

            0x60 => self.ld_h(self.b),
            0x61 => self.ld_h(self.c),
            0x62 => self.ld_h(self.d),
            0x63 => self.ld_h(self.e),
            0x64 => self.ld_h(self.h),
            0x65 => self.ld_h(self.l),
            0x66 => self.ld_h(self.mem[self.get_hl()]),

            0x68 => self.ld_l(self.b),
            0x69 => self.ld_l(self.c),
            0x6A => self.ld_l(self.d),
            0x6B => self.ld_l(self.e),
            0x6C => self.ld_l(self.h),
            0x6D => self.ld_l(self.l),
            0x6E => self.ld_l(self.mem[self.get_hl()]),

            0x70 => self.ld_hl(self.b),
            0x71 => self.ld_hl(self.c),
            0x72 => self.ld_hl(self.d),
            0x73 => self.ld_hl(self.e),
            0x74 => self.ld_hl(self.h),
            0x75 => self.ld_hl(self.l),
            0x76 => {
                let n = self.get_intermediate_u8();
                self.ld_hl(n);
            },

            // LD n, nn

            0x01 => {
                let (n1, n2) = self.get_intermediate_u16_tuple();
                self.b = n1;
                self.c = n2;
            },
            0x11 => {
                let (n1, n2) = self.get_intermediate_u16_tuple();
                self.d = n1;
                self.e = n2;
            },
            0x21 => {
                let (n1, n2) = self.get_intermediate_u16_tuple();
                self.h = n1;
                self.l = n2;
            },
            0x31 => {
                let nn = self.get_intermediate_u16();
                self.sp = nn;
            },

            // LD SP, HL
            0xF9 => self.sp = self.get_hl(),

            // XOR n

            0xAF => self.xor(self.a),
            0xA8 => self.xor(self.b),
            0xA9 => self.xor(self.c),
            0xAA => self.xor(self.d),
            0xAB => self.xor(self.e),
            0xAC => self.xor(self.h),
            0xAD => self.xor(self.l),
            0xAE => self.xor(self.mem[self.get_hl()]),

            _ => {
                println!("Unimplemented: {} {:x}", pc, self.mem[pc]);
                unimplemented!()
            }
        }

        self.pc += 1;
    }

    fn xor(&mut self, n: u8) {
        self.a = self.a ^ n;
        self.reset_and_set_zero(self.a);
    }

    fn reset_and_set_zero(&mut self, n: u8) {
        // OR 0 with 0x10000000 if a is zero
        self.f = 0 | if n == 0 {
            // Z N H C 0 0 0 0
            0b10000000
        } else {
            0
        };
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

    fn ld_hl(&mut self, n: u8) {
        self.mem.set_at(self.get_hl(), n);
    }
}


