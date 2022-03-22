use bit_field::BitField;

use crate::dmg::debug::{lookup_cb_prefix_op_code, lookup_op_code};

use super::Flags;
use super::ProcessingUnit;

impl ProcessingUnit {
    pub fn next(&mut self) -> u32 {
        self.check_and_execute_interrupts();

        let pc = self.pc;

        self.pc += 1;

        match self.read_byte(pc) {
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

            0x0A => self.ld_a(self.read_byte(self.get_bc())),
            0x1A => self.ld_a(self.read_byte(self.get_de())),
            0x7E => self.ld_a(self.read_byte(self.get_hl())),
            0xFA => {
                let v = self.get_immediate_u16();
                self.ld_a(self.read_byte(v));
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
            0x46 => self.ld_b(self.read_byte(self.get_hl())),

            0x48 => self.ld_c(self.b),
            0x49 => self.ld_c(self.c),
            0x4A => self.ld_c(self.d),
            0x4B => self.ld_c(self.e),
            0x4C => self.ld_c(self.h),
            0x4D => self.ld_c(self.l),
            0x4E => self.ld_c(self.read_byte(self.get_hl())),

            0x50 => self.ld_d(self.b),
            0x51 => self.ld_d(self.c),
            0x52 => self.ld_d(self.d),
            0x53 => self.ld_d(self.e),
            0x54 => self.ld_d(self.h),
            0x55 => self.ld_d(self.l),
            0x56 => self.ld_d(self.read_byte(self.get_hl())),

            0x58 => self.ld_e(self.b),
            0x59 => self.ld_e(self.c),
            0x5A => self.ld_e(self.d),
            0x5B => self.ld_e(self.e),
            0x5C => self.ld_e(self.h),
            0x5D => self.ld_e(self.l),
            0x5E => self.ld_e(self.read_byte(self.get_hl())),

            0x60 => self.ld_h(self.b),
            0x61 => self.ld_h(self.c),
            0x62 => self.ld_h(self.d),
            0x63 => self.ld_h(self.e),
            0x64 => self.ld_h(self.h),
            0x65 => self.ld_h(self.l),
            0x66 => self.ld_h(self.read_byte(self.get_hl())),

            0x68 => self.ld_l(self.b),
            0x69 => self.ld_l(self.c),
            0x6A => self.ld_l(self.d),
            0x6B => self.ld_l(self.e),
            0x6C => self.ld_l(self.h),
            0x6D => self.ld_l(self.l),
            0x6E => self.ld_l(self.read_byte(self.get_hl())),

            0x70 => self.ld_hl(self.b),
            0x71 => self.ld_hl(self.c),
            0x72 => self.ld_hl(self.d),
            0x73 => self.ld_hl(self.e),
            0x74 => self.ld_hl(self.h),
            0x75 => self.ld_hl(self.l),
            0x36 => {
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
            0x02 => self.write_byte(self.get_bc(), self.a),
            0x12 => self.write_byte(self.get_de(), self.a),
            0x77 => self.write_byte(self.get_hl(), self.a),
            0xEA => {
                let addr = self.get_immediate_u16();
                self.write_byte(addr, self.a)
            }

            // 5. LD A, (C)
            0xF2 => {
                let addr: u16 = 0xff00 + (self.c as u16);

                self.a = self.read_byte(addr);
            }

            // 6. LD (C), A
            0xE2 => {
                let addr: u16 = 0xff00 + (self.c as u16);
                self.write_byte(addr, self.a);
            }

            // 10, 11, 12: LDD (HL), A
            0x32 => {
                let hl = self.get_hl();
                self.write_byte(hl, self.a);
                self.set_hl(hl.wrapping_sub(1));
            }

            // 13, 14, 15: LD A, (HLI)

            0x2a => {
                self.lda_hli();
            }

            // 16, 17, 18: LDI (HL), A

            0x22 => {
                self.ldi_hla()
            }

            // 19. LDH (n), A
            0xe0 => {
                let n = self.get_immediate_u8();
                let addr: u16 = 0xff00 + (n as u16);
                self.write_byte(addr, self.a);
            }

            // 20. LDH A, (n)
            0xF0 => {
                let n = self.get_immediate_u8();
                let addr: u16 = 0xff00 + (n as u16);
                self.a = self.read_byte(addr);
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

            // 3. LD HL, SP+n
            // 4. LDHL SP, n

            0xF8 => {
                let v = (self.sp as i16 + self.get_immediate_i8() as i16) as u16;
                self.set_hl(v);
            }

            // 5. LD (nn),SP
            0x08 => {
                let addr = self.get_immediate_u16();
                let (sp1, sp2) = ((self.sp & 0xff00 >> 8) as u8, (self.sp & 0xff) as u8);
                self.write_byte(addr, sp1);
                self.write_byte(addr + 1, sp2);
            }

            // 6. PUSH nn
            0xF5 => self.push_u16(self.get_af()),
            0xC5 => self.push_u16(self.get_bc()),
            0xD5 => self.push_u16(self.get_de()),
            0xE5 => self.push_u16(self.get_hl()),

            // 3.3.3 8-bit ALU

            // 1. ADD A,n

            0x87 => self.add_a(self.a),
            0x80 => self.add_a(self.b),
            0x81 => self.add_a(self.c),
            0x82 => self.add_a(self.d),
            0x83 => self.add_a(self.e),
            0x84 => self.add_a(self.h),
            0x85 => self.add_a(self.l),
            0x86 => self.add_a(self.read_byte(self.get_hl())),
            0xc6 => {
                let n = self.get_immediate_u8();
                self.add_a(n)
            }

            // 2. ADC A, n

            0x8F => self.adc(self.a),
            0x88 => self.adc(self.b),
            0x89 => self.adc(self.c),
            0x8A => self.adc(self.d),
            0x8B => self.adc(self.e),
            0x8C => self.adc(self.h),
            0x8D => self.adc(self.l),
            0x8E => self.adc(self.read_byte(self.get_hl())),
            0xCE => {
                let n = self.get_immediate_u8();
                self.adc(n)
            }


            // 3. SUB n

            0x97 => self.sub_a(self.a),
            0x90 => self.sub_a(self.b),
            0x91 => self.sub_a(self.c),
            0x92 => self.sub_a(self.d),
            0x93 => self.sub_a(self.e),
            0x94 => self.sub_a(self.h),
            0x95 => self.sub_a(self.l),
            0x96 => self.sub_a(self.read_byte(self.get_hl())),
            0xD6 => {
                let n = self.get_immediate_u8();
                self.sub_a(n)
            }

            // 4. SBC A, n
            0x9f => self.sbc(self.a),
            0x98 => self.sbc(self.b),
            0x99 => self.sbc(self.c),
            0x9a => self.sbc(self.d),
            0x9b => self.sbc(self.e),
            0x9c => self.sbc(self.h),
            0x9d => self.sbc(self.l),
            0x9e => self.sbc(self.read_byte(self.get_hl())),

            // 5. AND n
            0xa7 => self.and(self.a),
            0xa0 => self.and(self.b),
            0xa1 => self.and(self.c),
            0xa2 => self.and(self.d),
            0xa3 => self.and(self.e),
            0xa4 => self.and(self.h),
            0xa5 => self.and(self.l),
            0xa6 => self.and(self.read_byte(self.get_hl())),
            0xe6 => {
                let param = self.get_immediate_u8();
                self.and(param)
            }

            // 6. OR n
            0xb7 => self.or(self.a),
            0xb0 => self.or(self.b),
            0xb1 => self.or(self.c),
            0xb2 => self.or(self.d),
            0xb3 => self.or(self.e),
            0xb4 => self.or(self.h),
            0xb5 => self.or(self.l),
            0xb6 => self.or(self.read_byte(self.get_hl())),
            0xf6 => {
                let param = self.get_immediate_u8();
                self.or(param)
            }


            // 7. XOR n
            0xAF => self.xor(self.a),
            0xA8 => self.xor(self.b),
            0xA9 => self.xor(self.c),
            0xAA => self.xor(self.d),
            0xAB => self.xor(self.e),
            0xAC => self.xor(self.h),
            0xAD => self.xor(self.l),
            0xAE => self.xor(self.read_byte(self.get_hl())),
            0xEE => {
                let param = self.get_immediate_u8();
                self.xor(param)
            }


            // 8. CP n
            0xBF => { self.compare_a_with(self.a) }
            0xB8 => { self.compare_a_with(self.b) }
            0xB9 => { self.compare_a_with(self.c) }
            0xBA => { self.compare_a_with(self.d) }
            0xBC => { self.compare_a_with(self.h) }
            0xBD => { self.compare_a_with(self.l) }
            0xBE => { self.compare_a_with(self.read_byte(self.get_hl())) }
            0xFE => {
                let param = self.get_immediate_u8();
                self.compare_a_with(param)
            }

            // 9. INC n
            0x3C => {
                let a = self.a;
                self.a = self.a.wrapping_add(1);

                self.reset_and_set_carry_zero(a, self.a);
            }
            0x04 => {
                let b = self.b;
                self.b = self.b.wrapping_add(1);

                self.reset_and_set_carry_zero(b, self.b);
            }
            0x0C => {
                let c = self.c;
                self.c = self.c.wrapping_add(1);

                self.reset_and_set_carry_zero(c, self.c);
            }
            0x14 => {
                let d = self.d;
                self.d = self.d.wrapping_add(1);

                self.reset_and_set_carry_zero(d, self.d);
            }
            0x1C => {
                let e = self.e;
                self.e = self.e.wrapping_add(1);

                self.reset_and_set_carry_zero(e, self.e);
            }
            0x24 => {
                let h = self.h;
                self.h = self.h.wrapping_add(1);

                self.reset_and_set_carry_zero(h, self.h);
            }
            0x2C => {
                let l = self.l;
                self.l = self.l.wrapping_add(1);

                self.reset_and_set_carry_zero(l, self.l);
            }
            0x34 => {
                let hl = self.get_hl();
                let n = self.read_byte(hl);
                let nn = n.wrapping_add(1);

                self.reset_and_set_carry_zero(n, nn);
                self.write_byte(hl, nn);
            }

            // 10. DEC n

            0x3D => {
                let prev = self.a;
                self.a = self.a.wrapping_sub(1);
                self.dec_flags(prev, self.a);
            }
            0x05 => {
                let prev = self.b;
                self.b = self.b.wrapping_sub(1);
                self.dec_flags(prev, self.b);
            }
            0x0D => {
                let prev = self.c;
                self.c = self.c.wrapping_sub(1);
                self.dec_flags(prev, self.c);
            }
            0x15 => {
                let prev = self.d;
                self.d = self.d.wrapping_sub(1);
                self.dec_flags(prev, self.d);
            }
            0x1D => {
                let prev = self.e;
                self.e = self.e.wrapping_sub(1);
                self.dec_flags(prev, self.e);
            }
            0x25 => {
                let prev = self.h;
                self.h = self.h.wrapping_sub(1);
                self.dec_flags(prev, self.h);
            }
            0x2D => {
                let prev = self.l;
                self.l = self.l.wrapping_sub(1);
                self.dec_flags(prev, self.l);
            }
            0x35 => {
                let hl = self.get_hl();
                let prev = self.read_byte(hl);
                let r = prev.wrapping_sub(1);
                self.write_byte(hl, r);
                self.dec_flags(prev, r);
            }


            // 3.3.4 16-bit Arithmetic

            // 1. ADD HL,n
            0x09 => self.add_hl_16(self.get_bc()),
            0x19 => self.add_hl_16(self.get_de()),
            0x29 => self.add_hl_16(self.get_hl()),
            0x39 => self.add_hl_16(self.sp),

            // 3. INC nn

            0x03 => { self.set_bc(self.get_bc().wrapping_add(1)) }
            0x13 => { self.set_de(self.get_de().wrapping_add(1)) }
            0x23 => { self.set_hl(self.get_hl().wrapping_add(1)) }
            0x33 => { self.sp = self.sp.wrapping_add(1) }

            // 4. DEC nn

            0x0b => { self.set_bc(self.get_bc().wrapping_sub(1)) }
            0x1b => { self.set_de(self.get_de().wrapping_sub(1)) }
            0x2b => { self.set_hl(self.get_hl().wrapping_sub(1)) }
            0x3b => { self.sp = self.sp.wrapping_sub(1) }

            // 3.3.5 Miscellaneous

            // 3. CPL
            0x2f => {
                self.a = !self.a;

                self.f.insert(Flags::N);
                self.f.insert(Flags::H);
            }

            // 4. CCF
            0x3f => {
                self.f.remove(Flags::N);
                self.f.remove(Flags::H);
                self.f.toggle(Flags::CARRY);
            }

            // 5. SCF
            0x37 => {
                self.f.remove(Flags::N);
                self.f.remove(Flags::H);
                self.f.insert(Flags::CARRY);
            }

            // 6. NOP
            0x00 => {}

            // 9. DI

            0xf3 => {
                self.master_interrupt_enabled = false;
            }

            // 10. EI
            0xfb => {
                self.master_interrupt_enabled = true;
            }


            // 3.3.6 Rotates & shifts

            // 1. RLCA

            0x07 => {
                self.a = self.rlca_8(self.a);
            }

            // 2. RLA
            0x17 => { self.a = self.rl_8(self.a); }

            // 3. RRCA
            0x0F => {
                self.rrca_8(self.a);
            }

            // 4. RRA
            0x1F => {
                self.rr_8(self.a);
            }

            0xCB => {
                let npc = pc + 1;
                self.pc += 1;
                match self.read_byte(npc) {


                    // 1. SWAP n
                    0x37 => self.a = self.swap(self.a),
                    0x30 => self.b = self.swap(self.b),
                    0x31 => self.c = self.swap(self.c),
                    0x32 => self.d = self.swap(self.d),
                    0x33 => self.e = self.swap(self.e),
                    0x34 => self.h = self.swap(self.h),
                    0x35 => self.l = self.swap(self.l),
                    0x36 => {
                        let hl = self.get_hl();
                        let r = self.swap(self.read_byte(hl));
                        self.write_byte(hl, r);
                    }


                    // 6. RL n
                    0x17 => self.a = self.rl_8(self.a),
                    0x10 => self.b = self.rl_8(self.b),
                    0x11 => self.c = self.rl_8(self.c),
                    0x12 => self.d = self.rl_8(self.d),
                    0x13 => self.e = self.rl_8(self.e),
                    0x14 => self.h = self.rl_8(self.h),
                    0x15 => self.l = self.rl_8(self.l),
                    0x16 => {
                        let hl = self.get_hl();
                        let r = self.rl_8(self.read_byte(hl));
                        self.write_byte(hl, r);
                    }

                    // 8. RR n

                    0x1F => self.a = self.rr_8(self.a),
                    0x18 => self.b = self.rr_8(self.b),
                    0x19 => self.c = self.rr_8(self.c),
                    0x1A => self.d = self.rr_8(self.d),
                    0x1B => self.e = self.rr_8(self.e),
                    0x1C => self.h = self.rr_8(self.h),
                    0x1D => self.l = self.rr_8(self.l),
                    0x1E => {
                        let hl = self.get_hl();
                        let r = self.rr_8(self.read_byte(hl));
                        self.write_byte(hl, r);
                    }

                    // 11. SRL n

                    0x3F => {
                        let a = self.a;
                        self.a = a >> 1;
                        self.f.set(Flags::ZERO, self.a == 0);
                        self.f.remove(Flags::N);
                        self.f.remove(Flags::H);
                        self.f.set(Flags::CARRY, (0b1 & a) != 0)
                    }

                    0x38 => {
                        let a = self.b;
                        self.b = a >> 1;
                        self.f.set(Flags::ZERO, self.b == 0);
                        self.f.remove(Flags::N);
                        self.f.remove(Flags::H);
                        self.f.set(Flags::CARRY, (0b1 & a) != 0)
                    }

                    // 3.3.7. Bit Opcodes


                    // BIT 0, A
                    0x47 => {
                        self.f.set(Flags::ZERO, !self.a.get_bit(0));
                        self.f.remove(Flags::N);
                        self.f.insert(Flags::H);
                    }
                    // BIT 3, A
                    0x5f => {
                        self.f.set(Flags::ZERO, !self.a.get_bit(3));
                        self.f.remove(Flags::N);
                        self.f.insert(Flags::H);
                    }

                    // BIT 7, H
                    0x7c => {
                        self.f.set(Flags::ZERO, !self.h.get_bit(7));
                        self.f.remove(Flags::N);
                        self.f.insert(Flags::H);
                    }
                    _ => {
                        println!("Unimplemented under 0xCB at pc={:x}, op={:x}: {}", npc, self.read_byte(npc), lookup_cb_prefix_op_code(self.read_byte(npc)).0);
                        println!("{:?}", self);
                        unimplemented!()
                    }
                }
            }

            // 3.3.8 Jumps

            // 1. JP nn
            0xC3 => { self.pc = self.get_immediate_u16() }

            // 2. JP cc,nn

            0xC2 => {
                if !self.f.contains(Flags::ZERO) {
                    self.pc = self.get_immediate_u16()
                }
            }

            0xCA => {
                if self.f.contains(Flags::ZERO) {
                    self.pc = self.get_immediate_u16()
                }
            }

            0xD2 => {
                if !self.f.contains(Flags::CARRY) {
                    self.pc = self.get_immediate_u16()
                }
            }

            0xDA => {
                if self.f.contains(Flags::CARRY) {
                    self.pc = self.get_immediate_u16()
                }
            }

            // 3. JP (HL)
            0xE9 => {
                self.pc = self.get_hl();
            }

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

            // 1. CALL nn
            0xCD => self.call(),

            // 2. CALL cc,nn

            0xC4 => {
                if !self.f.contains(Flags::ZERO) {
                    self.call();
                }
            }

            0xCC => {
                if self.f.contains(Flags::ZERO) {
                    self.call();
                }
            }

            0xD4 => {
                if !self.f.contains(Flags::CARRY) {
                    self.call();
                }
            }

            0xDC => {
                if self.f.contains(Flags::CARRY) {
                    self.call();
                }
            }

            // 3.3.10 Restarts

            // 1. RST n
            0xC7 => self.rst(pc, 0x00),
            0xCF => self.rst(pc, 0x08),
            0xD7 => self.rst(pc, 0x10),
            0xDF => self.rst(pc, 0x18),
            0xE7 => self.rst(pc, 0x20),
            0xEF => self.rst(pc, 0x28),
            0xF7 => self.rst(pc, 0x30),
            0xFF => self.rst(pc, 0x38),

            // 3.3.11 Returns

            // 1. RET
            0xC9 => self.ret(),

            // 2. RET cc
            0xC0 => {
                if !self.f.contains(Flags::ZERO) {
                    self.ret();
                }
            }

            // 3. RETI

            0xD9 => {
                self.ret();
                self.master_interrupt_enabled = true;
            }

            0xC8 => {
                if self.f.contains(Flags::ZERO) {
                    self.ret();
                }
            }

            0xD0 => {
                if !self.f.contains(Flags::CARRY) {
                    self.ret();
                }
            }

            0xD8 => {
                if self.f.contains(Flags::CARRY) {
                    self.ret();
                }
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
                println!("Unimplemented at pc={:x}, op={:x}: {}", pc, self.read_byte(pc), lookup_op_code(self.read_byte(pc)).0);
                println!("{:?}", self);
                unimplemented!()
            }
        }

        let cycles = self.lookup_op_code_for_pc(pc).1;
        self.cycles = self.cycles.wrapping_add(cycles);
        return cycles;
    }
}
