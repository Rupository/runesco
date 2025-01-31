use std::collections::HashMap;
use crate::{bus::Bus, opcodes};


pub struct CPU<'a> { // CPU with..  
    pub register_a: u8, // Accumulator A
    pub register_x: u8, // Register X
    pub register_y: u8, // Register Y
    pub stack_pointer: u8, // Stack Pointer
    pub status: u8, // Status flags [NV_BDIZC]
    pub program_counter: u16, // Program Counter
    pub bus: Bus<'a>,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    // Specifically, addressing modes that are not implied, relative, or indirect
    // which can be done implicitly with opcode implementation. These are covered
    // under NoneAddressing
   Immediate,
   ZeroPage,
   ZeroPage_X,
   ZeroPage_Y,
   Absolute,
   Absolute_X,
   Absolute_Y,
   Indirect_X,
   Indirect_Y,
   NoneAddressing,
}

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8; 

    fn mem_write(&mut self, addr: u16, data: u8);
    
    fn mem_read_u16(&mut self, pos: u16) -> u16 { // read little endian, u8 + u8 data read as u16
        let lo = self.mem_read(pos);
        let hi = self.mem_read(pos + 1);
        u16::from_le_bytes([lo,hi]) // Converts to full memory address: $hilo
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) { // write little endian, u16 data written as u8 + u8
        let hi = (data >> 8) as u8; 
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

impl Mem for CPU<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }
 
    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }
  
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

fn page_cross(addr1: u16, addr2 : u16) -> bool {
    addr1 & 0xFF00 != addr2 & 0xFF00
}

mod interrupt {
    #[derive(PartialEq, Eq)]
    pub enum InterruptType {
        NMI,
    }

    #[derive(PartialEq, Eq)]

    // pub: Normally, this makes an item public, meaning it is accessible from any other module.
    // super: This refers to the parent module of the current module. The super keyword is used to access 
    // the parent scope in Rust, specifically the CPU here
    pub(super) struct Interrupt {
        pub(super) itype: InterruptType, // NMI/IRQ/BRK (some unimplemented)
        pub(super) vector_addr: u16, // Location PPU jumps to on an interrupt
        pub(super) b_flag_mask: u8, // ensures correctly masked flags [?]
        pub(super) cpu_cycles: u8, // cycles the interrupt consumes
    }
    pub(super) const NMI: Interrupt = Interrupt {
        itype: InterruptType::NMI,
        vector_addr: 0xfffA,
        b_flag_mask: 0b00100000,
        cpu_cycles: 2,
    };
}

impl<'a> CPU<'a> {
    
    pub fn new<'b>(bus: Bus<'b>) -> CPU<'b> {

        // Lifetimes in CPU Initialization
        // There are two lifetime annotations here: 'a and 'b.

        // - 'a: This is a lifetime parameter for the CPU struct itself. It indicates that the CPU struct contains
        //  references that must be valid for the lifetime 'a.
        // - 'b: This is a lifetime parameter for the new function itself. It allows new to accept a Bus reference 
        // with a potentially different lifetime 'b and then return a CPU instance with a lifetime tied to 'b.

        // The purpose of using these lifetimes is to make sure that the CPU struct can borrow the Bus struct for 
        // as long as the Bus struct itself is valid, avoiding any invalid references.

        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stack_pointer: 0xfd,
            status: 0b100100,
            program_counter: 0,
            bus: bus,
        }
    }

    pub fn reset(&mut self) { // resets when new cartridge is loaded
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0b100100;

        self.stack_pointer = 0xfd;
 
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        for i in 0..(program.len() as u16) { // Write the program at the ROM space, from 0x0600 - 0xXXXX
            self.mem_write(0x0600 + i, program[i as usize]);
        }
        self.mem_write_u16(0xFFFC, 0x0600);
    }

    pub fn get_absolute_address(&mut self, mode: &AddressingMode, addr: u16) -> (u16, bool) {
        // returns address and whether page has been crossed or not
        match mode {
            AddressingMode::ZeroPage => (self.mem_read(addr) as u16, false),
            // Gets u8 address from program counter, of which only 
            // the last two bits of converted the u16 will be relevant.
            // Only access first 256 bytes of memory

            AddressingMode::Absolute => (self.mem_read_u16(addr),false),
            // full u16 address is read, can access 0-65536 bytes.

            AddressingMode::ZeroPage_X => {
                // Takes 0-page address and adds the value stored
                // in the X register to it. Wraps around if $ff, X (X>0)
                let pos = self.mem_read(addr);
                let addr = pos.wrapping_add(self.register_x) as u16;
                (addr, false)
            }
            AddressingMode::ZeroPage_Y => {
                // See 0-page X
                let pos = self.mem_read(addr);
                let addr = pos.wrapping_add(self.register_y) as u16;
                (addr, false)
            }

            AddressingMode::Absolute_X => {
                // Takes absolute address and adds the value stored
                // in the X register to it. Wraps around if $ff, X (X>0)
                let base = self.mem_read_u16(addr);
                let addr = base.wrapping_add(self.register_x as u16);
                (addr, page_cross(base, addr))
            }
            AddressingMode::Absolute_Y => {
                // See absolute X
                let base = self.mem_read_u16(addr);
                let addr = base.wrapping_add(self.register_y as u16);
                (addr, page_cross(base, addr))
            }

            AddressingMode::Indirect_X => {
                // Gets a 0-page memory address
                let base = self.mem_read(addr);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x); // adds what's in X to it
                let lo = self.mem_read(ptr as u16); // reads what's at the pointer
                let hi = self.mem_read(ptr.wrapping_add(1) as u16); // and then at pointer + 1
                (u16::from_le_bytes([lo,hi]), false) // converts to full memory address $hilo

                // A page cross is theoretically possible (See indirect_Y) but none of our opcodes lead to it.
            }
            AddressingMode::Indirect_Y => {
                // Gets a 0-page memory address
                let base = self.mem_read(addr);

                let lo = self.mem_read(base as u16); // reads what's at pointer 
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16); // reads whats at pointer + 1
                let deref_base = u16::from_le_bytes([lo,hi]); // combines into full address, dereferncing base
                let deref = deref_base.wrapping_add(self.register_y as u16); // adds whats's in Y to deref-ed address.
                (deref, page_cross(deref, deref_base))
            }

            _ => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Immediate => (self.program_counter, false),
            // gives whatever hex value is in the instruction as the value to be used.

            _ => self.get_absolute_address(mode, self.program_counter),
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {

        if result == 0 { // if register = 0
            self.status = self.status | 0b0000_0010; 
            // Z (zero flag) set to 1 with bitwise OR
        } else {
            self.status = self.status & 0b1111_1101;
            //otherwise, bitwise AND keeps everything else the same 
            //and sets Z to 0.
        }

        if result & 0b1000_0000 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
            self.status = self.status | 0b1000_0000; 
            // N (negative flag) is set to 1 with bitwise OR
        } else {
            self.status = self.status & 0b0111_1111;
            // N (negative flag) is set to 0  with bitwise AND
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);
       
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);
       
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);
       
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);

        if page_cross {
            self.bus.tick(1);
        }
    }
  
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        if value == 0xff { 
            value = 0;
        } else {
            value += 1;
        }
        self.mem_write(addr, value);

        self.update_zero_and_negative_flags(value);
        // note: Carry is NOT USED! Addition here is in modulo 0xff, loops back to 0.
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        if value == 0 { 
            value = 0xff;
        } else {
            value -= 1;
        }
        self.mem_write(addr, value);

        self.update_zero_and_negative_flags(value);
        // note: Carry is NOT USED! Subtraction here is in modulo 0xff, loops back to 0xff.
    }

    fn inx(&mut self) {
        if self.register_x == 0xff { 
            self.register_x = 0;
        } else {
            self.register_x += 1;
        }
        self.update_zero_and_negative_flags(self.register_x);
        // note: Carry is NOT USED! Addition here is in modulo 0xff, loops back to 0.
    }

    fn dex(&mut self) {
        if self.register_x == 0 { 
            self.register_x = 0xff;
        } else {
            self.register_x -= 1;
        }
        self.update_zero_and_negative_flags(self.register_x);
        // note: Carry is NOT USED! Subtraction here is in modulo 0xff, loops back to 0xff.
    }

    fn dey(&mut self) {
        if self.register_y == 0 { 
            self.register_y = 0xff;
        } else {
            self.register_y -= 1;
        }
        self.update_zero_and_negative_flags(self.register_y);
        // note: Carry is NOT USED! Subtraction here is in modulo 0xff, loops back to 0xff.
    }

    fn iny(&mut self) {
        if self.register_y == 0xff { 
            self.register_y = 0;
        } else {
            self.register_y += 1;
        }
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a & value;
        self.update_zero_and_negative_flags(self.register_a);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a | value;
        self.update_zero_and_negative_flags(self.register_a);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a ^ value;
        self.update_zero_and_negative_flags(self.register_a);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let check = self.register_a.wrapping_sub(value);

        if self.register_a >= value {
            self.sec();
        } else {
            self.clc()
        }
        self.update_zero_and_negative_flags(check);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let check = self.register_x.wrapping_sub(value);

        if self.register_x >= value {
            self.sec();
        } else {
            self.clc()
        }
        self.update_zero_and_negative_flags(check);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let check = self.register_y.wrapping_sub(value);

        if self.register_y >= value {
            self.sec();
        } else {
            self.clc()
        }
        self.update_zero_and_negative_flags(check);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn asl(&mut self, mode: &AddressingMode) {

        if mode == &AddressingMode::NoneAddressing {
            let value = self.register_a;
            if value & 0b1000_0000 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
                self.sec(); 
                // C (carry flag) is set to 1 with bitwise OR, see below
            } else {
                self.clc();
                // C (carry flag) is set to 0  with bitwise AND, see below
            }
            self.register_a = value << 1 ;
            self.update_zero_and_negative_flags(self.register_a);

        } else {
            let (addr, _) = self.get_operand_address(mode);
            let mut value = self.mem_read(addr);
            if value & 0b1000_0000 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
                self.sec(); 
                // C (carry flag) is set to 1 with bitwise OR, see below
            } else {
                self.clc();
                // C (carry flag) is set to 0  with bitwise AND, see below
            }
            value = value << 1;
            self.mem_write(addr, value);

            self.update_zero_and_negative_flags(value);
        }
    }

    fn rol(&mut self, mode: &AddressingMode) {

        let original_carry_flag = self.status & 0b0000_0001;

        if mode == &AddressingMode::NoneAddressing {
            let value = self.register_a;
            if value & 0b1000_0000 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
                self.sec(); 
                // C (carry flag) is set to 1 with bitwise OR, see below
            } else {
                self.clc();
                // C (carry flag) is set to 0  with bitwise AND, see below
            }
            self.register_a = value << 1;
            self.register_a = self.register_a | original_carry_flag; // 0 bit set to og carry flag
            // with bitwise OR

            self.update_zero_and_negative_flags(self.register_a);

        } else {
            let (addr, _) = self.get_operand_address(mode);
            let mut value = self.mem_read(addr);
            if value & 0b1000_0000 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
                self.sec(); 
                // C (carry flag) is set to 1 with bitwise OR, see below
            } else {
                self.clc();
                // C (carry flag) is set to 0  with bitwise AND, see below
            }
            value = value << 1;
            value = value | original_carry_flag; // 0 bit set to og carry flag with bitwise OR

            self.mem_write(addr, value);

            self.update_zero_and_negative_flags(value);
        }
    }
    

    fn lsr(&mut self, mode: &AddressingMode) {

        if mode == &AddressingMode::NoneAddressing {
            let value = self.register_a;
            if value & 0b0000_0001 != 0 { // if bit 0 of register is set, checked w/ bitwise AND
                self.sec(); 
                // C (carry flag) is set to 1 with bitwise OR, see below
            } else {
                self.clc();
                // C (carry flag) is set to 0  with bitwise AND, see below
            }
            self.register_a = value >> 1 ;
            self.update_zero_and_negative_flags(self.register_a);

        } else {
            let (addr, _) = self.get_operand_address(mode);
            let mut value = self.mem_read(addr);
            if value & 0b0000_0001 != 0 { // if bit 0 of register is set, checked w/ bitwise AND
                self.sec(); 
                // C (carry flag) is set to 1 with bitwise OR, see below
            } else {
                self.clc();
                // C (carry flag) is set to 0  with bitwise AND, see below
            }
            value = value >> 1;
            self.mem_write(addr, value);

            self.update_zero_and_negative_flags(value);
        }
    }

    fn ror(&mut self, mode: &AddressingMode) {

        let original_carry_flag = self.status & 0b0000_0001;

        if mode == &AddressingMode::NoneAddressing {

            let value = self.register_a;
            if value & 0b0000_0001 != 0 { // if bit 0 of register is set, checked w/ bitwise AND
                self.sec(); 
                // C (carry flag) is set to 1 with bitwise OR, see below
            } else {
                self.clc();
                // C (carry flag) is set to 0  with bitwise AND, see below
            }
            self.register_a = value >> 1 ;

            if original_carry_flag == 0b0000_0001 {
                self.register_a = self.register_a | 0b1000_0000; // last bit set to og carry flag 
                // with bitwise OR
            } else {
                self.register_a = self.register_a | 0b0000_0000
            }; 
            
            self.update_zero_and_negative_flags(self.register_a);

        } else {

            let (addr, _) = self.get_operand_address(mode);
            let mut value = self.mem_read(addr);
            if value & 0b0000_0001 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
                self.sec(); 
                // C (carry flag) is set to 1 with bitwise OR, see below
            } else {
                self.clc();
                // C (carry flag) is set to 0  with bitwise AND, see below
            }
            value = value >> 1;
            
            if original_carry_flag == 0b0000_0001 {
                value = value | 0b1000_0000; // last bit set to og carry flag 
                // with bitwise OR
            } else {
                value = value | 0b0000_0000
            }; 

            self.mem_write(addr, value);

            self.update_zero_and_negative_flags(value);
        }
    }

    fn sec(&mut self) {
        self.status = self.status | 0b0000_0001;
    }

    fn set_v(&mut self) {
        self.status = self.status | 0b0100_0000;
    }

    fn clc(&mut self) {
        self.status = self.status & 0b1111_1110;
    }

    fn sei(&mut self) {
        self.status = self.status | 0b0000_0100;
    }

    fn cli(&mut self) {
        self.status = self.status & 0b1111_1011;
    }

    fn sed(&mut self) {
        self.status = self.status | 0b0000_1000;
    }

    fn cld(&mut self) {
        self.status = self.status & 0b1111_0111;
    }

    fn clv(&mut self) {
        self.status = self.status & 0b1011_1111;
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if (self.register_a & value) == 0 { // if register = 0
            self.status = self.status | 0b0000_0010; 
            // Z (zero flag) set to 1 with bitwise OR
        } else {
            self.status = self.status & 0b1111_1101;
            //otherwise, bitwise AND keeps everything else the same 
            //and sets Z to 0.
        }

        if (value & 0b0100_0000) == 0b0100_0000 { // if 6th bit is set
            self.status = self.status | 0b0100_0000; // set V
        } else {
            self.status = self.status & 0b1011_1111;
            //otherwise, bitwise AND keeps everything else the same 
            //and unsets V
        }

        if (value & 0b1000_0000) == 0b1000_0000 { // if 6th bit is set
            self.status = self.status | 0b1000_0000; // set N
        } else {
            self.status = self.status & 0b0111_1111;
            //otherwise, bitwise AND keeps everything else the same 
            //and unsets N
        }

        self.status = self.status | (value & 0b1100_0000); // bracketed potion gets bit 7 and
        // 6 out of the value , which are then copied into N and V with bitwise OR.

    }

    fn bne(&mut self) {
        if self.status & 0b0000_0010 == 0b0000_0000 {
            self.bus.tick(1);

            let value = self.mem_read(self.program_counter);

            let shift = value as i8;
            let old_pc = self.program_counter;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }

            if page_cross(old_pc, self.program_counter) {
                self.bus.tick(1);
            }

        }
    }

    fn beq(&mut self) {
        if self.status & 0b0000_0010 == 0b0000_0010 {
            self.bus.tick(1);

            let value = self.mem_read(self.program_counter);

            let shift = value as i8;
            let old_pc = self.program_counter;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }

            if page_cross(old_pc, self.program_counter) {
                self.bus.tick(1);
            }
        }
    }

    fn bcc(&mut self) {
        if self.status & 0b0000_0001 == 0b0000_0000 {
            self.bus.tick(1);

            let value = self.mem_read(self.program_counter);

            let shift = value as i8;
            let old_pc = self.program_counter;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }

            if page_cross(old_pc, self.program_counter) {
                self.bus.tick(1);
            }
        }
    }

    fn bcs(&mut self) {
        if self.status & 0b0000_0001 == 0b0000_0001 {
            self.bus.tick(1);

            let value = self.mem_read(self.program_counter);

            let shift = value as i8;
            let old_pc = self.program_counter;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }

            if page_cross(old_pc, self.program_counter) {
                self.bus.tick(1);
            }
        }
    }

    fn bmi(&mut self) {
        if self.status & 0b1000_0000 == 0b1000_0000 {
            self.bus.tick(1);

            let value = self.mem_read(self.program_counter);

            let shift = value as i8;
            let old_pc = self.program_counter;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }

            if page_cross(old_pc, self.program_counter) {
                self.bus.tick(1);
            }
        }
    }

    fn bpl(&mut self) {
        if self.status & 0b1000_0000 == 0b0000_0000 {
            self.bus.tick(1);
            
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;
            let old_pc = self.program_counter;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }

            if page_cross(old_pc, self.program_counter) {
                self.bus.tick(1);
            }
        }
    }

    fn bvc(&mut self) {
        if self.status & 0b0100_0000 == 0b0000_0000 {
            self.bus.tick(1);
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;
            let old_pc = self.program_counter;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }

            if page_cross(old_pc, self.program_counter) {
                self.bus.tick(1);
            }
        }
    }

    fn bvs(&mut self) {
        if self.status & 0b0100_0000 == 0b0100_0000 {
            self.bus.tick(1);
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;
            let old_pc = self.program_counter;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }

            if page_cross(old_pc, self.program_counter) {
                self.bus.tick(1);
            }
        }
    }

    fn jmp(&mut self, mode: &AddressingMode) {

        if mode == &AddressingMode::NoneAddressing { // INDIRECT ADDRESSING
            let addr = self.mem_read_u16(self.program_counter);

                    //6502 bug mode with with page boundary:
                    //  if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
                    // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
                    // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000

                    let mut indirect_ref = self.mem_read_u16(addr);

                    if (addr & 0x00FF) == 0x00FF {
                        let lo = self.mem_read(addr);
                        let hi = self.mem_read(addr & 0xFF00);
                        indirect_ref = u16::from_le_bytes([lo,hi]);
                    }

                    self.program_counter = indirect_ref;
        }
        else {
            let (addr, _) = self.get_operand_address(mode);
            self.program_counter = addr;
        }
    }

    fn pha(&mut self) {
        let copy = self.register_a;
        let addr = 0x0100 + ((self.stack_pointer) as u16);

        self.mem_write(addr, copy);
        self.stack_pointer -= 1; // wrapping is not used here as rust will panic on overflow,
        // implicitly encoding it
    }

    fn php(&mut self) {
        let copy = self.status | 0b0001_0000; // set B flag for copy being pushed to stack
        
        let addr = 0x0100 + ((self.stack_pointer) as u16);

        self.mem_write(addr, copy);
        self.stack_pointer -= 1;
    }
    
    fn pla(&mut self) {
        self.stack_pointer += 1; // wrapping is not used here as rust will panic on underflow,
        // implicitly encoding it.

        // Added to SP before rest of the pull ensures correct indexing for memory address.
        let addr = 0x0100 + ((self.stack_pointer) as u16);
        self.register_a = self.mem_read(addr);
        
        // NOTE: NO NEED TO RESET VALUE TO 0x00 AT THAT POSITION.

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn plp(&mut self) {
        self.stack_pointer += 1;
        let addr = 0x0100 + ((self.stack_pointer) as u16);
        self.status = self.mem_read(addr);

        if (self.status & 0b0001_0000) == 0b0001_0000 { // if B flag is set in stack status
            self.status = self.status & 0b1110_1111 // unset it
        }

        self.status = self.status | 0b0010_0000; // set empty flag (always set to 1)
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        let mut stack_addr = 0x0100 + ((self.stack_pointer) as u16);
        
        let future_pc = self.program_counter + 1;

        let hi = (future_pc >> 8) as u8;
        let lo = (future_pc & 0xff) as u8;

        self.mem_write(stack_addr, hi);
        self.stack_pointer -= 1;
        stack_addr -= 1;

        self.mem_write(stack_addr, lo);
        self.stack_pointer -= 1;

        let (addr, _) = self.get_operand_address(mode);
        self.program_counter = addr;
    }

    fn rts(&mut self) { 
        self.stack_pointer += 1;
        let mut stack_addr = 0x0100 + ((self.stack_pointer) as u16);
        let lo = self.mem_read(stack_addr);

        self.stack_pointer +=1;
        stack_addr += 1;
        let hi = self.mem_read(stack_addr);

        self.program_counter = u16::from_le_bytes([lo,hi]) + 1;
    }

    fn rti(&mut self) {
        self.stack_pointer += 1;
        let mut stack_addr = 0x0100 + (self.stack_pointer as u16);
        self.status = self.mem_read(stack_addr);

        if (self.status & 0b0001_0000) == 0b0001_0000 { // if B flag is set in stack status
            self.status = self.status & 0b1110_1111 // unset it
        }

        self.status = self.status | 0b0010_0000; // set empty flag (always set to 1)
    
        self.stack_pointer += 1;
        stack_addr += 1;
        let lo = self.mem_read(stack_addr);
    
        self.stack_pointer += 1;
        stack_addr += 1;
        let hi = self.mem_read(stack_addr);

        self.program_counter = u16::from_le_bytes([lo,hi]);
    }

    fn plus_minus(&mut self, data: u8) {
        // based on the following resource and the tutorial github
        // https://www.righto.com/2012/12/the-6502-overflow-flag-explained.html

        let c6 = (self.status << 7) >> 7;
        let sum = self.register_a as u16
            + data as u16
            + c6 as u16;

        let carry = sum > 0xff;

        if carry {
            self.status = self.status | 0b0000_0001;
        } else {
            self.status = self.status & 0b1111_1110;
        }

        let result = sum as u8;

        if (data ^ result) & (result ^ self.register_a) & 0x80 != 0 {
            self.status = self.status | 0b0100_0000; // set overflow
        } else {
            self.status = self.status & 0b1011_1111; // unset overflow
        }

        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.plus_minus(value);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        value = !value; // 1's complement 
        self.plus_minus(value); // X - Y ==  X + -Y, and -Y == !Y  in signed complements.

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn dcp(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        value = value.wrapping_sub(1);
        self.mem_write(addr, value);

        if value <= self.register_a {
            self.sec();
        }

        self.update_zero_and_negative_flags(self.register_a.wrapping_sub(value));

    }

    fn rla(&mut self, mode: &AddressingMode) {
        self.rol(mode);
        self.and(mode);
    }

    fn slo(&mut self, mode: &AddressingMode) {
        self.asl(mode);
        self.ora(mode);
    }

    fn sre(&mut self, mode: &AddressingMode) {
        self.lsr(mode);
        self.eor(mode);
    }

    fn axs(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if value <=  (self.register_a & self.register_x){
            self.sec();
        }

        let result = (self.register_a & self.register_x).wrapping_sub(value);
        self.register_x = result;

        self.update_zero_and_negative_flags(self.register_x);
    }

    fn arr(&mut self, mode: &AddressingMode) {
        self.and(mode);
        self.ror(mode);

        let b5 = (self.register_a >> 5) & 1;
        let b6 = (self.register_a >> 6) & 1;

        if b5 == 1 && b6 == 1 {
            self.sec();
            self.clv();
        }
        else if b5 == 0 && b6 == 0 {
            self.clc();
            self.clv();
        }
        else if b5 == 1 && b6 == 0 {
            self.set_v();
            self.clc();
        }
        else if b5 == 0 && b6 == 1 {
            self.sec();
            self.set_v();
        }
    }

    fn anc(&mut self, mode: &AddressingMode) {
        self.and(mode);

        if self.status & 0b1000_0000 == 0b1000_0000 {
            self.sec();
        }
        else {
            self.clc();
        }
    }

    fn alr(&mut self, mode: &AddressingMode) {
        self.and(mode);
        self.lsr(mode);
    }

    fn rra(&mut self, mode: &AddressingMode) {
        self.ror(mode);
        self.adc(mode);
    }

    fn isb(&mut self, mode: &AddressingMode) {
        self.inc(mode);
        self.sbc(mode);
    }

    fn lax(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.ldx(mode);
    }

    fn sax(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.register_a & self.register_x;

        self.mem_write(addr, data);
    }

    fn interrupt(&mut self, interrupt: interrupt::Interrupt) {
        // self.stack_push_u16(self.program_counter);

        let hi = (self.program_counter >> 8) as u8;
        let lo = (self.program_counter & 0xff) as u8;
        let mut addr = 0x0100 + ((self.stack_pointer) as u16);

        self.mem_write(addr, hi);
        self.stack_pointer -= 1;

        addr = 0x0100 + ((self.stack_pointer) as u16);
        self.mem_write(addr, lo);
        self.stack_pointer -= 1;

        let mut flag = self.status.clone();

        flag = flag & 0b1110_1111; // unset B flag
        flag = flag | 0b0010_0000; // set Unused flag

        addr = 0x0100 + ((self.stack_pointer) as u16);

        self.mem_write(addr, flag);
        self.stack_pointer -= 1;

        self.status = self.status | 0b0000_0100; // set I (disable all additional Interrupts) flag

        self.bus.tick(interrupt.cpu_cycles);
        self.program_counter = self.mem_read_u16(interrupt.vector_addr);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {}); // This is a closure. A closure is like a function except it captures values
        // from its environment (that is, rust intelligently infers the type it will be operating with without needing to be told explicitly)
    }

    // The run_with_callback method is the actual workhorse function that drives the CPU simulation. 
    // This function accepts a callback parameter, F, which is a closure or function that takes a &mut CPU as an argument.

    // Callback Parameter (F: FnMut(&mut CPU))
    // The callback function F must implement the FnMut(&mut CPU) trait. This means:
    // - The callback function will accept a mutable reference to the CPU object.
    // - The callback can modify the CPU each time it’s called.
    // - The FnMut trait means that this function might change its state between calls, making it suitable for 
    // callbacks that track or modify something over time (e.g., tracking CPU cycles or responding to specific CPU states).

    // This is used for recursion. This also comes in handy when rendering the screen using the PPU 
    // and passing the callback to the Bus, which changes the CPU state.

    pub fn run_with_callback<F>(&mut self, mut callback: F) // F is a generic type... 
    where F: FnMut(&mut CPU), // such that F is a mutable closure which does not move captured values out of their body, 
    // but might mutate the captured values. These closures can be called more than once.

    // https://doc.rust-lang.org/book/ch13-01-closures.html
        
    {   
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;
        // create a reference opdcodes in the cpu of the Hashmap type from u8 to OpCode data, from OPCODES_MAP in 
        // opcode.rs. OPCODES_MAP is dereferenced as it is a ref, and to get values out of it (instead of pointers) we must
        // deref with *.

        loop {
            if let Some(_nmi) = self.bus.poll_nmi_status() {
                self.interrupt(interrupt::NMI);
            }

            callback(self); // Queue the inputs (orders) and execute them as and when possible...
            
            // ... while the current known inputs can be processed.
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = opcodes.get(&code).expect(&format!("OpCode {:x} is not recognized", code));
            // gets the value (opcode data) from a reference to the key (code), otherwise throws an exception.

            match code {
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    self.lda(&opcode.mode);
                }

                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => {
                    self.ldx(&opcode.mode);
                }

                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => {
                    self.ldy(&opcode.mode);
                }

                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }

                0x86 | 0x96 | 0x8e => {
                    self.stx(&opcode.mode);
                }

                0x84 | 0x94 | 0x8c => {
                    self.sty(&opcode.mode);
                }

                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => {
                    self.and(&opcode.mode);
                }

                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 /* unofficial -> */ | 0xeb => {
                    self.sbc(&opcode.mode);
                }

                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => {
                    self.adc(&opcode.mode);
                }

                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => {
                    self.ora(&opcode.mode);
                }

                0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => {
                    self.eor(&opcode.mode);
                }

                0x0a | 0x06 | 0x16 | 0x0e | 0x1e => {
                    self.asl(&opcode.mode);
                }

                0x2a | 0x26 | 0x36 | 0x2e | 0x3e => {
                    self.rol(&opcode.mode);
                }

                0x4a | 0x46 | 0x56 | 0x4e | 0x5e => {
                    self.lsr(&opcode.mode);
                }

                0x6a | 0x66 | 0x76 | 0x6e | 0x7e => {
                    self.ror(&opcode.mode);
                }

                0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => {
                    self.cmp(&opcode.mode);
                }

                0xe0 | 0xe4 | 0xec => {
                    self.cpx(&opcode.mode);
                }

                0xc0 | 0xc4 | 0xcc => {
                    self.cpy(&opcode.mode);
                }

                0x40 => self.rti(),

                0x20 => self.jsr(&opcode.mode),

                0x60 => self.rts(),

                0xd0 => self.bne(),

                0x90 => self.bcc(),

                0xb0 => self.bcs(),

                0xf0 => self.beq(),

                0x30 => self.bmi(),

                0x10 => self.bpl(),

                0x50 => self.bvc(),

                0x70 => self.bvs(),
                
                0xaa => self.tax(),

                0xa8 => self.tay(),

                0x8a => self.txa(),

                0x98 => self.tya(),

                0xba => self.tsx(),

                0x9a => self.txs(),

                0xe6 | 0xf6 | 0xee | 0xfe => {
                    self.inc(&opcode.mode);
                }

                0xc6 | 0xd6 | 0xce | 0xde => {
                    self.dec(&opcode.mode);
                }

                0x24 | 0x2c => {
                    self.bit(&opcode.mode);
                }

                0x4c | 0x6c => {
                    self.jmp(&opcode.mode);
                }

                0x48 => self.pha(),

                0x08 => self.php(),

                0x68 => self.pla(),

                0x28 => self.plp(),

                0xe8 => self.inx(),

                0xca => self.dex(),

                0xc8 => self.iny(),

                0x88 => self.dey(),

                0x38 => self.sec(),

                0x18 => self.clc(),

                0x78 => self.sei(),

                0x58 => self.cli(),

                0xf8 => self.sed(),

                0xd8 => self.cld(),

                0xb8 => self.clv(),
                
                0xea /* <- main*/ | 0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa 
                | 0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xb2 | 0xd2 | 0xf2=> {
                    // NOP basic and KIL
                },

                // Other NOPs which read memory
                0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 | 0x0c | 0x1c
                | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc | 0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 => {
                    let (addr, _) = self.get_operand_address(&opcode.mode);
                    #[allow(unused_variables)]
                    let data = self.mem_read(addr);
                }

                0xc7 | 0xd7 | 0xCF | 0xdF | 0xdb | 0xd3 | 0xc3 => {
                    self.dcp(&opcode.mode)
                },

                0x27 | 0x37 | 0x2F | 0x3F | 0x3b | 0x33 | 0x23 => {
                    self.rla(&opcode.mode)
                },

                0x07 | 0x17 | 0x0F | 0x1f | 0x1b | 0x03 | 0x13 =>  {
                    self.slo(&opcode.mode)
                }

                0x47 | 0x57 | 0x4F | 0x5f | 0x5b | 0x43 | 0x53 => {
                    self.sre(&opcode.mode)
                }

                0xcb => {
                    self.axs(&opcode.mode)
                }

                0x6b => {
                    self.arr(&opcode.mode);
                }

                0x0b | 0x2b => {
                    self.anc(&opcode.mode);
                }

                0x4b => {
                    self.alr(&opcode.mode);
                }

                0x67 | 0x77 | 0x6f | 0x7f | 0x7b | 0x63 | 0x73 => {
                    self.rra(&opcode.mode);
                }

                0xe7 | 0xf7 | 0xef | 0xff | 0xfb | 0xe3 | 0xf3 => {
                    self.isb(&opcode.mode);
                }

                0xa7 | 0xb7 | 0xaf | 0xbf | 0xa3 | 0xb3 => {
                    self.lax(&opcode.mode);
                }

                0x87 | 0x97 | 0x8f | 0x83 => {
                    self.sax(&opcode.mode);
                }

                0x00 => { // BRK
                    self.status = self.status | 0b0001_0000; // set B flag
                    return; 
                }

                _ => todo!(),
            }

            self.bus.tick(opcode.cycles);

            if program_counter_state == self.program_counter { 
                // [-] Why would this ever be false?
                // [A] Because of CPU and PPU cycles!
                self.program_counter += (opcode.len - 1) as u16;
                // Steps to increase program counter by = bytes processed by opcode - 1
                // -1, because first increase caused by opcode matching is already accounted for. 
            }
        }
    }


    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }
}