use std::collections::HashMap;
use crate::opcodes;


pub struct CPU { // CPU with..  
    pub register_a: u8, // Accumulator A
    pub register_x: u8, // Register X
    pub register_y: u8, // Register Y
    pub stack_pointer: u8, // Stack Pointer
    pub status: u8, // Status flags [NV_BDIZC]
    pub program_counter: u16, // Program Counter
    memory: [u8; 0xFFFF] // ...and 64 Kilobits of total memory space
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

trait Mem {
    fn mem_read(&self, addr: u16) -> u8; 

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

impl Mem for CPU {
    
    fn mem_read(&self, addr: u16) -> u8 { // returns next 8 bit integer instruction
        self.memory[addr as usize] // from a 16 bit address, and converts to usize (compatibility)
    }

    fn mem_write(&mut self, addr: u16, data: u8) { // writes data to an address in memory
        self.memory[addr as usize] = data; 
    }
}

impl CPU {
    
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stack_pointer: 0xff,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }

    pub fn reset(&mut self) { // resets when new cartridge is loaded
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.stack_pointer = 0xff;
 
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        // Memory will be written (by slicing) from address 0x8000 to 0xXXXX, depending on program
        self.mem_write_u16(0xFFFC, 0x8000); // program counter, stored in 0xFFFC 
        // is set to 0x8000
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {

        match mode {
            AddressingMode::Immediate => self.program_counter, // Not really an addressing mode:
            // gives whatever hex value is in the instruction as the value to be used.
 
            AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
            // Gets u8 address from program counter, of which only 
            // the last two bits of converted the u16 will be relevant.
            // Only access first 256 bytes of memory
    
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            // full u16 address is read, can access 0-65536 bytes.
         
            AddressingMode::ZeroPage_X => { 
                // Takes 0-page address and adds the value stored
                // in the X register to it. Wraps around if $ff, X (X>0)
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                // See 0-page X
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
 
            AddressingMode::Absolute_X => {
                // Takes absolute address and adds the value stored
                // in the X register to it. Wraps around if $ff, X (X>0)
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                // See absolute X
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
 
            AddressingMode::Indirect_X => {
                // Gets a 0-page memory address
                let base = self.mem_read(self.program_counter);
 
                let ptr: u8 = (base as u8).wrapping_add(self.register_x); // adds what's in X to it
                let lo = self.mem_read(ptr as u16); // reads what's at the pointer
                let hi = self.mem_read(ptr.wrapping_add(1) as u16); // and then at pointer + 1
                u16::from_le_bytes([lo,hi]) // converts to full memory address $hilo
                
            }
            AddressingMode::Indirect_Y => {
                // Gets a 0-page memory address
                let base = self.mem_read(self.program_counter);
 
                let lo = self.mem_read(base as u16); // reads what's at pointer 
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16); // reads whats at pointer + 1
                let deref_base = u16::from_le_bytes([lo,hi]); // combines into full address, dereferncing base
                let deref = deref_base.wrapping_add(self.register_y as u16); // adds whats's in Y to deref-ed address.
                deref
            }
          
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
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
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
       
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
       
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
       
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
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
        let addr = self.get_operand_address(mode);
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
        let addr = self.get_operand_address(mode);
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
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a & value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a | value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a ^ value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let check = (self.register_a as i8) - (value as i8);

        if check >= 0 { // conversion to i8 necessary for comparison
            self.sec();
        }
        self.update_zero_and_negative_flags(check as u8);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let check = (self.register_x as i8) - (value as i8);

        if check >= 0 { // conversion to i8 necessary for comparison
            self.sec();
        }
        self.update_zero_and_negative_flags(check as u8);
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let check = (self.register_y as i8) - (value as i8);

        if check >= 0 { // conversion to i8 necessary for comparison
            self.sec();
        }
        self.update_zero_and_negative_flags(check as u8);
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
            let addr = self.get_operand_address(mode);
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
            let addr = self.get_operand_address(mode);
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
            let addr = self.get_operand_address(mode);
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

            let addr = self.get_operand_address(mode);
            let mut value = self.mem_read(addr);
            if value & 0b1000_0000 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
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

    fn clc(&mut self) {
        self.status = self.status & 0b1111_1110;
    }

    fn sei(&mut self) {
        self.status = self.status | 0b0000_0100;
    }

    fn cli(&mut self) {
        self.status = self.status & 0b1111_1011;
    }

    fn clv(&mut self) {
        self.status = self.status & 0b1011_1111;
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if (self.register_a & value) == 0 { // if register = 0
            self.status = self.status | 0b0000_0010; 
            // Z (zero flag) set to 1 with bitwise OR
        } else {
            self.status = self.status & 0b1111_1101;
            //otherwise, bitwise AND keeps everything else the same 
            //and sets Z to 0.
        }

        self.status = self.status | (value & 0b1100_0000); // bracketed potion gets bit 7 and
        // 6 out of the value , which are then copied into N and V with bitwise OR.

    }

    fn bne(&mut self) {
        if self.status & 0b0000_0010 == 0b0000_0000 {
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }
        }
    }

    fn beq(&mut self) {
        if self.status & 0b0000_0010 == 0b0000_0010 {
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }
        }
    }

    fn bcc(&mut self) {
        if self.status & 0b0000_0001 == 0b0000_0000 {
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }
        }
    }

    fn bcs(&mut self) {
        if self.status & 0b0000_0001 == 0b0000_0001 {
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }
        }
    }

    fn bmi(&mut self) {
        if self.status & 0b1000_0000 == 0b1000_0000 {
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }
        }
    }

    fn bpl(&mut self) {
        if self.status & 0b1000_0000 == 0b0000_0000 {
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }
        }
    }

    fn bvc(&mut self) {
        if self.status & 0b0100_0000 == 0b0000_0000 {
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }
        }
    }

    fn bvs(&mut self) {
        if self.status & 0b0100_0000 == 0b0100_0000 {
            let value = self.mem_read(self.program_counter);

            let shift = value as i8;

            if shift >= 0 {
                self.program_counter = self.program_counter + 1 + (shift as u16);
                // increment the counter to put it at appropriate postion, and then shift ahead.
            } else {
                self.program_counter = self.program_counter - (0xffff - shift as u16);
                // shift back, and the way this is implemented by the datatypes (because of 2's complement),
                // counter gets automatically shifted correctly.
            }
        }
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
    }

    fn pha(&mut self) {
        let copy = self.register_a;
        let addr = 0x0100 + ((0xff - self.stack_pointer) as u16);

        self.mem_write(addr, copy);
        self.stack_pointer -= 1; // wrapping is not used here as rust will panic on overflow,
        // implicitly encoding it
    }

    fn php(&mut self) {
        self.status = self.status | 0b0001_0000; // set B flag
        
        let copy = self.status;
        let addr = 0x0100 + ((0xff - self.stack_pointer) as u16);

        self.mem_write(addr, copy);
        self.stack_pointer -= 1;
    }
    
    fn pla(&mut self) {
        self.stack_pointer += 1; // wrapping is not used here as rust will panic on underflow,
        // implicitly encoding it.

        // Added to SP before rest of the pull ensures correct indexing for memory address.
        let addr = 0x0100 + ((0xff - self.stack_pointer) as u16);
        self.register_a = self.mem_read(addr);

        self.mem_write(addr, 0x00);
        
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn plp(&mut self) {
        self.stack_pointer += 1;
        let addr = 0x0100 + ((0xff - self.stack_pointer) as u16);
        self.status = self.mem_read(addr);

        self.mem_write(addr, 0x00);
    }

    pub fn run(&mut self) {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;
        // create a reference opdcodes in the cpu of the Hashmap type from u8 to OpCode data, from OPCODES_MAP in 
        // opcode.rs. OPCODES_MAP is dereferenced as it is a ref, and to get values out of it (instead of pointers) we must
        // deref with *.

        loop {
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

                0xb8 => self.clv(),

                //0x90 => self.bcc(),

                0xea => {} , // NOP

                0x00 => { // BRK
                    self.status = self.status | 0b0001_0000; // set B flag
                    return; 
                }

                _ => todo!(),
            }

            if program_counter_state == self.program_counter { // [?] Why would this ever be false?
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

#[cfg(test)]
mod test {
   use super::*;

   #[test]
   #[should_panic]
   fn panic_overflow_stack_pha() {
    let mut cpu = CPU::new();
    cpu.stack_pointer = 0x00;
    cpu.load(vec![0xa9, 0x80, 0x48, 0x00]); // Didn't use normal load_and_run
    // as that would reset the stack pointer.
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
   }

   #[test]
   #[should_panic]
   fn panic_underflow_stack_pla() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x80, 0x68, 0x00]);
   }

   #[test]
   fn test_0x48_0x08_0x68_0x28_stack_ops_no_overflow_no_underflow() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x7f, 0x48, 0x38, 0x08, 0x18, 0xa9, 0x20, 0x28, 0x68, 0x00]);
    assert_eq!(cpu.stack_pointer, 0xff);
    assert_eq!(cpu.register_a, 0x7f);
    assert_eq!(cpu.mem_read(0x0100), 0x00);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0001);
    assert_eq!(cpu.mem_read(0x0101), 0x00);
   }

  #[test]
   fn test_0x48_0x08_stack_pushes_no_overflow_no_underflow() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x7f, 0x48, 0x38, 0x08, 0x18, 0xa9, 0x20, 0x00]);
    assert_eq!(cpu.stack_pointer, 0xfd);
    assert_ne!(cpu.register_a, 0x7f);
    assert_eq!(cpu.register_a, 0x20);
    assert_eq!(cpu.mem_read(0x0100), 0x7f);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0000);
    assert!(cpu.mem_read(0x0101) & 0b0000_0001 == 0b0000_0001);
   }

   #[test]
   fn test_0x4c_jmp_jump_absolute() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x03, 0x4c, 0x07, 0x80, 0xa9, 0x30, 0x8d, 0x00, 0x02]);
    assert_eq!(cpu.program_counter, 0x800b); // stops at 11 instead of 10 because there is no break,
    // and an unencoded break occurs when it reads from memory outside of the program vector,
    // which are all 0x0000 = 0x00, leading us to a break.
    assert_eq!(cpu.mem_read_u16(0x0200), 0x03); 
   }

   #[test]
   fn test_0xd0_bne_branch_not_equal_positive_shift() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x01, 0xc9, 0x02, 0xd0, 0x02, 0x85, 0x22, 0x00]);
    assert_eq!(cpu.mem_read(0x22), 0);
    assert_eq!(cpu.program_counter, 0x8009);
   }

   #[test]
   fn test_0xd0_bne_branch_not_equal_negative_shift() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa2, 0x08, 0xca, 0x8e, 0x00, 0x02, 0xe0, 0x03, 0xd0, 0xf8, 0x8e, 0x01, 0x02, 0x00]);
    assert_eq!(cpu.register_x, 0x03);
    assert_eq!(cpu.mem_read(0x0201), 0x03);
    assert_eq!(cpu.program_counter, 0x800e);
   }

   #[test]
   fn test_0x24_bit_test() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x20, 0xc0);
    cpu.load_and_run(vec![0xa9, 0x3f, 0x24, 0x20, 0x00]);
    assert!(cpu.status & 0b0000_0010 == 0b0000_0010); // Z
    assert!(cpu.status & 0b1000_0000 == 0b1000_0000); // N
    assert!(cpu.status & 0b0100_0000 == 0b0100_0000); // V
   }

   #[test]
   fn test_0x6a_ror_accumulator_rotate_right() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x38, 0xa9, 0x02, 0x6a, 0x00]);
    assert_eq!(cpu.register_a, 0x81);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0000);
   }

   #[test]
   fn test_0x66_ror_from_memory_rotate_right() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x20, 0x02);
    cpu.load_and_run(vec![0x38, 0x66, 0x20, 0x00]);
    assert_eq!(cpu.mem_read(0x20), 0x81);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0000);
   }

   #[test]
   fn test_0x2a_rol_accumulator_rotate_left() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x38, 0xa9, 0x7f, 0x2a, 0x00]);
    assert_eq!(cpu.register_a, 0xff);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0000);
   }

   #[test]
   fn test_0x26_rol_from_memory_rotate_left() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x20, 0x7f);
    cpu.load_and_run(vec![0x38, 0x26, 0x20, 0x00]);
    assert_eq!(cpu.mem_read(0x20), 0xff);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0000);
   }

   #[test]
   fn test_e6_c6_inc_dec_zero_page() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x20, 0x80);
    cpu.mem_write(0x21, 0x80);
    cpu.load_and_run(vec![0xe6, 0x20, 0xc6, 0x21, 0x00]);
    assert_eq!(cpu.mem_read(0x20), 0x81);
    assert_eq!(cpu.mem_read(0x21), 0x7f);
   }

   #[test]
   fn test_0x49_eor_immediate_xor() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x80, 0x49, 0xff, 0x00]);
    assert_eq!(cpu.register_a, 0b0111_1111);
   }

   #[test]
   fn test_0x09_ora_immediate_logical_or_inclusive() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x80, 0x09, 0x7f, 0x00]);
    assert_eq!(cpu.register_a, 0b1111_1111);
   }

   #[test]
   fn test_0x4a_lsr_accumulator_left_shift() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x81, 0x4a, 0x00]);
    assert_eq!(cpu.register_a, 0x40);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0001);
   }

   #[test]
   fn test_0x46_lsr_from_memory_left_shift() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x20, 0x81);
    cpu.load_and_run(vec![0x46, 0x20, 0x00]);
    assert_eq!(cpu.mem_read(0x20), 0x40);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0001);
   }


   #[test]
   fn test_0xeo_cpx_comparison_x_immediate() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa2, 0x80, 0xe0, 0x80, 0x00]);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0001); // C is set
    assert!(cpu.status & 0b0000_0010 == 0b0000_0010); // Z is set
    assert!(cpu.status & 0b1000_0000 == 0b0000_0000); // N is unset
   }

   #[test]
   fn test_0xc9_cmp_comparison_immediate() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x80, 0xc9, 0x80, 0x00]);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0001); // C is set
    assert!(cpu.status & 0b0000_0010 == 0b0000_0010); // Z is set
    assert!(cpu.status & 0b1000_0000 == 0b0000_0000); // N is unset
   }

   #[test]
   fn test_0xc9_cmp_comparison_from_memory() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x80, 0x85, 0x20, 0xc5, 0x20, 0x00]);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0001); // C is set
    assert!(cpu.status & 0b0000_0010 == 0b0000_0010); // Z is set
    assert!(cpu.status & 0b1000_0000 == 0b0000_0000); // N is unset
   }

   #[test]
   fn test_tsx_transfer_stack_pointer_value_to_x() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xba,  0x00]);
    assert_eq!(cpu.register_x , 0xff);
   }

   #[test]
   fn test_non_overflow_sets_and_clears() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x38, 0x18, 0x78, 0x58, 0x00]);
    assert!(cpu.status & 0b0000_0001 == 0);
    assert!(cpu.status & 0b1000_0100 == 0);
   }

   #[test]
   fn test_0x0a_asl_accumulator_left_shift() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x80, 0x0a, 0x00]);
    assert_eq!(cpu.register_a, 0);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0001);
   }

   #[test]
   fn test_0x06_asl_from_memory_left_shift() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x20, 0x70);
    cpu.load_and_run(vec![0x06, 0x20, 0x00]);
    assert_eq!(cpu.mem_read(0x20), 0xe0);
    assert!(cpu.status & 0b0000_0001 == 0b0000_0000);
   }

   #[test]
   fn test_0x29_and_immediate_logical_and_bitwise() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x80, 0x29, 0x01, 0x00]);
    assert_eq!(cpu.register_a, 0b0000_0000);
   }

   #[test]
   fn test_0x2d_and_absolute_logical_and_bitwise() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x20, 0x80);
    cpu.load_and_run(vec![0xa9, 0x80, 0x2d, 0x01, 0x00, 0x00]);
    assert_eq!(cpu.register_a, 0b0000_0000);
   }
 
   #[test]
   fn test_0xa9_lda_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
       assert_eq!(cpu.register_a, 0x05);
       assert!(cpu.status & 0b0000_0010 == 0b00); // since A =/= 0, tests whether Z flag is set or not 
       // (should be unset)
       assert!(cpu.status & 0b1000_0000 == 0); // since 7th bit of A is not set, 
       // tests whther N flag is set or not (should be unset)
   }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10); // since A = 0, tests whether Z flag 
        // is set or not (should be set)
    }

    #[test]
    fn test_0xa9_lda_neg_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000); // since 7th bit of A is set, 
        // tests whther N flag is set or not (should be set)
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
        assert_eq!(cpu.register_x, cpu.register_a)
    }

    #[test]
    fn test_0xaa_txa_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10); // since A = 0, and then X = 0,
        // tests whether Z flag is set or not (should be set)
    }

    #[test]
    fn test_0xaa_txa_neg_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0xaa, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000); // since A has 7th bit set, and then so does X,
        // tests whether N flag is set or not (should be set)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 1)
    }

    #[test]
   fn test_5_ops_working_together() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
 
       assert_eq!(cpu.register_x, 0xc1)
   }

   #[test]
   fn test_lda_from_memory() {
       let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0x55);

       cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

       assert_eq!(cpu.register_a, 0x55);
   }
}