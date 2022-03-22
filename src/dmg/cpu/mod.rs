use std::cell::RefCell;
use std::io::{stdout, Write};
use std::ops::{BitAnd, BitOr};
use std::rc::Rc;

use bit_field::BitField;
use bitflags::bitflags;

use crate::dmg::debug::{lookup_cb_prefix_op_code, lookup_op_code};

use super::mem::MemoryBus;

mod step;

bitflags! {
    pub struct Flags: u8 {
        const ZERO = 0b10000000;
        const N = 0b01000000;
        const H = 0b00100000;
        const CARRY = 0b00010000;
        const F = Self::ZERO.bits | Self::N.bits | Self::H.bits | Self::CARRY.bits;
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

    master_interrupt_enabled: bool,
    enable_debugging: bool,

    cycles: u32,
}


impl ProcessingUnit {
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
            master_interrupt_enabled: false,
            bus,

            enable_debugging: false,

            cycles: 0,
        }
    }


    fn swap(&mut self, n: u8) -> u8 {
        let r = n.swap_bytes();

        self.f.set(Flags::ZERO, r == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.remove(Flags::CARRY);

        r
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
        let (msb, lsb) = self.get_immediate_u16_tuple();

        ((msb as u16) << 8) | (lsb as u16)
    }

    fn get_immediate_u16_tuple(&mut self) -> (u8, u8) {
        let v = (self.read_byte(self.pc + 1), self.read_byte(self.pc));
        self.pc += 2;
        v
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        // if addr == 0xff00 {
        //     println!("Write {:04x} {:x}", addr, value);
        // }

        // if addr == 0xff50 {
        //     self.enable_debugging = true;
        // }

        match addr {
            0xff01 => {
                println!("{:?} {:02x}", value as char, value);
                stdout().flush().expect("No flush?");
            }
            _ => {
                self.bus.borrow_mut().write_byte(addr, value)
            }
        }
    }

    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff00 => self.read_joypad(),
            _ => self.bus.borrow().read_byte(addr)
        }
    }


    fn read_joypad(&self) -> u8 {
        // TODO: right now joypad is hard-coded to no buttons pressed
        0xef
    }

    pub fn debug_print(&self, pc: u16) {
        if !self.enable_debugging {
            return;
        }

        let op_code = self.lookup_op_code_for_pc(pc).0;


        // Logging registers in pairs like bgb
        // println!("{:5x}: {:<10}\ta: {:2x}, b: {:2x}, c: {:2x}, d: {:2x}, e: {:2x}, h: {:2x}, l: {:2x}, sp: {:4x}, flags: {:?}", pc, op_code, self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.f)
        println!("{:5x}: {:<10}\taf: {:04x}, bc: {:04x}, de: {:04x}, hl: {:04x}, sp: {:4x}, flags: {:?}", pc, op_code, self.get_af(), self.get_bc(), self.get_de(), self.get_hl(), self.sp, self.f)
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
        // if let Some(count) = self.serial_countdown {
        //     if count >= self.cycles {
        //         self.mem.gpu.intf.insert(InterruptFlag::SERIAL);
        //     }
        // }



        if self.master_interrupt_enabled && self.bus.borrow().interrupts_enabled.intersects(self.bus.borrow().ppu.intf) {
            let interrupt_flags = self.bus.borrow().ppu.intf;
            if let Some(addr) = interrupt_flags.interrupt_starting_address() {
                self.master_interrupt_enabled = false;
                let triggered = interrupt_flags.highest_prio_bit();

                // println!("Handle interrupt {:?}", triggered);
                {
                    self.bus.borrow_mut().ppu.intf.remove(triggered);
                }
                self.push_u16(self.pc);
                self.pc = addr;
            }
        }
    }

    fn add_hl_16(&mut self, hl: u16) {
        let prev = self.get_hl();
        let (new_hl, overflow) = prev.overflowing_add(hl);
        self.f.remove(Flags::N);

        self.f.set(Flags::H, (((prev & 0xfff) + (hl & 0xfff)) & 0x1000) > 0);
        self.f.set(Flags::CARRY, overflow);
        self.set_hl(new_hl);
    }

    fn lda_hli(&mut self) {
        let hl = self.get_hl();
        self.a = self.read_byte(hl);

        self.set_hl(hl.wrapping_add(1));
    }

    fn ldi_hla(&mut self) {
        self.ld_hl(self.a);

        let n = self.get_hl();
        let nn = n.wrapping_add(1);
        self.set_hl(nn)
    }

    fn call(&mut self) {
        let nn = self.get_immediate_u16();
        self.push_u16(self.pc);
        self.pc = nn;
    }

    fn ret(&mut self) {
        let lsb = self.read_sp_u8() as u16;
        let msb = self.read_sp_u8() as u16;

        let dest = (msb << 8) | lsb;
        self.pc = dest
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

    fn rrca_8(&mut self, v: u8) -> u8 {
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

    fn rlca_8(&mut self, v: u8) -> u8 {
        let carry = v.get_bit(7);
        let mut v = v << 1;
        v.set_bit(0, carry);


        self.f.set(Flags::ZERO, v == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.set(Flags::CARRY, carry);

        v
    }

    fn xor(&mut self, n: u8) {
        self.a = self.a ^ n;
        self.reset_and_set_zero(self.a);
    }

    fn push_u8(&mut self, n: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, n);
    }

    fn push_u16(&mut self, n: u16) {
        let (n_msb, n_lsb) = (((n & 0xff00) >> 8) as u8, (n & 0xff) as u8);
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

    fn read_sp_u8(&mut self) -> u8 {
        let x = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);

        return x;
    }

    fn dec_flags(&mut self, prev: u8, n: u8) {
        self.f.set(Flags::ZERO, n == 0);
        self.f.insert(Flags::N);
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
        self.set_half_carry(self.a, nn);
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
        let r = self.a.bitand(n);
        self.f.set(Flags::ZERO, r == 0);
        self.f.remove(Flags::N);
        self.f.insert(Flags::H);
        self.f.remove(Flags::CARRY);
        self.a = r;
    }

    fn or(&mut self, n: u8) {
        let r = self.a.bitor(n);
        self.f.set(Flags::ZERO, r == 0);
        self.f.remove(Flags::N);
        self.f.remove(Flags::H);
        self.f.remove(Flags::CARRY);
        self.a = r;
    }

    fn rst(&mut self, addr: u16) {
        self.push_u16(self.pc);
        self.pc = addr;
    }

    fn adc(&mut self, n: u8) {
        let (aa, overflow) = self.a.overflowing_add(n + if self.f.contains(Flags::CARRY) { 1 } else { 0 });

        self.f.set(Flags::ZERO, aa == 0);
        self.f.remove(Flags::N);
        self.set_half_carry(self.a, aa);
        self.f.set(Flags::CARRY, overflow);

        self.a = aa;
    }

    fn sbc(&mut self, n: u8) {
        let (aa, overflow) = self.a.overflowing_sub(n.wrapping_add(if self.f.contains(Flags::CARRY) { 1 } else { 0 }));


        self.f.set(Flags::ZERO, aa == 0);
        self.f.remove(Flags::N);
        self.set_half_carry(self.a, aa);
        self.f.set(Flags::CARRY, overflow);

        self.a = aa;
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

        cpu.xor(cpu.a);


        assert_eq!(cpu.a, 0x00);

        assert!(cpu.f.contains(Flags::ZERO));
    }

    #[test]
    fn xor_0f_works() {
        let mut cpu = setup_cpu_for_xor();

        cpu.xor(0x0f);


        assert_eq!(cpu.a, 0xf0);

        assert!(!cpu.f.contains(Flags::ZERO));
    }

    #[test]
    fn xor_hl_works() {
        let mut cpu = setup_cpu_for_xor();

        cpu.xor(cpu.read_byte(cpu.get_hl()));


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
