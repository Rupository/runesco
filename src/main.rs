pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod opcodes;
pub mod trace;

pub mod ppu;
pub mod render;

//use bus::Bus;
//use cpu::Mem;
//use cpu::CPU;
//use rand::Rng;
use cartridge::Rom;
use render::frame::Frame;
use render::palette;
//use trace::trace;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
//use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
//use sdl2::EventPump;
// use std::time::Duration;

#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
fn show_tile(chr_rom: &Vec<u8>, bank: usize, tile_n: usize) -> Frame {
    // bank: specifies which of the two 4KiB banks of tile data to fetch the data from. bank == 0 or 1
    // tile_n: tile number
    assert!(bank <= 1);

    let mut frame = Frame::new();
    let bank = (bank * 0x1000) as usize;
    // for bank 0, points to 0x0000 in chr_rom
    // for bank 1, points to 0x1000 in chr_rom

    let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

    // Each tile is represented by 16 bytes: 8 bytes for the low bit plane and 8 bytes for the high bit plane.
    // For a given tile_n, the function calculates the start and end of the tile data in chr_rom:

    // tile_n * 16 gives the byte offset for this tile in the bank.
    // bank + tile_n * 16 gives the actual starting position in chr_rom for this tile.
    // (bank + tile_n * 16 + 15) marks the end of the 16-byte tile.

    // tile is thus a 16-byte slice that represents one 8x8 pixel tile.

    for y in 0..=7 {
        // loops through 8 rows of the 8x8 byte tile
        let mut upper = tile[y]; // gets one of the bit planes for palette
        let mut lower = tile[y + 8]; // gets the second half of the bit planes

        for x in (0..=7).rev() {
            let value = (1 & upper) << 1 | (1 & lower); // combine the planes together:
                                                        // extracts the lowest bit from upper and lower to form a 2-bit value for each pixel.

            // draw the pixel according to this value by matching with palette
            let rgb = match value {
                0 => palette::SYSTEM_PALLETE[0x01],
                1 => palette::SYSTEM_PALLETE[0x23],
                2 => palette::SYSTEM_PALLETE[0x27],
                3 => palette::SYSTEM_PALLETE[0x30],
                _ => panic!("can't be"),
            };
            frame.set_pixel(x, y, rgb);

            // rshift both by 1 to process next bit in the byte chain for the next pixel
            upper = upper >> 1;
            lower = lower >> 1;
        }
    }
    frame
}

fn show_tile_bank(chr_rom: &Vec<u8>, bank: usize) ->Frame {
    assert!(bank <= 1);

    let mut frame = Frame::new();
    let mut tile_y = 0;
    let mut tile_x = 0;
    let bank = (bank * 0x1000) as usize;

    for tile_n in 0..255 {
        if tile_n != 0 && tile_n % 20 == 0 { 
            // every time 20 tiles are drawn in a row,
            tile_y += 10; // move to next row
            tile_x = 0;
        }
        let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[0x01],
                    1 => palette::SYSTEM_PALLETE[0x23],
                    2 => palette::SYSTEM_PALLETE[0x27],
                    3 => palette::SYSTEM_PALLETE[0x30],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(tile_x + x, tile_y + y, rgb)
            }
        }

        tile_x += 10; // move to next tile in that row
    }
    frame
}

fn main() {
    // init sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "runesco: Tileset",
            (256.0 * 3.0) as u32,
            (240.0 * 3.0) as u32,
        )
        // 32x32 screen, scaled by a factor of 20.
        .position_centered()
        .build()
        .unwrap();

    // A 'canvas': something which can be 'drawn' on is put over the window
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    #[allow(unused_variables, unused_mut)]
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();

    // "Using .unwrap() is justifiable here because it's the outer layer of our application.
    // There are no other layers that potentially can handle Err values and do something about it."

    // The canvas is given a 'texture': which handles visuals.
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();
    // We specify that the visuals are in the form of 256 x 240 pixel grid

    //load the game
    let nes_file_data: Vec<u8> = std::fs::read("pacman.nes").unwrap();
    let rom = Rom::new(&nes_file_data).unwrap();

    let rom_len = rom.chr_rom.len();
    println!("Rom length is: {rom_len}");

    let bank = show_tile_bank(&rom.chr_rom, 1);


    texture.update(None, &bank.data, 256 * 3).unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                _ => { /* do nothing */ }
            }
        }
    }

    //let bus = Bus::new(rom);
    //let mut cpu = CPU::new(bus);
    //cpu.reset();
    //cpu.program_counter = 0xC000;

    // let mut screen_state = [0 as u8; 32 * 3 * 32]; // initialise the screen state array
    // let mut rng = rand::thread_rng();

    // run the game cycle
    //cpu.run_with_callback(move |cpu| {
    //println!("{}", trace(cpu));
    // CPU is moved (explicitly borrowed) so that nothing outside the gameloop
    // can change the CPU state.

    // handle_user_input(cpu, &mut event_pump);
    // cpu.mem_write(0xfe, rng.gen_range(1, 16));

    //if read_screen_state(cpu, &mut screen_state) { // update the screen if it needs to be updated
    //texture.update(None, &screen_state, 32 * 3).unwrap();
    //canvas.copy(&texture, None, None).unwrap();
    //canvas.present();
    //}

    //::std::thread::sleep(Duration::new(0, 10_000)); // slows down pace for playability
    //});
}
