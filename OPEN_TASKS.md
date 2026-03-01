# Ferrum GameBoy Emulator — Open Tasks & Improvements

## Context

This document catalogs all open tasks, bugs, incomplete features, and improvement opportunities identified in the Ferrum codebase. Items are organized by severity and component, sourced from TODO comments, code analysis, and comparison against GameBoy hardware specifications (Pan Docs).

---

## Critical Bugs

### 1. PPU Bitwise Operation Errors
**Files:** `src/ppu/mod.rs:310`, `src/ppu/mod.rs:325`

Two LCDC register methods use multiplication (`*`) instead of bitwise AND (`&`):
```rust
fn window_tile_map_select(&self) -> bool {
    self.data * (1 << 6) != 0  // BUG: should be &
}
fn tile_data_select(&self) -> bool {
    self.data * (1 << 4) != 0  // BUG: should be &
}
```
These return incorrect values for nearly all inputs, breaking tile map and tile data address selection.

### 2. LYC Register Read Returns Wrong Value
**File:** `src/ppu/mod.rs:654`

Reading `0xFF45` (LYC) returns `self.ly` instead of `self.lyc`. Games that rely on LY==LYC comparison for raster effects will break.

### 3. LYC Register Write Handler Missing
**File:** `src/ppu/mod.rs` (write8 method)

No write handler for `0xFF45`. Games cannot set the LYC compare value, breaking STAT interrupt-based effects.

---

## Missing Core Features (High Priority)

### 4. Joypad Input Not Mapped
**File:** `src/gb/mod.rs:114`
- `TODO: Handle Gameboy Joypad input.`
- Keyboard events are captured but only Escape and Space are handled
- Register `0xFF00` (P1/JOYPAD) not implemented in MMU
- No keyboard-to-GameBoy-button mapping
- No joypad interrupt firing
- **Impact:** Games are completely unplayable

### 5. Sprite (OBJ) Rendering Not Implemented
**File:** `src/ppu/mod.rs:805`
- `TODO: OAM search will happen here (when implemented).`
- OAM scanning is stubbed — no sprite evaluation per scanline
- Sprite struct is defined (`src/ppu/mod.rs:157-217`) but never instantiated
- `sprites: Vec<Sprite>` exists but is never populated
- `init_sprites()` method exists but is never called
- No sprite pixel mixing during drawing mode
- **Impact:** No sprites visible in any game

### 6. Window Layer Not Rendered
**File:** `src/ppu/mod.rs`
- `window_tiles`, `window_map`, `window_fetch` fields exist but are unused
- Window position registers (WX, WY) are readable/writable but ignored during rendering
- `window_tile_map_select()` has the multiplication bug (item #1)
- **Impact:** HUDs, menus, and text boxes missing in many games

### 7. PPU Layer Enable Flags Not Synced with LCDC
**File:** `src/ppu/mod.rs:480-482, 587-589`
- `bg_enabled`, `window_enabled`, `sprite_enabled` are hardcoded to `false` on init
- Never updated when LCDC register is written
- Games cannot toggle layers on/off

### 8. DMA Transfer Not Implemented
- Register `0xFF46` (OAM DMA) has no handler
- Games rely on DMA for sprite loading — without it, OAM is never populated even if sprite rendering were implemented

### 9. Cartridge Type Support Limited
**File:** `src/cartridge/mod.rs:63-64`
- `TODO: Implement other cartridge types.`
- Only RomOnly and MBC1 supported
- Unsupported types (`todo!()` panic): MBC2, MBC3, MBC5, MBC6, MBC7, MMM01, HuC1, HuC3
- **Impact:** Large portion of GameBoy library unplayable

---

## Missing Features (Medium Priority)

### 10. Audio/APU Not Implemented
**File:** `src/gb/mod.rs:28`
- `TODO: Look at using cpal for audio output, spin up a thread to handle audio, etc.`
- No APU module exists at all
- Audio registers `0xFF10-0xFF3F` fall through to generic I/O buffer
- APU not cycled in MMU tick loop (`src/mmu/mod.rs:260`)

### 11. Battery-Backed RAM Persistence
**File:** `src/cartridge/mbc1.rs:4`
- `TODO: Implement saving and loading of battery backed RAM.`
- Game saves are lost when emulator closes
- Also mentioned in `src/gb/mod.rs:128`: `TODO: Handle emulation exit, such as saving RAM to file...`

### 12. Tile Data Addressing Mode Not Implemented
**File:** `src/ppu/mod.rs:825-828`
- Fetcher always uses tile map base `0x9800`
- LCDC.3 (BG tile map select) and LCDC.4 (tile data select) not used
- Missing signed tile index mode (0x8800 base)

### 13. I/O Register Coverage Incomplete
**File:** `src/mmu/mod.rs:164, 209`
- `TODO: Implement the rest of the IO registers.` (appears twice)
- Many registers silently fall through to generic `self.io[]` buffer
- Missing dedicated handlers for: joypad (0xFF00), serial control (0xFF02), all audio (0xFF10-0xFF3F)

---

## Code Quality & Robustness

### 14. Panic Points in Critical Paths
| Location | Issue |
|----------|-------|
| `src/ppu/fifo.rs:32,43` | `panic!("FIFO is full/empty")` — crashes on timing edge cases |
| `src/timer/mod.rs:56,76,82` | `panic!("Unsupported address")` / `panic!("")` — crashes on invalid timer access |
| `src/cartridge/mod.rs:64` | `todo!()` — crashes on unsupported cartridge types |
| `src/cpu/execute.rs:13,1174` | `.unwrap()` on opcode lookup — crashes if opcode missing |

### 15. Illegal Opcode Handling
**File:** `src/cpu/execute.rs:1157-1159`
- Illegal opcodes only log a warning, CPU continues executing
- Real hardware behavior varies — should at minimum halt

### 16. Cycle Count Return Value
**File:** `src/mmu/mod.rs:259-272`
- `cycle()` returns `cpu_ticks + gpu_ticks` but timer ticks are not included
- Return value may be incorrect

### 17. LCD Off State Incomplete
**File:** `src/ppu/mod.rs:726-742`
- When LCD disabled: VRAM/OAM accessibility not updated, STAT register not cleared, mode not properly reset

### 18. Commented-Out / Dead Code
| Location | Description |
|----------|-------------|
| `src/cpu/registers.rs:211,221,231` | Commented-out overflow panics |
| `src/cpu/mod.rs:92` | `_debug_print_state()` — unused debug function |
| `src/mmu/mod.rs:176-177` | Commented-out LY hardcode |
| `src/mmu/mod.rs:244-246` | Commented-out alternate read16 |
| `src/ppu/mod.rs:869` | `//todo!("PPU is a WIP...")` |

---

## Low Priority / Future Work

### 19. Serial Port — Output Only
**File:** `src/mmu/mod.rs:215-220`
- Serial data register (0xFF01) only prints to stdout
- Serial control register (0xFF02) not implemented
- No link cable emulation

### 20. Register Initialization
**File:** `src/cpu/registers.rs:88-101`
- All registers initialized to 0x00
- After boot ROM, Pan Docs specifies: A=0x01, F=0xB0, B=0x00, C=0x13, D=0x00, E=0xD8, H=0x01, L=0x4D
- Currently relies on boot ROM to set these, which is correct if boot ROM runs

### 21. CGB (GameBoy Color) Support
- Not started — listed as future goal in project README

### 22. Unit Tests
- No Rust unit tests exist
- Testing relies solely on Blargg's test ROMs run manually

### 23. Missing `cpal` Dependency
- Code TODO references `cpal` for audio but it's not in `Cargo.toml`

---

## Summary by Priority

| Priority | Count | Key Items |
|----------|-------|-----------|
| **Critical (Bugs)** | 3 | PPU bitwise bugs, LYC read/write errors |
| **High (Missing Core)** | 6 | Joypad, sprites, window, DMA, layer flags, cartridge types |
| **Medium (Missing)** | 4 | APU, save RAM, tile addressing, I/O registers |
| **Code Quality** | 5 | Panics, dead code, cycle counts, LCD off, illegal opcodes |
| **Low / Future** | 5 | Serial, register init, CGB, tests, dependencies |

---

## Verification

After addressing any items:
- Run `cargo build` and `cargo clippy` to verify compilation
- Run Blargg's CPU instruction test ROMs (`roms/test/blargg/`) to verify CPU correctness
- Visually verify rendering with known-good ROMs
