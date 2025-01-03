# ruNESco 🕹️  

An emulator for the [Nintendo Entertainment System (NES)](https://en.wikipedia.org/wiki/Nintendo_Entertainment_System) written in [Rust](https://www.rust-lang.org/), with a focus on **online cooperative multiplayer** over the web.

Project Poster (made with Canva)

![[ruNESco Poster.png]]


---
### Introduction  

ruNESco was inspired by a fascination with emulation and retro gaming, and a desire to stay connected during the global pandemic. The project seeks to recreate the experience of classic NES games, enhanced by the ability to play with friends, even across long distances.

This emulator leverages **Rust** for its performance and reliability, alongside modern technologies such as **SDL2**, **WebAssembly**, and rollback networking to provide a seamless multiplayer retro gaming experience.

---
### Key Features  

- **Rust 🦀:** The core of the project, chosen for its:  
  - **Performance**: Efficient and fast emulation.  
  - **Memory Safety**: Reduced debugging effort and runtime error prevention.  
  - **Concurrency**: Parallelism for tasks such as rendering and low-latency multiplayer.  

- **[Rust-SDL2](https://github.com/Rust-SDL2/rust-sdl2)**: Enables visual rendering via the Picture Processing Unit (PPU), with plans for future audio support through the Audio Processing Unit (APU).  

- **WebAssembly** (WIP): Aims to ensure cross-platform compatibility by allowing the emulator to run directly in web browsers with near-native performance.  

- **Multiplayer with Rollback Netplay** (WIP): Incorporates -
  - [Matchbox](https://github.com/johanhelsing/matchbox) for peer-to-peer connections.  
  - [GGRS](https://github.com/gschup/ggrs) for rollback netcode, ensuring low-latency synchronization of user inputs.  

---
### Current Progress  
#### Completed Goals:  
- [x] **Core NES Emulator**: A fully functional emulator capable of running NES games.  
- [x] **Two-Player Local Multiplayer**: Developed support for a second joypad and player input.  

### Future Goals:  
#### Feature Completeness:  
- [ ] **Audio Processing Unit (APU)**: Implementation of accurate sound emulation.  
- [ ] **Support for Additional NES Mappers**: Expanding the range of compatible games.  
- [ ] **iNES 2.0 Compatibility**: Enabling support for a broader variety of ROMs.  

#### Online Multiplayer:  
- [ ] **Fully Functional Online Multiplayer**: Integration of rollback netplay for synchronized multiplayer.  
- [ ] **4+ Player Support**: Supporting cooperative and party-style games.  

#### Cross-Platform Accessibility:  
- [ ] **WebAssembly Compilation**: Allowing the emulator to run directly in modern web browsers.  

---
### How to Use  

1. **Clone the repository**:  
   
```
git clone https://github.com/yourusername/ruNESco.git 
cd ruNESco
```

2. **Build the project (ensure that you have installed Rust):**

```
cargo build --release
```

3. **Load the rom:**
	- You will have to provide the .nes ROM file.
	- Place it in the runesco folder: the same location SDL2.dll is stored in
	- Navigate to the "src" folder, and update line 169 of main.rs to the name of your rom.

```
let nes_file_data: Vec<u8> = std::fs::read("<name_of_your_rom>.nes").unwrap();
```

4. Check the control configuration:
	- Player 1:
		- A - Z
		- B - X
		- Select - Right Shift
		- Start - Enter
		- $\uparrow$ - Up Arrow Key
		- $\downarrow$ - Down Arrow Key
		- $\leftarrow$ - Left Arrow Key
		- $\rightarrow$ - Right Arrow Key
	-  Player 2:
		- A - XBox 360 (or equivalent Controller) A
		- B - Controller B
		- Select - Controller Back
		- Start - Controller Up
		- $\uparrow$ - Controller DPad Up
		- $\downarrow$ - Controller DPad Down
		- $\leftarrow$ - Controller DPad Left
		- $\rightarrow$ - Controller DPad Right
	- To remap these bindings, in main.rs, you may edit the following lines (174 - 192)

```
    let mut p1 = HashMap::new();
    p1.insert(Keycode::Down, joypads::JoypadButton::DOWN);
    p1.insert(Keycode::Up, joypads::JoypadButton::UP);
    p1.insert(Keycode::Right, joypads::JoypadButton::RIGHT);
    p1.insert(Keycode::Left, joypads::JoypadButton::LEFT);
    p1.insert(Keycode::RShift, joypads::JoypadButton::SELECT);
    p1.insert(Keycode::Return, joypads::JoypadButton::START);
    p1.insert(Keycode::Z, joypads::JoypadButton::BUTTON_A);
    p1.insert(Keycode::X, joypads::JoypadButton::BUTTON_B);

    let mut p2 = HashMap::new();
    p2.insert(Button::DPadDown, joypads::JoypadButton::DOWN);
    p2.insert(Button::DPadUp, joypads::JoypadButton::UP);
    p2.insert(Button::DPadRight, joypads::JoypadButton::RIGHT);
    p2.insert(Button::DPadLeft, joypads::JoypadButton::LEFT);
    p2.insert(Button::Back, joypads::JoypadButton::SELECT);
    p2.insert(Button::Start, joypads::JoypadButton::START);
    p2.insert(Button::A, joypads::JoypadButton::BUTTON_A);
    p2.insert(Button::B, joypads::JoypadButton::BUTTON_B);

```

5. **Run the emulator:**
   
```
cargo run --release
```

---
### Dependencies

To build and run the project, ensure you have the following dependencies installed:  

- [Rust](https://www.rust-lang.org/): The programming language used for this project.  
- [Rust-SDL2](https://github.com/Rust-SDL2/rust-sdl2): A Rust binding for the SDL2 library, required for rendering.  

---
### Acknowledgments

This project was made possible thanks to:  
- [Rafael Bagmanov](https://github.com/bugzmanov/): Author of the tutorial *[Writing \[an\] NES Emulator in Rust](https://bugzmanov.github.io/nes_ebook/chapter_1.html)* 
  of which this project is essentially a fork!
- The [NESdev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki): An invaluable resource for technical documentation.
- The [r/EmuDev](https://www.reddit.com/r/EmuDev/) (and [r/Rust](https://www.reddit.com/r/rust/)) community, for answering my endless questions.
- [Rodrigo Alfonso](https://github.com/afska): creator of [NEStation](https://github.com/afska/nestation), and [Ted Steen](https://github.com/tedsteen): developer of [NES Bundler](https://github.com/tedsteen/nes-bundler), for inspiration on multiplayer.

---
### Contributing

Contributions are welcome! Please feel free to submit issues, fork the repository, or open pull requests.

---
### License

This project is licensed under the GPL-3.0 License. See the LICENSE file for more information.
