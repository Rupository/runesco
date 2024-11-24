use crate::cpu::Mem;
use crate::cartridge::Rom;
use crate::ppu::NesPPU;
use crate::joypads::Joypad;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
//const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const PRG: u16 = 0x8000;
const PRG_END: u16 = 0xFFFF;

pub struct Bus<'call> {
    // <'call> is a lifetime parameter for the Bus struct. It indicates that some part of the Bus struct 
    // (specifically the gameloop_callback field) contains a reference 
    // (or borrowed data) that must live as long as 'call.

    cpu_vram: [u8; 2048], // 2KiB of Ram, from 0x0000 to 0x2000 (with higest two bits 0-ed)
    prg_rom: Vec<u8>,
    ppu: NesPPU,
    cycles: usize,

    gameloop_callback: Box<dyn FnMut(&NesPPU, &mut Joypad, &mut Joypad) + 'call>,

    // Boxes: allow for data storage to the heap. Helpful when size is unknown (like in recursion!)
    // See: https://doc.rust-lang.org/book/ch15-01-box.html

    // dyn: By default, Rust uses static dispatch, which means that when you call a method on a type, 
    // the exact method implementation is determined at compile time. However, sometimes you want to call 
    // methods on types that might change at runtime, such as function traits (Fn, FnMut, etc.). This is
    // the purpose dyn serves: dynamic dispatch.

    // The + 'call part after FnMut(&NesPPU) specifies that the data required by this function 
    // (or any references it uses) will live as long as 'call, tying it to the 'call lifetime parameter.

    // Why Box<dyn FnMut(...)> instead of a plain function pointer?
    // Using dyn FnMut (a trait object) allows us to pass any closure or function that matches the 
    // signature FnMut(&NesPPU) without knowing its exact type. 
    // 
    // The Box makes it a heap-allocated, fixed-size pointer, which is necessary because dyn trait 
    // objects don’t have a known size at compile time, but pointers do!

    joypad1: Joypad,
    joypad2: Joypad,
}

impl<'a> Bus<'a> { // can be any lifetime 'a
    pub fn new<'call, F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
    where F: FnMut(&NesPPU, &mut Joypad, &mut Joypad) + 'call,
    {
        let ppu = NesPPU::new(rom.chr_rom, rom.screen_mirroring);

        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu: ppu,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
            joypad1 : Joypad::new(),
            joypad2 : Joypad::new(),
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles *3);
        let nmi_after = self.ppu.nmi_interrupt.is_some();
        
        if !nmi_before && nmi_after {
            (self.gameloop_callback)(&self.ppu, &mut self.joypad1, &mut self.joypad2);
        }

        // If an NMI has just been triggered (i.e., the NMI flag was false before and is true now), the function calls gameloop_callback
        // to render the next frame.
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.ppu.nmi_interrupt.take()
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr = addr - 0x8000; // gets the position of the "cursor" 
        // (how far the position is from the start of the prg rom location)
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // if length is 16KiB, and cursor has gone beyond this length,
            // mirror it.
            addr = addr % 0x4000; // by resetting the cursor
        }
        self.prg_rom[addr as usize] // get that position from the prg rom
    }
}

impl Mem for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b0000_0111_1111_1111; 
                // drops the two most significant bits due to wiring losses
                // as in accounted for in original hardware
                self.cpu_vram[mirror_down_addr as usize]
                // link the mirrored down address with the CPU's vram
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),

            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }

            0x4000..=0x4015 => {
                //ignore APU 
                0
            }

            0x4016 => {
                self.joypad1.read()
                
            }

            0x4017 => {
                self.joypad2.read()
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
            0x2000 => {
                self.ppu.write_to_ctrl(data);
            }
            0x2001 => {
                self.ppu.write_to_mask(data);
            }
            0x2002 => panic!("attempt to write to PPU status register"),

            0x2003 => {
                self.ppu.write_to_oam_addr(data);
            }

            0x2004 => {
                println!("Writing to OAM data");
                self.ppu.write_to_oam_data(data);
            }
            0x2005 => {
                self.ppu.write_to_scroll(data);
            }

            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);
            }

            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }

            0x4000..=0x4013 | 0x4015 => {
                //ignore APU 
            }

            0x4014 => { 
                // OAM sprite write operations happen
                // in a single go using this!

                // Transfers 256 bytes from CPU memory at once.
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                // data is 0x02, then hi = data << 8 <u16> gives 0x0200 as the starting address.
                // If data is 0x03, then the starting address is 0x0300, and so on.

                // This tells us the starting index of the page to be read to draw sprites

                // This formatting is necessary for the reading the subsequent 256
                // bytes of data directly, insted of passing them one by one (0x0201, 0x0202, ... )
                for i in 0..256u16 {
                    buffer[i as usize] = self.mem_read(hi + i);
                }

                // NOTE (^):
                // The memory page selected for the 0x4014 DMA transfer will contain the intended sprite 
                // data because the game or program running on the NES is responsible for placing sprite 
                //data in that specific page before triggering the transfer.

                // Init: Before calling the 0x4014 DMA write, the game code will typically load the 256 bytes of 
                // sprite data (e.g., positions, tile indices, colors) into a designated memory page in the 
                // CPU’s RAM (see: const RAM above). This data layout follows the PPU’s OAM format so that the
                // sprite attributes are ready to be copied as-is to the PPU.

                // Maintanance: Games usually follow a predictable layout for memory, especially for sprites. 
                // For instance, many games will designate a specific page, such as 0x0200 to 0x02FF, 
                // exclusively for sprite data, and the game engine will write sprite attributes to this page 
                // each frame as they update.

                // So, this read operation makes sense.

                self.ppu.write_oam_dma(&buffer);
            }

            0x4016 => {
                self.joypad1.write(data);
                self.joypad2.write(data);
            }

            0x4017 => {

            }

            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}