use crate::cpu;
/// Represents the GameBoy DMG-01 hardware.
struct GameBoy {
    // The heart of the Gameboy, the CPU.
    // The CPU is responsible for decoding and executing instructions.
    // The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080.
    // TODO: Implement CPU
    cpu: cpu::CPU,
    // The DMG-01 didn't have an actual Memory Management Unit (MMU), but it had a memory-mapped I/O system with a single RAM chip.
    // To make emulation easier, we will define a MMU.
    // The MMU is responsible for mapping memory addresses to actual memory locations.
    // TODO: Implement MMU
}
