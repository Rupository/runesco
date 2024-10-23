pub mod cpu;
pub mod opcodes;
pub mod bus;

use cpu::Mem;
use cpu::CPU;
use rand::Rng;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use std::time::Duration;

#[macro_use]
extern crate lazy_static;

fn read_screen_state(cpu: &CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool { 
    // Takes the state of the CPU and the screen, represented as a 32*3*32 sized array.
    // Returns whether it has been/needs to be updated or not.

   let mut frame_idx = 0;
   let mut update = false;
   for i in 0x0200..0x600 { 
    // for each value in the cpu memory which corresponds to displaying the screen...
    
       let color_idx = cpu.mem_read(i as u16);
       let (b1, b2, b3) = color(color_idx).rgb();

       // ... get the RGB colours to be displayed 
       if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
        // if any of them don't already have the correct colours...
           frame[frame_idx] = b1;
           frame[frame_idx + 1] = b2;
           frame[frame_idx + 2] = b3;
           update = true;
       } // ... display them by updating the frame array
       frame_idx += 3; // since RGB, 3 entries are updated each run.
   }
   update
}

fn color(byte: u8) -> Color { // White for Snake, Black for BG, varying colours
    // for the apple.
   match byte {
       0 => sdl2::pixels::Color::BLACK,
       1 => sdl2::pixels::Color::WHITE,
       2 | 9 => sdl2::pixels::Color::GREY,
       3 | 10 => sdl2::pixels::Color::RED,
       4 | 11 => sdl2::pixels::Color::GREEN,
       5 | 12 => sdl2::pixels::Color::BLUE,
       6 | 13 => sdl2::pixels::Color::MAGENTA,
       7 | 14 => sdl2::pixels::Color::YELLOW,
       _ => sdl2::pixels::Color::CYAN,
   }
}


fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) { // the address 0xFF stores the lates user input
   for event in event_pump.poll_iter() {
       match event {
           Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
               std::process::exit(0)
           },
           Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
               cpu.mem_write(0xff, 0x77);
           },
           Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
               cpu.mem_write(0xff, 0x73);
           },
           Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
               cpu.mem_write(0xff, 0x61);
           },
           Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
               cpu.mem_write(0xff, 0x64);
           }
           _ => {/* do nothing */}
       }
   }
}

fn main() {
    // init sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("ruNESco CPU: Snake6502 ", (32.0 * 20.0) as u32, (32.0 * 20.0) as u32)
        // 32x32 screen, scaled by a factor of 20.
        .position_centered()
        .build().unwrap();

    // A 'canvas': something which can be 'drawn' on is put over the window
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(20.0, 20.0).unwrap();

    // "Using .unwrap() is justifiable here because it's the outer layer of our application. 
    // There are no other layers that potentially can handle Err values and do something about it."

    // The canvas is given a 'texture': which handles visuals.
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();
        // We specify that the visuals are in the form of 32x32 pixelated grid
        // where each pixel takes 3 bytes (RGB). The texture is thus given by a
        // 32x32x3 array of bytes.

    let game_code = vec![
        0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02,
        0x85, 0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9,
        0x0f, 0x85, 0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85,
        0x00, 0xa5, 0xfe, 0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20,
        0x8d, 0x06, 0x20, 0xc3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c,
        0x38, 0x06, 0xa5, 0xff, 0xc9, 0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0,
        0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60, 0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85,
        0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0, 0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01,
        0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02, 0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05,
        0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06, 0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00,
        0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07, 0xe6, 0x03, 0xe6, 0x03, 0x20,
        0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06, 0xb5, 0x11, 0xc5, 0x11,
        0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c, 0x35, 0x07, 0x60,
        0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02, 0x4a, 0xb0,
        0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9, 0x20,
        0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
        0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10,
        0xb0, 0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5,
        0x10, 0x29, 0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe,
        0x91, 0x00, 0x60, 0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10,
        0x60, 0xa6, 0xff, 0xea, 0xea, 0xca, 0xd0, 0xfb, 0x60,
    ]; // taken from tutorial github, not book!
 
    //load the game
    let mut cpu = CPU::new();
    cpu.load(game_code);
    cpu.reset();
    cpu.program_counter = 0x0600;
 
    let mut screen_state = [0 as u8; 32 * 3 * 32]; // initialise the screen state array
    let mut rng = rand::thread_rng();

    // run the game cycle
    cpu.run_with_callback(move |cpu| { 
        // CPU is moved (explicitly borrowed) so that nothing outside the gameloop
        // can change the CPU state.

        handle_user_input(cpu, &mut event_pump);
        cpu.mem_write(0xfe, rng.gen_range(1, 16)); 
        // writes a random number, stored at 0xfe, spawning an apple with 16 possible
        // random colours when read.

        // [?] How are the random apple positions chosen?
        // [A?] 0xFE is the 6502's default random number generator. The cycles before and after
        // writing to 0xFE will have different random numbers ranging from 0-255. Combining with another
        // random number from 0xFE in the next cycle, we will be able to place an apple in the 32x32 grid.
        // The game code handles this.

        // [?] How is 0xFE, a memory location that has not been explicitly defined to be a RNG, functioning as one?
        // This makes sense for the 6502 testing resource which may have explicilty accounted for it
        // (https://skilldrick.github.io/easy6502), but this emulator has not...

        // [A?] Implicitly handled in a way I don't understand?

        // [A?] There is a possibility that this 16 number RNG is exactly what our emulator accounts for, and the machine code vector
        // is adjusted in a way that it plots the apple positions using the 16 numbers instead of the 256 of the 6502 testing resource.
        // This is supported by the fact that the colours of the machine code version range from 0-256, but only 16 here.
        // However, a hexdump of the testing resource's version of the game on first pass seems to be identical to this vector.

        // [A*] The most likely answer is that the apples are generated exclusively from a pool of 16 positions 
        // (or some function with input of 0-15 instead of 0-255 as in the original game code), instead of the 32x32 = 
        // 1024 positions which could be generated in the the original game code. 

        if read_screen_state(cpu, &mut screen_state) { // update the screen if it needs to be updated
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        ::std::thread::sleep(Duration::new(0, 70_000)); // slows down pace for playability
    });
 }
 