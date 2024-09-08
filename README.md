# 🕹️ ruNESco 🎮

An emulator for the[ Nintendo Entertainment System (NES)](https://en.wikipedia.org/wiki/Nintendo_Entertainment_System) written in [Rust](https://www.rust-lang.org/), which supports cooperative online multiplayer over any modern web browser using [Web Assembly](https://webassembly.org/), and peer-to-peer communication through [Matchbox](https://github.com/johanhelsing/matchbox). 

The idea for ruNESco started during the lockdown with a personal fascination with emulators and playing retro games, along with constantly looking for free multiplayer games to play together in that period.
##### [Rust](https://www.rust-lang.org/) 🦀
Rust is an excellent programming language for emulation, as it allows for
- Enhanced performance, allowing for faster emulation
- Memory safety, allowing for ease in debugging and preventing potential errors before they can occur
- Concurrency, crucial for emulation of parallel tasks on a single CPU, as well as for minimal latency for multiplayer 
##### [Web Assembly](https://webassembly.org/) 🕸️
WebAssembly (wasm) is a assembly-like language with near-native performance and provides a language like Rust a compilation target so that it can run on the web. This would allow crossplay between any platform with a modern browser, preventing having to ensure compability between operating systems.
##### [Matchbox](https://github.com/johanhelsing/matchbox) 🔥
Matchbox is a rust based tool which allows users to use peer-to-peer WebRTC (Web Real-Time-Communication) networking for rust's native and wasm applications, in order to facilitate low-latency online gameplay.

---
### Primary Goals 💪
- [ ] Implement a fully-functional and feature complete NES emulator
- [ ] Implement basic compilation to a website
- [ ] Implement 2-4 Player Multiplayer (as allowed for by original hardware like the Nintendo Four-Score or the NES Satellite)
### Stretch Goals 🦾
- [ ] 4+ Players (max limit: 8, or maybe more?)
- [ ] A prettier website

---
## Credits 🖋️

- [Rafael Bagmanov](https://github.com/bugzmanov/), for his tutorial on '[Writing \[an\] NES Emulator in Rust](https://bugzmanov.github.io/nes_ebook/chapter_1.html)'.
- The [Nesdev](https://www.nesdev.org/wiki/Nesdev_Wiki) Wiki and its contributors, for every resource imaginable.
- [Rodrigo Alfonso](https://github.com/afska) for [NEStation](https://github.com/afska/nestation#nestation), the [inspiration](https://forums.nesdev.org/viewtopic.php?t=19090) for the multiplayer implementation.
- [Johan Klokkhammer Helsing](https://github.com/johanhelsing) for [Matchbox](https://github.com/johanhelsing/matchbox), without which I'd have to learn [WebRTC](https://webrtc.org/) / [WebRTC.rs](https://github.com/webrtc-rs/webrtc) on my own.
