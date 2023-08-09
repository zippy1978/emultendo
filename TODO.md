# TODO

A messy todo list of some of the things that should be done to improve the emulator.

## Core

- Remove self internal memory support for CPU (make bus connection mandatory)
- Handle sprite draw order on PPU
- Consider an new or improved implementation of the PPU inspired by https://github.com/takahirox/nes-rust/blob/master/src/ppu.rs
- Implement APU support
- Support non 0 mappers

## Standalone

- Joysticks support
- Enhance UI

## Debugger

- PPU scroll tracking is not drawn properly when moving to second nametable.
- Find why emulator cycles are slowing down when drawing PPU nametables (should drop frames, but not slowdown !)

