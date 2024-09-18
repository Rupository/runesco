use std::collections::HashMap;
use crate::opcodes;


pub struct CPU { // CPU with..  
    pub register_a: u8, // Accumulator A
    pub register_x: u8, // Register X
    pub register_y: u8, // Register Y
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
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }

    pub fn reset(&mut self) { // resets when new cartridge is loaded
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;
 
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

    fn inx(&mut self) {
        if self.register_x == 0xff { 
            self.register_x = 0;
        } else {
            self.register_x += 1;
        }
        self.update_zero_and_negative_flags(self.register_x);
        // note: Carry is NOT USED! Addition here is in modulo 0xff, loops back to 0.
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

    fn asl(&mut self, mode: &AddressingMode) {

        if mode == &AddressingMode::NoneAddressing {
            let value = self.register_a;
            if value & 0b1000_0000 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
                self.status = self.status | 0b0000_0001; 
                // C (carry flag) is set to 1 with bitwise OR
            } else {
                self.status = self.status & 0b1111_1110;
                // C (carry flag) is set to 0  with bitwise AND
            }
            self.register_a = value << 1 ;
            self.update_zero_and_negative_flags(self.register_a);

        } else {
            let addr = self.get_operand_address(mode);
            let mut value = self.mem_read(addr);
            if value & 0b1000_0000 != 0 { // if 7th (last) bit of register is set, checked w/ bitwise AND
                self.status = self.status | 0b0000_0001; 
                // C (carry flag) is set to 1 with bitwise OR
            } else {
                self.status = self.status & 0b1111_1110;
                // C (carry flag) is set to 0  with bitwise AND
            }
            value = value << 1;
            self.mem_write(addr, value);

            self.update_zero_and_negative_flags(value);
        }
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

                0x0a | 0x06 | 0x16 | 0x0e | 0x1e => {
                    self.asl(&opcode.mode);
                }
                
                0xaa => self.tax(),

                0xa8 => self.tay(),

                0x8a => self.txa(),

                0x98 => self.tya(),

                0xe8 => self.inx(),

                0xc8 => self.iny(),

                0x00 => return,
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