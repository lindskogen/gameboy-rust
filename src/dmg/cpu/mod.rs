use std::cell::RefCell;
use std::rc::Rc;

use bit_field::BitField;
use bitflags::bitflags;

use crate::dmg::debug::{lookup_cb_prefix_op_code, lookup_op_code};

use super::mem::MemoryBus;

mod step;

bitflags! {
    pub struct Flags: u8 {
        const ZERO = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const CARRY = 0b0001_0000;
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

    bus: Rc<RefCell<MemoryBus>>,

    halted: bool,
    interrupt_master_enable: bool,
    enable_debugging: bool,
}

impl ProcessingUnit {
    pub fn initialize_gameboy_doctor(&mut self) {
        self.enable_debugging = true;
        self.a = 0x01;
        self.f = Flags::CARRY | Flags::H | Flags::ZERO;
        assert_eq!(self.f.bits, 0xb0);
        self.b = 0x00;
        self.c = 0x13;
        self.d = 0x00;
        self.e = 0xd8;
        self.h = 0x01;
        self.l = 0x4d;
        self.sp = 0xFFFE;
        self.pc = 0x0100;
    }

    pub fn new(bus: Rc<RefCell<MemoryBus>>) -> ProcessingUnit {
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
            halted: false,
            interrupt_master_enable: false,
            bus,

            enable_debugging: false,
        }
    }

    fn swap(&mut self, n: u8) -> u8 {

        self.f.set(Flags::ZERO, n == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.remove(Flags::CARRY);

        (n >> 4) | (n << 4)
    }

    fn get_carry(&self) -> u8 {
        if self.f.contains(Flags::CARRY) {
            1
        } else {
            0
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
        let v = self.read_byte(self.pc);
        self.pc += 1;
        v
    }

    fn get_immediate_i8(&mut self) -> i8 {
        let v = self.read_byte(self.pc) as i8;
        self.pc += 1;
        v
    }

    fn get_immediate_u16(&mut self) -> u16 {
        let (msb, lsb) = (self.read_byte(self.pc + 1), self.read_byte(self.pc));
        self.pc += 2;

        ((msb as u16) << 8) | (lsb as u16)
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        self.bus.borrow_mut().write_byte(addr, value);
    }

    fn daa(&mut self) {
        let mut adjust = if self.f.contains(Flags::CARRY) { 0x60 } else { 0 };

        if self.f.contains(Flags::H) { adjust |= 0x06; };

        if !self.f.contains(Flags::N) {
            if self.a & 0x0f > 0x09 { adjust |= 0x06; };
            if self.a > 0x99 { adjust |= 0x60; };
            self.a = self.a.wrapping_add(adjust);
        } else {
            self.a = self.a.wrapping_sub(adjust);
        }

        self.f.set(Flags::CARRY, adjust >= 0x60);
        self.f.remove(Flags::H);
        self.f.set(Flags::ZERO, self.a == 0);
    }

    fn read_byte(&self, addr: u16) -> u8 {
        self.bus.borrow().read_byte(addr)
    }

    pub fn debug_print(&self, pc: u16) {
        if self.enable_debugging {
            println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}", self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, pc, self.read_byte(pc), self.read_byte(pc+1), self.read_byte(pc+2), self.read_byte(pc+3));
        }

    }

    fn lookup_op_code_for_pc(&self, pc: u16) -> (&str, u32) {
        if self.read_byte(pc) != 0xCB {
            lookup_op_code(self.read_byte(pc))
        } else {
            lookup_cb_prefix_op_code(self.read_byte(pc + 1))
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

    fn ld_hl(&mut self, n: u8) {
        self.write_byte(self.get_hl(), n);
    }

    fn check_and_execute_interrupts(&mut self) {
        if self.interrupt_master_enable
            && self
            .bus
            .borrow()
            .interrupt_enable
            .intersects(self.bus.borrow().ppu.interrupt_flag)
        {
            let interrupt_flags = self.bus.borrow().ppu.interrupt_flag;
            if let Some(addr) = interrupt_flags.interrupt_starting_address() {
                self.interrupt_master_enable = false;
                let triggered = interrupt_flags.highest_prio_bit();

                // eprintln!("-- Handle interrupt {:?}", triggered);

                self.bus.borrow_mut().ppu.interrupt_flag.remove(triggered);
                self.halted = false;
                self.call(addr);
            }
        }
    }

    fn add_16_imm(&mut self, a: u16) -> u16 {
        let b = self.get_immediate_i8() as i16 as u16;

        self.f.remove(Flags::N);
        self.f.remove(Flags::ZERO);
        self.f.set(Flags::H, (a & 0xf) + (b & 0xf) > 0xf);
        self.f.set(Flags::CARRY, (a & 0xff) + (b & 0xff) > 0xff);

        a.wrapping_add(b)
    }

    fn add_hl_16(&mut self, hl: u16) {
        let prev = self.get_hl();
        let (new_hl, overflow) = prev.overflowing_add(hl);
        self.f.remove(Flags::N);

        let h_flag = (((prev & 0xfff) + (hl & 0xfff)) & 0x1000) > 0;
        self.f.set(Flags::H, h_flag);
        self.f.set(Flags::CARRY, overflow);
        self.set_hl(new_hl);
    }

    fn lda_hli(&mut self) {
        let hl = self.hli();
        self.a = self.read_byte(hl);
    }

    fn hli(&mut self) -> u16 {
        let hl = self.get_hl();
        self.set_hl(hl.wrapping_add(1));
        hl
    }

    fn hld(&mut self) -> u16 {
        let hl = self.get_hl();
        self.set_hl(hl.wrapping_sub(1));
        hl
    }

    fn ldi_hla(&mut self) {
        self.ld_hl(self.a);

        self.hli();
    }

    fn call(&mut self, nn: u16) {
        self.push_u16(self.pc);
        self.pc = nn;
    }

    fn rr_8(&mut self, v: u8) -> u8 {
        let carry = v.get_bit(0);
        let mut v = v >> 1;
        v.set_bit(7, self.f.contains(Flags::CARRY));

        self.f.set(Flags::ZERO, v == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.set(Flags::CARRY, carry);

        v
    }

    fn rrc_8(&mut self, v: u8) -> u8 {
        let carry = v.get_bit(0);
        let mut v = v >> 1;
        v.set_bit(7, carry);

        self.f.set(Flags::ZERO, v == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.set(Flags::CARRY, carry);

        v
    }

    fn rl_8(&mut self, v: u8) -> u8 {
        let carry = v.get_bit(7);
        let mut v = v << 1;
        v.set_bit(0, self.f.contains(Flags::CARRY));

        self.f.set(Flags::ZERO, v == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.set(Flags::CARRY, carry);

        v
    }

    fn rlc_8(&mut self, v: u8) -> u8 {
        let carry = v.get_bit(7);
        let mut v = v << 1;
        v.set_bit(0, carry);

        self.f.set(Flags::ZERO, v == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.set(Flags::CARRY, carry);

        v
    }

    fn xor_a(&mut self, n: u8) {
        self.a = self.a ^ n;
        self.reset_and_set_zero(self.a);
    }

    fn push_u8(&mut self, n: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, n);
    }

    fn get_bits(n: u16) -> (u8, u8) {
        (((n & 0xff00) >> 8) as u8, (n & 0xff) as u8)
    }

    fn push_u16(&mut self, n: u16) {
        let (n_msb, n_lsb) = Self::get_bits(n);
        self.push_u8(n_msb);
        self.push_u8(n_lsb);
    }

    fn reset_and_set_carry_zero(&mut self, prev: u8, new: u8) {
        self.f.set(Flags::ZERO, new == 0);
        self.f.set(Flags::H, (((prev & 0xf) + 1) & 0x10) == 0x10);
        self.f.remove(Flags::N);
    }

    fn reset_and_set_zero(&mut self, n: u8) {
        self.f.bits = 0;
        self.f.set(Flags::ZERO, n == 0);
    }

    fn read_sp_u16(&mut self) -> u16 {
        let lsb = self.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let msb = self.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    fn read_sp_u8(&mut self) -> u8 {
        let x = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);

        return x;
    }

    fn dec_flags(&mut self, prev: u8, n: u8) {
        self.f.set(Flags::ZERO, n == 0);
        self.f.insert(Flags::N);
        self.set_half_carry(prev, n);
        self.f.set(Flags::H, (prev & 0xf0) != (n & 0xf0));
    }

    fn set_half_carry(&mut self, prev: u8, n: u8) {
        // H: Set if no borrow from bit 4
        self.f.set(Flags::H, (prev & 0xf0) != (n & 0xf0))
    }

    fn compare_a_with(&mut self, n: u8) {
        let (r, did_overflow) = self.a.overflowing_sub(n);
        self.f.set(Flags::ZERO, r == 0);
        self.f.insert(Flags::N);
        self.f.set(Flags::H, (self.a & 0xf) < (n & 0xf));
        self.f.set(Flags::CARRY, did_overflow);
    }

    fn sub_a(&mut self, n: u8) {
        let (nn, overflow) = self.a.overflowing_sub(n);
        self.f.set(Flags::ZERO, nn == 0);
        self.f.insert(Flags::N);
        self.f.set(Flags::H, (self.a & 0xf) < (n & 0xf));
        self.f.set(Flags::CARRY, overflow);
        self.a = nn;
    }

    fn add_a(&mut self, n: u8) {
        let (nn, overflow) = self.a.overflowing_add(n);
        self.f.set(Flags::ZERO, nn == 0);
        self.f.remove(Flags::N);
        self.f.set(Flags::H, (((self.a & 0xf) + (n & 0xf)) & 0x10) > 0);
        self.f.set(Flags::CARRY, overflow);
        self.a = nn;
    }

    fn and(&mut self, n: u8) {
        self.a =  self.a & n;
        self.reset_and_set_zero(self.a);
        self.f.insert(Flags::H);
    }

    fn or(&mut self, n: u8) {
        self.a = self.a | n;
        self.reset_and_set_zero(self.a);
    }

    fn rst(&mut self, addr: u16) {
        self.push_u16(self.pc);
        self.pc = addr;
    }

    fn adc(&mut self, n: u8) {
        let carry = self.get_carry();
        let r = self.a.wrapping_add(n).wrapping_add(carry);

        self.f.set(Flags::ZERO, r == 0);
        self.f.remove(Flags::N);
        self.f.set(Flags::H, (self.a & 0xf) + (n & 0xf) + carry > 0xf);
        self.f.set(Flags::CARRY, (self.a as u16) + (n as u16) + (carry as u16) > 0xff);

        self.a = r;
    }

    fn bit(bit: usize, reg: u8, flags: &mut Flags) {
        flags.set(Flags::ZERO, !reg.get_bit(bit));
        flags.remove(Flags::N);
        flags.insert(Flags::H);
    }

    fn sbc(&mut self, n: u8) {
        let carry = self.get_carry();
        let r = self.a.wrapping_sub(n).wrapping_sub(carry);

        self.f.set(Flags::ZERO, r == 0);
        self.f.set(Flags::H, (self.a & 0x0f) < (n & 0x0f) + carry);
        self.f.insert(Flags::N);
        self.f.set(Flags::CARRY,  (self.a as u16) < (n as u16) + (carry as u16));

        self.a = r;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::dmg::cpu::{Flags, ProcessingUnit};
    use crate::dmg::mem::MemoryBus;

    fn setup_cpu_for_compare() -> ProcessingUnit {
        let bootloader = [0x40u8; 256];
        let mut cpu = ProcessingUnit::new(Rc::new(RefCell::new(MemoryBus::new(bootloader, None))));

        cpu.f = Flags::empty();
        cpu.a = 0x3c;
        cpu.b = 0x2f;
        cpu.set_hl(0x10);

        cpu
    }

    #[test]
    fn cp_b_works() {
        let mut cpu = setup_cpu_for_compare();

        cpu.compare_a_with(cpu.b);

        assert!(!cpu.f.contains(Flags::ZERO));
        assert!(cpu.f.contains(Flags::H));
        assert!(cpu.f.contains(Flags::N));
        assert!(!cpu.f.contains(Flags::CARRY));
    }

    #[test]
    fn cp_3c_works() {
        let mut cpu = setup_cpu_for_compare();

        cpu.compare_a_with(0x3c);

        assert!(cpu.f.contains(Flags::ZERO));
        assert!(!cpu.f.contains(Flags::H));
        assert!(cpu.f.contains(Flags::N));
        assert!(!cpu.f.contains(Flags::CARRY));
    }

    #[test]
    fn cp_hl_works() {
        let mut cpu = setup_cpu_for_compare();

        cpu.compare_a_with(cpu.read_byte(cpu.get_hl()));

        assert!(!cpu.f.contains(Flags::ZERO));
        assert!(!cpu.f.contains(Flags::H));
        assert!(cpu.f.contains(Flags::N));
        assert!(cpu.f.contains(Flags::CARRY));
    }

    fn setup_cpu_for_add_hl() -> ProcessingUnit {
        let bootloader = [0u8; 256];
        let mut cpu = ProcessingUnit::new(Rc::new(RefCell::new(MemoryBus::new(bootloader, None))));

        cpu.f = Flags::empty();
        cpu.set_hl(0x8a23);
        cpu.set_bc(0x0605);

        cpu
    }

    #[test]
    fn add_hl_bc_works() {
        let mut cpu = setup_cpu_for_add_hl();

        cpu.add_hl_16(cpu.get_bc());

        assert_eq!(cpu.get_hl(), 0x9028);

        assert!(!cpu.f.contains(Flags::ZERO));
        assert!(cpu.f.contains(Flags::H));
        assert!(!cpu.f.contains(Flags::N));
        assert!(!cpu.f.contains(Flags::CARRY));
    }

    #[test]
    fn add_hl_hl_works() {
        let mut cpu = setup_cpu_for_add_hl();

        cpu.add_hl_16(cpu.get_hl());

        assert_eq!(cpu.get_hl(), 0x1446);

        assert!(!cpu.f.contains(Flags::ZERO));
        assert!(cpu.f.contains(Flags::H));
        assert!(!cpu.f.contains(Flags::N));
        assert!(cpu.f.contains(Flags::CARRY));
    }

    // LDI

    fn setup_cpu_for_ldi() -> ProcessingUnit {
        let bootloader = [0x56u8; 256];
        let mut cpu = ProcessingUnit::new(Rc::new(RefCell::new(MemoryBus::new(bootloader, None))));

        cpu.f = Flags::empty();
        cpu.set_hl(0x0ff);

        cpu
    }

    #[test]
    fn ldi_a_works() {
        let mut cpu = setup_cpu_for_ldi();

        cpu.lda_hli();

        assert_eq!(cpu.a, 0x56);
        assert_eq!(cpu.get_hl(), 0x100);
    }

    // LDI

    fn setup_cpu_for_ldi_hla() -> ProcessingUnit {
        let bootloader = [0x56u8; 256];
        let mut cpu = ProcessingUnit::new(Rc::new(RefCell::new(MemoryBus::new(bootloader, None))));

        cpu.a = 0x56;
        cpu.f = Flags::empty();
        cpu.set_hl(0x67);

        cpu
    }

    #[test]
    fn ldi_hla_works() {
        let mut cpu = setup_cpu_for_ldi_hla();

        cpu.ldi_hla();

        assert_eq!(cpu.read_byte(cpu.get_hl()), 0x56);
        assert_eq!(cpu.get_hl(), 0x68);
    }

    // XOR

    fn setup_cpu_for_xor() -> ProcessingUnit {
        let bootloader = [0x8au8; 256];
        let mut cpu = ProcessingUnit::new(Rc::new(RefCell::new(MemoryBus::new(bootloader, None))));

        cpu.a = 0xff;
        cpu.f = Flags::empty();
        cpu.set_hl(0x10);

        cpu
    }

    #[test]
    fn xor_a_works() {
        let mut cpu = setup_cpu_for_xor();

        cpu.xor_a(cpu.a);

        assert_eq!(cpu.a, 0x00);

        assert!(cpu.f.contains(Flags::ZERO));
    }

    #[test]
    fn xor_0f_works() {
        let mut cpu = setup_cpu_for_xor();

        cpu.xor_a(0x0f);

        assert_eq!(cpu.a, 0xf0);

        assert!(!cpu.f.contains(Flags::ZERO));
    }

    #[test]
    fn xor_hl_works() {
        let mut cpu = setup_cpu_for_xor();

        cpu.xor_a(cpu.read_byte(cpu.get_hl()));

        assert_eq!(cpu.a, 0x75);

        assert!(!cpu.f.contains(Flags::ZERO));
    }

    // OR

    fn setup_cpu_for_or() -> ProcessingUnit {
        let bootloader = [0x0fu8; 256];
        let mut cpu = ProcessingUnit::new(Rc::new(RefCell::new(MemoryBus::new(bootloader, None))));

        cpu.a = 0x5a;
        cpu.f = Flags::empty();
        cpu.set_hl(0x10);

        cpu
    }

    #[test]
    fn or_a_works() {
        let mut cpu = setup_cpu_for_or();

        cpu.or(cpu.a);

        assert_eq!(cpu.a, 0x5a);

        assert!(!cpu.f.contains(Flags::ZERO));
    }

    #[test]
    fn or_3_works() {
        let mut cpu = setup_cpu_for_or();

        cpu.or(3);

        assert_eq!(cpu.a, 0x5b);

        assert!(!cpu.f.contains(Flags::ZERO));
    }

    #[test]
    fn or_hl_works() {
        let mut cpu = setup_cpu_for_or();

        cpu.or(cpu.read_byte(cpu.get_hl()));

        assert_eq!(cpu.a, 0x5f);

        assert!(!cpu.f.contains(Flags::ZERO));
    }
}
