use bit_field::BitField;

use super::debug::lookup_op_code;
use crate::dmg::mem::MemoryBus;

use super::Flags;
use super::ProcessingUnit;

impl ProcessingUnit {
    pub fn next(&mut self, bus: &mut MemoryBus) -> u32 {
        if self.check_and_execute_interrupts(bus) {
            return 4;
        }

        if self.halted {
            return 4;
        }

        let pc = self.pc;

        self.debug_print(pc, bus);

        self.pc += 1;

        match self.read_byte(bus, pc) {
            // 3.3.1 8-bit loads
            // 1. LD nn,n
            0x06 => {
                let n = self.get_immediate_u8(bus);
                self.ld_b(n);
            }
            0x0E => {
                let n = self.get_immediate_u8(bus);
                self.ld_c(n);
            }
            0x16 => {
                let n = self.get_immediate_u8(bus);
                self.ld_d(n);
            }
            0x1E => {
                let n = self.get_immediate_u8(bus);
                self.ld_e(n);
            }
            0x26 => {
                let n = self.get_immediate_u8(bus);
                self.ld_h(n);
            }
            0x2E => {
                let n = self.get_immediate_u8(bus);
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

            0x0A => self.ld_a(self.read_byte(bus, self.get_bc())),
            0x1A => self.ld_a(self.read_byte(bus, self.get_de())),
            0x7E => self.ld_a(self.read_byte(bus, self.get_hl())),
            0xFA => {
                let v = self.get_immediate_u16(bus);
                self.ld_a(self.read_byte(bus, v));
            }
            0x3E => {
                let v = self.get_immediate_u8(bus);
                self.ld_a(v);
            }

            0x40 => self.ld_b(self.b),
            0x41 => self.ld_b(self.c),
            0x42 => self.ld_b(self.d),
            0x43 => self.ld_b(self.e),
            0x44 => self.ld_b(self.h),
            0x45 => self.ld_b(self.l),
            0x46 => self.ld_b(self.read_byte(bus, self.get_hl())),

            0x48 => self.ld_c(self.b),
            0x49 => self.ld_c(self.c),
            0x4A => self.ld_c(self.d),
            0x4B => self.ld_c(self.e),
            0x4C => self.ld_c(self.h),
            0x4D => self.ld_c(self.l),
            0x4E => self.ld_c(self.read_byte(bus, self.get_hl())),

            0x50 => self.ld_d(self.b),
            0x51 => self.ld_d(self.c),
            0x52 => self.ld_d(self.d),
            0x53 => self.ld_d(self.e),
            0x54 => self.ld_d(self.h),
            0x55 => self.ld_d(self.l),
            0x56 => self.ld_d(self.read_byte(bus, self.get_hl())),

            0x58 => self.ld_e(self.b),
            0x59 => self.ld_e(self.c),
            0x5A => self.ld_e(self.d),
            0x5B => self.ld_e(self.e),
            0x5C => self.ld_e(self.h),
            0x5D => self.ld_e(self.l),
            0x5E => self.ld_e(self.read_byte(bus, self.get_hl())),

            0x60 => self.ld_h(self.b),
            0x61 => self.ld_h(self.c),
            0x62 => self.ld_h(self.d),
            0x63 => self.ld_h(self.e),
            0x64 => self.ld_h(self.h),
            0x65 => self.ld_h(self.l),
            0x66 => self.ld_h(self.read_byte(bus, self.get_hl())),

            0x68 => self.ld_l(self.b),
            0x69 => self.ld_l(self.c),
            0x6A => self.ld_l(self.d),
            0x6B => self.ld_l(self.e),
            0x6C => self.ld_l(self.h),
            0x6D => self.ld_l(self.l),
            0x6E => self.ld_l(self.read_byte(bus, self.get_hl())),

            0x70 => self.ld_hl(self.b, bus),
            0x71 => self.ld_hl(self.c, bus),
            0x72 => self.ld_hl(self.d, bus),
            0x73 => self.ld_hl(self.e, bus),
            0x74 => self.ld_hl(self.h, bus),
            0x75 => self.ld_hl(self.l, bus),
            0x36 => {
                let n = self.get_immediate_u8(bus);
                self.ld_hl(n, bus);
            }

            // 4. LD n, A
            // 0x7F => self.ld_a(self.a),
            0x47 => self.ld_b(self.a),
            0x4F => self.ld_c(self.a),
            0x57 => self.ld_d(self.a),
            0x5F => self.ld_e(self.a),
            0x67 => self.ld_h(self.a),
            0x6F => self.ld_l(self.a),
            0x02 => self.write_byte(bus, self.get_bc(), self.a),
            0x12 => self.write_byte(bus, self.get_de(), self.a),
            0x77 => self.write_byte(bus, self.get_hl(), self.a),
            0xEA => {
                let addr = self.get_immediate_u16(bus);
                self.write_byte(bus, addr, self.a)
            }

            // 5. LD A, (C)
            0xF2 => {
                let addr: u16 = 0xff00 + (self.c as u16);

                self.a = self.read_byte(bus, addr);
            }

            // 6. LD (C), A
            0xE2 => {
                let addr: u16 = 0xff00 + (self.c as u16);
                self.write_byte(bus, addr, self.a);
            }

            // 7, 8, 9:
            // LD A, (HLD)
            // LD A, (HL-)
            // LDD A, (HL)
            0x3a => {
                let hl = self.hld();
                self.a = self.read_byte(bus, hl);
            }

            // 10, 11, 12:
            // LD (HLD), A
            // LD (HL-), A
            // LDD (HL), A
            0x32 => {
                let hl = self.hld();

                self.write_byte(bus, hl, self.a);
            }

            // 13, 14, 15:
            // LD A, (HLI)
            // LD A, (HL+)
            // LDI A, (HL)
            0x2a => {
                self.lda_hli(bus);
            }

            // 16, 17, 18:
            // LD (HLI), A
            // LD (HL+), A
            // LDI (HL), A
            0x22 => self.ldi_hla(bus),

            // 19. LDH (n), A
            0xe0 => {
                let n = self.get_immediate_u8(bus) as u16;
                let addr = 0xff00 + n;
                self.write_byte(bus, addr, self.a);
            }

            // 20. LDH A, (n)
            0xF0 => {
                let n = self.get_immediate_u8(bus) as u16;
                let addr = 0xff00 + n;
                self.a = self.read_byte(bus, addr);
            }

            // 3.3.2 16-bit loads
            // 1. LD n, nn
            0x01 => {
                let nn = self.get_immediate_u16(bus);
                self.set_bc(nn);
            }
            0x11 => {
                let nn = self.get_immediate_u16(bus);
                self.set_de(nn);
            }
            0x21 => {
                let nn = self.get_immediate_u16(bus);
                self.set_hl(nn)
            }
            0x31 => {
                self.sp = self.get_immediate_u16(bus);
            }

            // 2. LD SP, HL
            0xF9 => self.sp = self.get_hl(),

            // 3. LD HL, SP+n
            // 4. LDHL SP, n
            0xF8 => {
                let r = self.add_16_imm(self.sp, bus);
                self.set_hl(r);
            }

            // 5. LD (nn),SP
            0x08 => {
                let lsb_addr = self.get_immediate_u16(bus);
                let msb_addr = lsb_addr.wrapping_add(1);
                let (sp_msb, sp_lsb) = Self::get_bits(self.sp);

                self.write_byte(bus, lsb_addr, sp_lsb);
                self.write_byte(bus, msb_addr, sp_msb);
            }

            // 6. PUSH nn
            0xC5 => self.push_u16(self.get_bc(), bus),
            0xD5 => self.push_u16(self.get_de(), bus),
            0xE5 => self.push_u16(self.get_hl(), bus),
            0xF5 => self.push_u16(self.get_af(), bus),

            // 3.3.3 8-bit ALU

            // 1. ADD A,n
            0x87 => self.add_a(self.a),
            0x80 => self.add_a(self.b),
            0x81 => self.add_a(self.c),
            0x82 => self.add_a(self.d),
            0x83 => self.add_a(self.e),
            0x84 => self.add_a(self.h),
            0x85 => self.add_a(self.l),
            0x86 => self.add_a(self.read_byte(bus, self.get_hl())),
            0xc6 => {
                let n = self.get_immediate_u8(bus);
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
            0x8E => self.adc(self.read_byte(bus, self.get_hl())),
            0xCE => {
                let n = self.get_immediate_u8(bus);
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
            0x96 => self.sub_a(self.read_byte(bus, self.get_hl())),
            0xD6 => {
                let n = self.get_immediate_u8(bus);
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
            0x9e => self.sbc(self.read_byte(bus, self.get_hl())),
            0xDE => {
                let n = self.get_immediate_u8(bus);
                self.sbc(n);
            }

            // 5. AND n
            0xa7 => self.and(self.a),
            0xa0 => self.and(self.b),
            0xa1 => self.and(self.c),
            0xa2 => self.and(self.d),
            0xa3 => self.and(self.e),
            0xa4 => self.and(self.h),
            0xa5 => self.and(self.l),
            0xa6 => self.and(self.read_byte(bus, self.get_hl())),
            0xe6 => {
                let param = self.get_immediate_u8(bus);
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
            0xb6 => self.or(self.read_byte(bus, self.get_hl())),
            0xf6 => {
                let param = self.get_immediate_u8(bus);
                self.or(param)
            }

            // 7. XOR n
            0xAF => self.xor_a(self.a),
            0xA8 => self.xor_a(self.b),
            0xA9 => self.xor_a(self.c),
            0xAA => self.xor_a(self.d),
            0xAB => self.xor_a(self.e),
            0xAC => self.xor_a(self.h),
            0xAD => self.xor_a(self.l),
            0xAE => self.xor_a(self.read_byte(bus, self.get_hl())),
            0xEE => {
                let param = self.get_immediate_u8(bus);
                self.xor_a(param)
            }

            // 8. CP n
            0xBF => self.compare_a_with(self.a),
            0xB8 => self.compare_a_with(self.b),
            0xB9 => self.compare_a_with(self.c),
            0xBA => self.compare_a_with(self.d),
            0xBB => self.compare_a_with(self.e),
            0xBC => self.compare_a_with(self.h),
            0xBD => self.compare_a_with(self.l),
            0xBE => self.compare_a_with(self.read_byte(bus, self.get_hl())),
            0xFE => {
                let param = self.get_immediate_u8(bus);
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
                let n = self.read_byte(bus, hl);
                let nn = n.wrapping_add(1);

                self.reset_and_set_carry_zero(n, nn);
                self.write_byte(bus, hl, nn);
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
                let prev = self.read_byte(bus, hl);
                let r = prev.wrapping_sub(1);
                self.write_byte(bus, hl, r);
                self.dec_flags(prev, r);
            }

            // 3.3.4 16-bit Arithmetic

            // 1. ADD HL,n
            0x09 => self.add_hl_16(self.get_bc()),
            0x19 => self.add_hl_16(self.get_de()),
            0x29 => self.add_hl_16(self.get_hl()),
            0x39 => self.add_hl_16(self.sp),

            // 2. ADD SP,n

            0xE8 => self.sp = self.add_16_imm(self.sp, bus),

            // 3. INC nn
            0x03 => self.set_bc(self.get_bc().wrapping_add(1)),
            0x13 => self.set_de(self.get_de().wrapping_add(1)),
            0x23 => self.set_hl(self.get_hl().wrapping_add(1)),
            0x33 => self.sp = self.sp.wrapping_add(1),

            // 4. DEC nn
            0x0b => self.set_bc(self.get_bc().wrapping_sub(1)),
            0x1b => self.set_de(self.get_de().wrapping_sub(1)),
            0x2b => self.set_hl(self.get_hl().wrapping_sub(1)),
            0x3b => self.sp = self.sp.wrapping_sub(1),

            // 3.3.5 Miscellaneous

            // 2. DAA

            0x27 => {
                self.daa();
            }

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

            // 7. HALT
            0x76 => {
                // assert!(self.interrupt_master_enable, "WARN: HALT while IME==false??");

                self.halted = true;
            }

            // 8. STOP
            0x10 => {
                // No action to be done at STOP?
            }

            // 9. DI
            0xf3 => {
                self.interrupt_master_enable = false;
            }

            // 10. EI
            0xfb => {
                self.interrupt_master_enable = true;
            }

            // 3.3.6 Rotates & shifts

            // 1. RLCA
            0x07 => {
                self.a = self.rlc_8(self.a);
                self.f.remove(Flags::ZERO);
            }

            // 2. RLA
            0x17 => {
                self.a = self.rl_8(self.a);
                self.f.remove(Flags::ZERO);
            }

            // 3. RRCA
            0x0F => {
                self.a = self.rrc_8(self.a);
                self.f.remove(Flags::ZERO);
            }

            // 4. RRA
            0x1F => {
                self.a = self.rr_8(self.a);
                self.f.remove(Flags::ZERO);
            }

            0xCB => {
                let npc = pc + 1;
                self.pc += 1;
                match self.read_byte(bus, npc) {
                    // 3.3.5. Miscellaneous

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
                        let r = self.swap(self.read_byte(bus, hl));
                        self.write_byte(bus, hl, r);
                    }

                    // 3.3.7. Bit Opcodes

                    // 1. BIT b, r
                    op @ 0x40..=0x7f => {
                        let r = op & 0b111;
                        let b = ((op >> 3) & 0b111) as usize;

                        match r {
                            0b111 => Self::bit(b, self.a, &mut self.f),
                            0b000 => Self::bit(b, self.b, &mut self.f),
                            0b001 => Self::bit(b, self.c, &mut self.f),
                            0b010 => Self::bit(b, self.d, &mut self.f),
                            0b011 => Self::bit(b, self.e, &mut self.f),
                            0b100 => Self::bit(b, self.h, &mut self.f),
                            0b101 => Self::bit(b, self.l, &mut self.f),
                            0b110 => Self::bit(b, self.read_byte(bus, self.get_hl()), &mut self.f),
                            _ => unreachable!(),
                        };
                    }

                    // 2. SET b, r
                    op @ 0xc0..=0xff => {
                        let r = op & 0b111;
                        let b = ((op >> 3) & 0b111) as usize;

                        match r {
                            0b111 => { self.a.set_bit(b, true); }
                            0b000 => { self.b.set_bit(b, true); }
                            0b001 => { self.c.set_bit(b, true); }
                            0b010 => { self.d.set_bit(b, true); }
                            0b011 => { self.e.set_bit(b, true); }
                            0b100 => { self.h.set_bit(b, true); }
                            0b101 => { self.l.set_bit(b, true); }
                            0b110 => {
                                let hl = self.get_hl();
                                let mut v = self.read_byte(bus, hl);
                                v.set_bit(b, true);
                                self.write_byte(bus, hl, v);
                            }
                            _ => unreachable!(),
                        };
                    }

                    // 3. RES b,r
                    op @ 0x80..=0xbf => {
                        let r = op & 0b111;
                        let b = ((op >> 3) & 0b111) as usize;

                        match r {
                            0b111 => { self.a.set_bit(b, false); }
                            0b000 => { self.b.set_bit(b, false); }
                            0b001 => { self.c.set_bit(b, false); }
                            0b010 => { self.d.set_bit(b, false); }
                            0b011 => { self.e.set_bit(b, false); }
                            0b100 => { self.h.set_bit(b, false); }
                            0b101 => { self.l.set_bit(b, false); }
                            0b110 => {
                                let hl = self.get_hl();
                                let mut v = self.read_byte(bus, hl);
                                v.set_bit(b, false);
                                self.write_byte(bus, hl, v);
                            }
                            _ => unreachable!(),
                        };
                    }

                    // 3.3.6. Rotates & Shifts

                    // 5. RLC n

                    0x07 => self.a = self.rlc_8(self.a),
                    0x00 => self.b = self.rlc_8(self.b),
                    0x01 => self.c = self.rlc_8(self.c),
                    0x02 => self.d = self.rlc_8(self.d),
                    0x03 => self.e = self.rlc_8(self.e),
                    0x04 => self.h = self.rlc_8(self.h),
                    0x05 => self.l = self.rlc_8(self.l),
                    0x06 => {
                        let hl = self.get_hl();
                        let r = self.rlc_8(self.read_byte(bus, hl));
                        self.write_byte(bus, hl, r);
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
                        let r = self.rl_8(self.read_byte(bus, hl));
                        self.write_byte(bus, hl, r);
                    }

                    // 7. RLC n

                    0x0f => self.a = self.rrc_8(self.a),
                    0x08 => self.b = self.rrc_8(self.b),
                    0x09 => self.c = self.rrc_8(self.c),
                    0x0a => self.d = self.rrc_8(self.d),
                    0x0b => self.e = self.rrc_8(self.e),
                    0x0c => self.h = self.rrc_8(self.h),
                    0x0d => self.l = self.rrc_8(self.l),
                    0x0e => {
                        let hl = self.get_hl();
                        let r = self.rrc_8(self.read_byte(bus, hl));
                        self.write_byte(bus, hl, r);
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
                        let r = self.rr_8(self.read_byte(bus, hl));
                        self.write_byte(bus, hl, r);
                    }

                    // 9. SLA n
                    0x27 => self.a = self.sla_8(self.a),
                    0x20 => self.b = self.sla_8(self.b),
                    0x21 => self.c = self.sla_8(self.c),
                    0x22 => self.d = self.sla_8(self.d),
                    0x23 => self.e = self.sla_8(self.e),
                    0x24 => self.h = self.sla_8(self.h),
                    0x25 => self.l = self.sla_8(self.l),
                    0x26 => {
                        let hl = self.get_hl();
                        let r = self.sla_8(self.read_byte(bus, hl));
                        self.write_byte(bus, hl, r);
                    }

                    // 10. SRA n

                    0x2f => self.a = self.sra_8(self.a),
                    0x28 => self.b = self.sra_8(self.b),
                    0x29 => self.c = self.sra_8(self.c),
                    0x2A => self.d = self.sra_8(self.d),
                    0x2B => self.e = self.sra_8(self.e),
                    0x2C => self.h = self.sra_8(self.h),
                    0x2D => self.l = self.sra_8(self.l),
                    0x2E => {
                        let hl = self.get_hl();
                        let r = self.sra_8(self.read_byte(bus, hl));
                        self.write_byte(bus, hl, r);
                    }

                    // 11. SRL n
                    0x3F => self.a = self.srl_8(self.a),
                    0x38 => self.b = self.srl_8(self.b),
                    0x39 => self.c = self.srl_8(self.c),
                    0x3A => self.d = self.srl_8(self.d),
                    0x3B => self.e = self.srl_8(self.e),
                    0x3C => self.h = self.srl_8(self.h),
                    0x3D => self.l = self.srl_8(self.l),
                    0x3E => {
                        let hl = self.get_hl();
                        let r = self.srl_8(self.read_byte(bus, hl));
                        self.write_byte(bus, hl, r);
                    }
                }
            }

            // 3.3.8 Jumps

            // 1. JP nn
            0xC3 => self.pc = self.get_immediate_u16(bus),

            // 2. JP cc,nn
            0xC2 => {
                let nn = self.get_immediate_u16(bus);
                if !self.f.contains(Flags::ZERO) {
                    self.pc = nn
                }
            }

            0xCA => {
                let nn = self.get_immediate_u16(bus);
                if self.f.contains(Flags::ZERO) {
                    self.pc = nn
                }
            }

            0xD2 => {
                let nn = self.get_immediate_u16(bus);
                if !self.f.contains(Flags::CARRY) {
                    self.pc = nn
                }
            }

            0xDA => {
                let nn = self.get_immediate_u16(bus);
                if self.f.contains(Flags::CARRY) {
                    self.pc = nn
                }
            }

            // 3. JP (HL)
            0xE9 => {
                self.pc = self.get_hl();
            }

            // 4. JR n
            0x18 => {
                let n = self.get_immediate_i8(bus);
                self.pc = ((self.pc as i16) + n as i16) as u16;
            }

            // 5. JR cc,n

            // JR NZ,*
            0x20 => {
                let n = self.get_immediate_i8(bus);
                if !self.f.contains(Flags::ZERO) {
                    self.pc = ((self.pc as i16) + n as i16) as u16;
                }
            }
            // JR Z,*
            0x28 => {
                let n = self.get_immediate_i8(bus);
                if self.f.contains(Flags::ZERO) {
                    self.pc = ((self.pc as i16) + n as i16) as u16;
                }
            }
            // JR NC,*
            0x30 => {
                let n = self.get_immediate_i8(bus);
                if !self.f.contains(Flags::CARRY) {
                    self.pc = ((self.pc as i16) + n as i16) as u16;
                }
            }
            // JR C,*
            0x38 => {
                let n = self.get_immediate_i8(bus);
                if self.f.contains(Flags::CARRY) {
                    self.pc = ((self.pc as i16) + n as i16) as u16;
                }
            }

            // 3.3.9 Calls

            // 1. CALL nn
            0xCD => {
                let nn = self.get_immediate_u16(bus);
                self.call(nn, bus)
            }

            // 2. CALL cc,nn
            0xC4 => {
                let nn = self.get_immediate_u16(bus);
                if !self.f.contains(Flags::ZERO) {
                    self.call(nn, bus);
                }
            }

            0xCC => {
                let nn = self.get_immediate_u16(bus);
                if self.f.contains(Flags::ZERO) {
                    self.call(nn, bus);
                }
            }

            0xD4 => {
                let nn = self.get_immediate_u16(bus);
                if !self.f.contains(Flags::CARRY) {
                    self.call(nn, bus);
                }
            }

            0xDC => {
                let nn = self.get_immediate_u16(bus);
                if self.f.contains(Flags::CARRY) {
                    self.call(nn, bus);
                }
            }

            // 3.3.10 Restarts

            // 1. RST n
            0xC7 => self.rst(0x00, bus),
            0xCF => self.rst(0x08, bus),
            0xD7 => self.rst(0x10, bus),
            0xDF => self.rst(0x18, bus),
            0xE7 => self.rst(0x20, bus),
            0xEF => self.rst(0x28, bus),
            0xF7 => self.rst(0x30, bus),
            0xFF => self.rst(0x38, bus),

            // 3.3.11 Returns

            // 1. RET
            0xC9 => {
                self.ret(bus)
            }

            // 2. RET cc
            0xC0 => {
                if !self.f.contains(Flags::ZERO) {
                    self.ret(bus)
                }
            }

            0xC8 => {
                if self.f.contains(Flags::ZERO) {
                    self.ret(bus)
                }
            }

            0xD0 => {
                if !self.f.contains(Flags::CARRY) {
                    self.ret(bus)
                }
            }

            0xD8 => {
                if self.f.contains(Flags::CARRY) {
                    self.ret(bus)
                }
            }

            // 3. RETI
            0xD9 => {
                self.ret(bus);
                self.interrupt_master_enable = true;
            }


            // 7. POP nn
            0xC1 => {
                self.c = self.read_sp_u8(bus);
                self.b = self.read_sp_u8(bus);
            }
            0xD1 => {
                self.e = self.read_sp_u8(bus);
                self.d = self.read_sp_u8(bus);
            }
            0xE1 => {
                self.l = self.read_sp_u8(bus);
                self.h = self.read_sp_u8(bus);
            }
            0xF1 => {
                self.f.bits = self.read_sp_u8(bus) & 0xf0;
                self.a = self.read_sp_u8(bus);
            }

            _ => {
                println!(
                    "Unimplemented at pc={:x}, op={:x}: {}",
                    pc,
                    self.read_byte(bus, pc),
                    lookup_op_code(self.read_byte(bus, pc)).0
                );
                println!("{:?}", self);
                unimplemented!()
            }
        }

        self.lookup_op_code_for_pc(bus, pc).1
    }

    fn set_slr_flags(&mut self, c: bool, r: u8) {
        self.f.set(Flags::ZERO, r == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.set(Flags::CARRY, c);
    }


    fn sla_8(&mut self, v: u8) -> u8 {
        let c = (0x80 & v) == 0x80;
        let r = v << 1;
        self.set_slr_flags(c, r);

        r
    }

    fn sra_8(&mut self, v: u8) -> u8 {
        let c = v & 0x01 == 0x01;
        let r = (v >> 1) | (v & 0x80);
        self.set_slr_flags(c, r);

        r
    }

    fn srl_8(&mut self, v: u8) -> u8 {
        let c = v & 0x01 == 0x01;
        let r = v >> 1;
        self.set_slr_flags(c, r);

        r
    }
}
