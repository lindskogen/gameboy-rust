use bitflags::bitflags;

use dmg::debug::{lookup_cb_prefix_op_code, lookup_op_code};

use super::mem::Memory;

bitflags! {
    struct Flags: u8 {
        const ZERO = 0b10000000;
        const N = 0b01000000;
        const H = 0b00100000;
        const HALF_CARRY = 0b00100000;
        const C = 0b00010000;
        const CARRY = 0b00010000;
        const F = Self::ZERO.bits | Self::N.bits | Self::H.bits | Self::C.bits;
    }
}

#[derive(Debug)]
pub struct ProcessingUnit {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: Flags,
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
            f: Flags::empty(),
            h: 0,
            l: 0,
            pc: 0x0,
            sp: 0xFFFE,
            mem,
        }
    }

    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f.bits as u16)
    }
    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }
    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn set_bc(&mut self, v: u16) {
        self.b = (v >> 8) as u8;
        self.c = v as u8;
    }

    fn set_de(&mut self, v: u16) {
        self.d = (v >> 8) as u8;
        self.e = v as u8;
    }

    fn set_hl(&mut self, v: u16) {
        self.h = (v >> 8) as u8;
        self.l = v as u8;
    }

    fn get_immediate_u8(&mut self) -> u8 {
        let v = self.mem[self.pc];
        self.pc += 1;
        v
    }

    fn get_immediate_i8(&mut self) -> i8 {
        let v = self.mem[self.pc] as i8;
        self.pc += 1;
        v
    }

    fn get_immediate_u16(&mut self) -> u16 {
        let (n1, n2) = self.get_immediate_u16_tuple();

        ((n1 as u16) << 8) | (n2 as u16)
    }

    fn get_immediate_u16_tuple(&mut self) -> (u8, u8) {
        let v = (self.mem[self.pc + 1], self.mem[self.pc]);
        self.pc += 2;
        v
    }

    pub fn debug_print(&self, pc: u16) {

        let op_code = if self.mem[pc] != 0xCB {
            lookup_op_code(self.mem[pc])
        } else {
            lookup_cb_prefix_op_code(self.mem[pc])
        };



        println!("{:5x}: {:<10}\ta: {:2x}, b: {:2x}, c: {:2x}, d: {:2x}, e: {:2x}, h: {:2x}, l: {:2x}, sp: {:4x}, flags: {:?}", pc, op_code, self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.f)
    }

    pub fn next(&mut self) {
        let pc = self.pc;
        self.pc += 1;
        // println!("{:x}", pc);

        match self.mem[pc] {
            // 3.3.1 8-bit loads
            // 1. LD nn,n
            0x06 => {
                let n = self.get_immediate_u8();
                self.ld_b(n);
            }
            0x0E => {
                let n = self.get_immediate_u8();
                self.ld_c(n);
            }
            0x16 => {
                let n = self.get_immediate_u8();
                self.ld_d(n);
            }
            0x1E => {
                let n = self.get_immediate_u8();
                self.ld_e(n);
            }
            0x26 => {
                let n = self.get_immediate_u8();
                self.ld_h(n);
            }
            0x2E => {
                let n = self.get_immediate_u8();
                self.ld_l(n);
            }

            // 2. LD r1, r2
            0x7F => self.ld_a(self.a),
            0x78 => self.ld_a(self.b),
            0x79 => self.ld_a(self.c),
            0x7A => self.ld_a(self.d),
            0x7B => self.ld_a(self.e),
            0x7C => self.ld_a(self.h),
            0x7D => self.ld_a(self.l),

            0x0A => self.ld_a(self.mem[self.get_bc()]),
            0x1A => self.ld_a(self.mem[self.get_de()]),
            0x7E => self.ld_a(self.mem[self.get_hl()]),
            0xFA => {
                let v = self.get_immediate_u16();
                self.ld_a(self.mem[v]);
            }
            0x3E => {
                let v = self.get_immediate_u8();
                self.ld_a(v);
            }

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
                let n = self.get_immediate_u8();
                self.ld_hl(n);
            }

            // 4. LD n, A
            // 0x7F => self.ld_a(self.a),
            0x47 => self.ld_b(self.a),
            0x4F => self.ld_c(self.a),
            0x57 => self.ld_d(self.a),
            0x5F => self.ld_e(self.a),
            0x67 => self.ld_h(self.a),
            0x6F => self.ld_l(self.a),
            0x02 => self.mem.set_at(self.get_bc(), self.a),
            0x12 => self.mem.set_at(self.get_de(), self.a),
            0x77 => self.mem.set_at(self.get_hl(), self.a),
            0xEA => {
                let addr = self.get_immediate_u16();
                self.mem.set_at(addr, self.a)
            }

            // 5. LD A, (C)
            0xF2 => {
                let addr: u16 = 0xff00 + (self.c as u16);

                self.a = self.mem[addr];
            }

            // 6. LD (C), A
            0xE2 => {
                let addr: u16 = 0xff00 + (self.c as u16);
                self.mem.set_at(addr, self.a);
            }

            // 10, 11, 12: LDD (HL), A
            0x32 => {
                self.mem.set_at(self.get_hl(), self.a);
                let v = self.get_hl().wrapping_sub(1);
                self.set_hl(v);
            }

            // 16, 17, 18: LDI (HL), A

            0x22 => {
                self.ld_hl(self.a);

                let n = self.get_hl();
                let nn = n.wrapping_add(1);
                self.set_hl(nn)
            }

            // 19. LDH (n), A
            0xe0 => {
                let n = self.get_immediate_u8();
                let addr: u16 = 0xff00 + (n as u16);
                self.mem.set_at(addr, self.a);
            }

            // 20. LDH A, (n)
            0xF0 => {
                let n = self.get_immediate_u8();
                let addr: u16 = 0xff00 + (n as u16);
                self.a = self.mem[addr];
            }

            // 3.3.2 16-bit loads
            // 1. LD n, nn
            0x01 => {
                let (n1, n2) = self.get_immediate_u16_tuple();
                self.b = n1;
                self.c = n2;
            }
            0x11 => {
                let (n1, n2) = self.get_immediate_u16_tuple();
                self.d = n1;
                self.e = n2;
            }
            0x21 => {
                let (n1, n2) = self.get_immediate_u16_tuple();
                self.h = n1;
                self.l = n2;
            }
            0x31 => {
                let nn = self.get_immediate_u16();
                self.sp = nn;
            }

            // 2. LD SP, HL
            0xF9 => self.sp = self.get_hl(),

            // 6. PUSH nn
            0xF5 => self.push_u16(self.get_af()),
            0xC5 => self.push_u16(self.get_bc()),
            0xD5 => self.push_u16(self.get_de()),
            0xE5 => self.push_u16(self.get_hl()),

            // 3.3.3 8-bit ALU
            // 7 XOR n
            0xAF => self.xor(self.a),
            0xA8 => self.xor(self.b),
            0xA9 => self.xor(self.c),
            0xAA => self.xor(self.d),
            0xAB => self.xor(self.e),
            0xAC => self.xor(self.h),
            0xAD => self.xor(self.l),
            0xAE => self.xor(self.mem[self.get_hl()]),

            // 8. CP n
            0xBF => { self.compare_a_with(self.a) }
            0xB8 => { self.compare_a_with(self.b) }
            0xB9 => { self.compare_a_with(self.c) }
            0xBA => { self.compare_a_with(self.d) }
            0xBC => { self.compare_a_with(self.h) }
            0xBD => { self.compare_a_with(self.l) }
            0xBE => { self.compare_a_with(self.mem[self.get_hl()]) }
            0xFE => {
                let param = self.get_immediate_u8();
                self.compare_a_with(param)
            }

            // 9. INC n
            0x3C => {
                let a = self.a;
                self.a += 1;

                self.reset_and_set_carry_zero(a, self.a);
            }
            0x04 => {
                let b = self.b;
                self.b += 1;

                self.reset_and_set_carry_zero(b, self.b);
            }
            0x0C => {
                let c = self.c;
                self.c += 1;

                self.reset_and_set_carry_zero(c, self.c);
            }
            0x14 => {
                let d = self.d;
                self.d += 1;

                self.reset_and_set_carry_zero(d, self.d);
            }
            0x1C => {
                let e = self.e;
                self.e += 1;

                self.reset_and_set_carry_zero(e, self.e);
            }
            0x24 => {
                let h = self.h;
                self.h += 1;

                self.reset_and_set_carry_zero(h, self.h);
            }
            0x2C => {
                let l = self.l;
                self.l += 1;

                self.reset_and_set_carry_zero(l, self.l);
            }
            0x34 => {
                let n = self.mem[self.get_hl()];
                let nn = n + 1;

                self.reset_and_set_carry_zero(n, nn);
                self.mem.set_at(self.get_hl(), nn);
            }

            // 10. DEC n

            0x3D => {
                let prev = self.a;
                self.a -= 1;
                self.dec_flags(prev, self.a);
            }
            0x05 => {
                let prev = self.b;
                self.b -= 1;
                self.dec_flags(prev, self.b);
            }
            0x0D => {
                let prev = self.c;
                self.c -= 1;
                self.dec_flags(prev, self.c);
            }
            0x15 => {
                let prev = self.d;
                self.d -= 1;
                self.dec_flags(prev, self.d);
            }
            0x1D => {
                let prev = self.e;
                self.e -= 1;
                self.dec_flags(prev, self.e);
            }
            0x25 => {
                let prev = self.h;
                self.h -= 1;
                self.dec_flags(prev, self.h);
            }
            0x2D => {
                let prev = self.l;
                self.l -= 1;
                self.dec_flags(prev, self.l);
            }
            0x35 => {
                let prev = self.get_hl();
                self.set_hl(prev - 1);
                self.dec_flags_16(prev, self.get_hl());
            }


            // 3.3.4 16-bit Arithmetic

            // 3. INC nn

            0x03 => { self.set_bc(self.get_bc().wrapping_add(1)) }
            0x13 => { self.set_de(self.get_de().wrapping_add(1)) }
            0x23 => { self.set_hl(self.get_hl().wrapping_add(1)) }
            0x33 => { self.sp = self.sp.wrapping_add(1) }

            // 3.3.5 Miscellaneous

            // 6 NOP
            0x00 => {}

            // 3.3.6 Rotates & shifts

            0x17 => { self.a = self.rl_n_8(self.a); }

            // 3.3.7 Bit opcodes
            0xCB => {
                let npc = pc + 1;
                self.pc += 1;
                let bitinstruction = self.mem[npc].to_le();
                match bitinstruction {
                    // RL n
                    0x17 => { self.a = self.rl_n_8(self.a); }
                    0x11 => { self.c = self.rl_n_8(self.c); }
                    0x12 => { self.d = self.rl_n_8(self.d); }
                    0x13 => { self.e = self.rl_n_8(self.e); }
                    0x14 => { self.h = self.rl_n_8(self.h); }
                    0x15 => { self.l = self.rl_n_8(self.l); }
                    0x16 => {
                        let i = self.rl_n_16(self.get_hl());
                        self.set_hl(i);
                    }

                    0x7c => {
                        // BIT 7,H
                        let bit = (self.h >> 6) & 0b1;
                        if bit == 0 {
                            self.f.insert(Flags::ZERO);
                        }
                        self.f.remove(Flags::N);
                        self.f.insert(Flags::H);
                    }
                    _ => {
                        println!("Unimplemented under 0xCB at pc={:x}, op={:x}: {}", npc, self.mem[npc], lookup_cb_prefix_op_code(self.mem[npc]));
                        println!("{:?}", self);
                        unimplemented!()
                    }
                }
            }

            // 3.3.8 Jumps

            // 4. JR n
            0x18 => {
                let n = self.get_immediate_i8();
                self.pc = ((self.pc as i16) + n as i16) as u16;
            }


            // 5. JR cc,n

            // JR NZ,*
            0x20 => {
                let n = self.get_immediate_i8();
                if !self.f.contains(Flags::ZERO) {
                    self.pc = ((self.pc as i16) + n as i16) as u16;
                }
            }
            // JR Z,*
            0x28 => {
                let n = self.get_immediate_i8();
                if self.f.contains(Flags::ZERO) {
                    self.pc = ((self.pc as i16) + n as i16) as u16;
                }
            }
            // JR NC,*
            0x30 => {
                let n = self.get_immediate_i8();
                if !self.f.contains(Flags::CARRY) {
                    self.pc = ((self.pc as i16) + n as i16) as u16;
                }
            }
            // JR C,*
            0x38 => {
                let n = self.get_immediate_i8();
                if self.f.contains(Flags::CARRY) {
                    self.pc = ((self.pc as i16) + n as i16) as u16;
                }
            }

            // 3.3.9 Calls

            // CALL nn
            0xCD => {
                let nn = self.get_immediate_u16();
                self.push_u16(self.pc);
                self.pc = nn;
            }

            // 3.3.10 Restarts

            // 3.3.11 Returns

            // RET
            0xC9 => {
                let lsb = self.read_sp_u8() as u16;
                let msb = self.read_sp_u8() as u16;

                let dest = (msb << 8) | lsb;
                self.pc = dest
            }

            // 7. POP nn
            0xC1 => {
                self.c = self.read_sp_u8();
                self.b = self.read_sp_u8();
            }
            0xD1 => {
                self.e = self.read_sp_u8();
                self.d = self.read_sp_u8();
            }
            0xE1 => {
                self.l = self.read_sp_u8();
                self.h = self.read_sp_u8();
            }
            0xF1 => {
                self.f.bits = self.read_sp_u8();
                self.a = self.read_sp_u8();
            }

            _ => {
                println!("Unimplemented at pc={:x}, op={:x}: {}", pc, self.mem[pc], lookup_op_code(self.mem[pc]));
                println!("{:?}", self);
                unimplemented!()
            }
        }

        self.debug_print(pc);
    }

    fn rl_n_8(&mut self, v: u8) -> u8 {
        let carry = (v >> 6) & 0b1;
        let v = v.rotate_left(1);
        let bit = v & 0b1;
        if bit == 0 {
            self.f.insert(Flags::ZERO);
        }
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.set(Flags::C, carry == 1);

        v
    }

    fn rl_n_16(&mut self, v: u16) -> u16 {
        let carry = (v >> 6) & 0b1;
        let v = v.rotate_left(1);
        let bit = v & 0b1;
        if bit == 0 {
            self.f.insert(Flags::ZERO);
        }
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.set(Flags::C, carry == 1);

        v
    }


    fn xor(&mut self, n: u8) {
        self.a = self.a ^ n;
        self.reset_and_set_zero(self.a);
    }

    fn push_u8(&mut self, n: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.mem.set_at(self.sp, n);
    }

    fn push_u16(&mut self, n: u16) {
        let (n_msb, n_lsb) = (((n & 0xff00) >> 8) as u8, (n & 0xff) as u8);
        self.push_u8(n_msb);
        self.push_u8(n_lsb);
    }

    fn reset_and_set_carry_zero(&mut self, prev: u8, new: u8) {
        self.f.set(Flags::ZERO, new == 0);
        self.f.set(Flags::HALF_CARRY, (((prev & 0xf) + 1) & 0x10) == 0x10);
        self.f.remove(Flags::N);
    }

    fn reset_and_set_zero(&mut self, n: u8) {
        self.f.bits = 0;
        self.f.set(Flags::ZERO, n == 0);
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


    fn read_sp_u8(&mut self) -> u8 {
        let x = self.mem[self.sp];
        self.sp = self.sp.wrapping_add(1);

        return x;
    }

    fn dec_flags(&mut self, prev: u8, n: u8) {
        self.f.set(Flags::ZERO, n == 0);
        self.f.insert(Flags::N);
        self.set_half_carry(prev, n);
    }

    fn set_half_carry(&mut self, prev: u8, n: u8) {
        // H: Set if no borrow from bit 4
        self.f.set(Flags::H, ((prev >> 4) & 0b1) == ((n >> 4) & 0b1))
    }

    fn dec_flags_16(&mut self, prev: u16, n: u16) {
        self.f.set(Flags::ZERO, n == 0);
        self.f.insert(Flags::N);

        // H: Set if no borrow from bit 4
        self.f.set(Flags::H, ((prev >> 4) & 0b1) == ((n >> 4) & 0b1));
    }
    fn compare_a_with(&mut self, n: u8) {
        let nn = self.a.wrapping_sub(n);
        self.f.set(Flags::ZERO, nn == 0);
        self.f.insert(Flags::N);
        self.set_half_carry(n, nn);
        self.f.set(Flags::CARRY, self.a < n);
    }
}
