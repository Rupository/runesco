use crate::cpu::Mem;
use crate::cartridge::Rom;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const PRG: u16 = 0x8000;
const PRG_END: u16 = 0xFFFF;

pub struct Bus {
    cpu_vram: [u8; 2048], // 2KiB of Ram, from 0x0000 to 0x2000 (with higest two bits 0-ed)
    rom: Rom,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            cpu_vram: [0; 2048],
            rom: rom,
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr = addr - 0x8000; // gets the position of the "cursor" 
        // (how far the position is from the start of the prg rom location)
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // if length is 16KiB, and cursor has gone beyond this length,
            // mirror it.
            addr = addr % 0x4000; // by resetting the cursor
        }
        self.rom.prg_rom[addr as usize] // get that position from the prg rom
    }
}

impl Mem for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b0000_0111_1111_1111; 
                // drops the two most significant bits due to wiring losses
                // as in accounted for in original hardware
                self.cpu_vram[mirror_down_addr as usize]
                // link the mirrored down address with the CPU's vram
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b0010_0000_0000_0111;
                // similar dropping of bits for PPU due to mirroring. There are only
                // 8 bytes needed, and rest is mirrored. [?]
                todo!("PPU is not supported yet")
            }
            PRG..=PRG_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b0010_0000_0000_0111;
                todo!("PPU is not supported yet");
            }
            PRG..=PRG_END => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}