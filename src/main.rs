#![allow(unused_variables)]
#![allow(dead_code)]

pub struct CPU { // CPU with Accumulator A, Register X, Register Y, Status flags [NV_BDIZC], and Program Counter
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF] // ...and 64 Kilobits of total memory space
}

#[derive(Debug)] //
#[allow(non_camel_case_types)]
pub enum AddressingMode {
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

    fn mem_read(&self, addr: u16) -> u8 { // returns next 8 bit integer instruction
        self.memory[addr as usize] // from a 16 bit address, and converts to usize (compatibility)
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    // "Technically, mem_read_u16 doesn't need &mut self and can use immutable reference.
    // But, mem_read requests could be routed to external "devices" (such as PPU and Joypads), 
    // where reading does indeed modify the state of the simulated device."

    // [?] However, doing this makes the Addressing Mode implementation (later) break as it uses an immutable reference.
    // How does one ensure this capacity for potential modification while also maintining immutability?
    // Either remove mutability here or add mutable reference to addressing modes.
    
    fn mem_write(&mut self, addr: u16, data: u8) { // writes data to an address in memory
        self.memory[addr as usize] = data; 
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) { // write little endian, u16 data written as u8 + u8
        let hi = (data >> 8) as u8; 
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
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
            AddressingMode::Immediate => self.program_counter,
 
            AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
           
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
         
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
 
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
 
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
 
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
 
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }
          
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    // [A] The book later updates the above to have a mutable reference.

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

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }
  
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        if self.register_x == 0xff { 
            self.register_x = 0;
        } else {
            self.register_x += 1;
        }
        self.update_zero_and_negative_flags(self.register_x);
        // note: Carry is NOT USED! Addition is in modulo 0xff, loops back to 0.
    }
    
    pub fn run(&mut self) {
        
        loop {
            let opscode = self.mem_read(self.program_counter); 
            self.program_counter +=1; // reading the opcode takes 1 byte

            match opscode {

                0xA9 => { // LDA
                    let param = self.memory[self.program_counter as usize];
                    self.program_counter +=1; // using the parameter takes 1 byte
                    self.lda(param)
    
                }

                0xAA =>  { // TAX
                    self.tax() 
                }

                0x00 => { // BRK
                    return; 
                }

                0xE8 => { // INX
                    self.inx()
                }


                _ => todo!()
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
   fn test_0xa9_lda_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
       assert_eq!(cpu.register_a, 0x05);
       assert!(cpu.status & 0b0000_0010 == 0b00); // since A =/= 0, tests whether Z flag is set or not 
       // (should be unset)
       assert!(cpu.status & 0b1000_0000 == 0); // since 7th bit of A is not set, 
       // tests whther N flag is set or not (should be unset)

       // [?] Why is 0b00 being used in one case and 0 in the other? Aren't these equivalent?
       // [A] Probably just a choice to highlight the bit not being changed.
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
}

fn main () {
    println!("ruNESco: Rust NES Co-op");
}