#![allow(unused_variables)]
#![allow(dead_code)]

pub struct CPU { // CPU with Accumulator A, Status flags [NV_BDIZC], and Program Counter
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
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
   
    
    pub fn interpret(&mut self, program: Vec<u8>) { // Reads instructions given in machine code: 
        // Eg. interpret([a9, c0, aa, e8, 00]) 
        self.program_counter = 0;

        loop {
            let opscode = program[self.program_counter as usize]; // usize as it decides
            // based on native architecture, allowing compatibility 
            self.program_counter +=1; // reading the opcode takes 1 byte

            match opscode {

                0xA9 => { // LDA
                    let param = program[self.program_counter as usize];
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
}

#[cfg(test)]
mod test {
   use super::*;
 
   #[test]
   fn test_0xa9_lda_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.interpret(vec![0xa9, 0x05, 0x00]);
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
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10); // since A = 0, tests whether Z flag 
        // is set or not (should be set)
    }

    #[test]
    fn test_0xa9_lda_neg_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x80, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000); // since 7th bit of A is set, 
        // tests whther N flag is set or not (should be set)
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.interpret(vec![0xaa, 0x00]);
        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_0xaa_txa_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10); // since A = 0, and then X = 0,
        // tests whether Z flag is set or not (should be set)
    }

    #[test]
    fn test_0xaa_txa_neg_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x80, 0xaa, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000); // since A has 7th bit set, and then so does X,
        // tests whether N flag is set or not (should be set)
    }

    #[test]
   fn test_5_ops_working_together() {
       let mut cpu = CPU::new();
       cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
 
       assert_eq!(cpu.register_x, 0xc1)
   }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
}

fn main () {
    println!("ruNESco: Rust NES Co-op");
}