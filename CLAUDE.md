# CLAUDE.md

## Project Overview

Ferrum is a GameBoy (DMG-01) emulator written in Rust, aiming for near-cycle accurate emulation of the original GameBoy hardware. It is a research/hobby project. GameBoy Color support is a future goal.

- **Author**: m0x ([GitHub](https://github.com/m0xsec/ferrum))
- **Rust Edition**: 2021
- **Target Hardware**: Sharp LR35902 CPU (SM83 core) — a Z80/8080 hybrid

## Build & Run

```bash
# Build
cargo build

# Build release
cargo build --release

# Run with a ROM
cargo run -- -r path/to/rom.gb

# Run with logging enabled
RUST_LOG=info cargo run -- -r path/to/rom.gb
```

There is no custom `rustfmt.toml` or `clippy.toml` — use default Rust formatting (`cargo fmt`) and linting (`cargo clippy`).

## Testing

There are no Rust unit tests. Testing is done via Blargg's CPU instruction test ROMs located in `roms/test/blargg/`. Test output is printed to stdout via serial port interception (register `0xFF01`). Run a test ROM and check console output for pass/fail.

## Architecture

The emulator is organized into modules that mirror real GameBoy hardware components:

```
src/
├── main.rs              # Entry point, CLI arg parsing (clap)
├── gb/mod.rs            # GameBoy struct — ties CPU + MMU together, runs emulation loop
├── cpu/
│   ├── mod.rs           # CPU struct, fetch-decode-execute cycle, interrupt handling
│   ├── execute.rs       # Instruction execution (~2000 lines, all opcodes)
│   ├── opcodes.rs       # Opcode lookup table (lazy_static HashMap)
│   ├── registers.rs     # Register definitions (Reg8, Reg16, Flags)
│   └── interrupts.rs    # Interrupt flag definitions
├── mmu/
│   ├── mod.rs           # Memory Management Unit — memory map, I/O register routing
│   └── memory.rs        # Memory trait (read8, write8, read16, write16, cycle)
├── ppu/
│   ├── mod.rs           # Pixel Processing Unit — LCD rendering, mode state machine
│   ├── fetcher.rs       # Pixel FIFO fetcher (background tile data)
│   └── fifo.rs          # FIFO queue for pixel pipeline
├── cartridge/
│   ├── mod.rs           # Cartridge trait + factory (load ROM, detect type)
│   ├── header.rs        # ROM header parsing (title, type, size, checksums)
│   ├── mbc.rs           # MBC type enum
│   └── mbc1.rs          # MBC1 memory bank controller implementation
├── timer/
│   ├── mod.rs           # Timer registers (DIV, TIMA, TMA, TAC) + interrupt firing
│   └── clock.rs         # Clock divider
└── boot/mod.rs          # Boot ROM (256-byte static array)
```

### Key Design Patterns

- **Trait-based abstraction**: `Memory` trait (`mmu/memory.rs`) defines the interface for all memory-mapped components. `Cartridge` trait abstracts ROM types.
- **Interior mutability**: `Rc<RefCell<>>` is used to share the MMU between the CPU and GameBoy struct. The PPU also uses `Rc<RefCell<>>` for VRAM/OAM sharing with the FIFO fetcher.
- **Factory constructors**: Hardware components use `power_on()` or `new()` as constructors.
- **Hardware reference docs**: Code extensively references [gbdev.io](https://gbdev.io) and the Pan Docs for correctness.

### Memory Map (implemented in `mmu/mod.rs`)

| Range | Component |
|-------|-----------|
| `0x0000–0x00FF` | Boot ROM (swappable) |
| `0x0000–0x7FFF` | Cartridge ROM |
| `0x8000–0x9FFF` | VRAM |
| `0xA000–0xBFFF` | External RAM (cartridge) |
| `0xC000–0xDFFF` | WRAM |
| `0xFE00–0xFE9F` | OAM |
| `0xFF00–0xFF7F` | I/O Registers |
| `0xFF80–0xFFFE` | HRAM |
| `0xFFFF` | Interrupt Enable |

## Code Conventions

- **Naming**: Modules and functions use `snake_case`, types use `PascalCase`, constants use `SCREAMING_SNAKE_CASE`.
- **Register access**: `read8`/`write8` for bytes, `read16`/`write16` for words.
- **Logging**: Use the `log` crate (`info!`, `warn!`, etc.) — not `println!` for diagnostics. Controlled via `RUST_LOG` env var.
- **Doc comments**: `///` doc comments on public items. Reference hardware docs (Pan Docs, gbdev.io) in comments where applicable.
- **Commit messages**: Short imperative style, often prefixed with the component (e.g., `ppu: scrolling works`).

## Current Status / WIP Areas

- **CPU**: Fully implemented (all opcodes, CB-prefixed instructions, interrupts).
- **PPU**: Pixel FIFO rendering works for background layer. Window and sprite (object) layers are not yet complete.
- **Timer**: Implemented with interrupt support.
- **Cartridge**: ROM-only and MBC1 supported. Other MBC types not yet implemented.
- **Audio (APU)**: Not implemented.
- **Joypad Input**: Keyboard events are captured via minifb but joypad register mapping is incomplete.
- **Graphics**: 160x144 LCD, DMG greyscale palette, rendered via minifb with X11 backend.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `bitflags` | CPU flag register bit manipulation |
| `clap` | CLI argument parsing (`-r`/`--rom`) |
| `env_logger` | Logger initialization from `RUST_LOG` |
| `lazy_static` | Static opcode lookup table |
| `log` | Logging macros |
| `minifb` | Window creation and framebuffer rendering (X11) |
| `num_enum` | Numeric enum conversions |
| `rand` | Random memory initialization |
| `tinyvec` | Small fixed-size vectors (FIFO) |

## Useful References

- [Pan Docs](https://gbdev.io/pandocs/) — comprehensive GameBoy hardware documentation
- [GB Opcode Table](https://gbdev.io/gb-opcodes/optables/errata) — opcode reference
- [Gameboy-logs](https://github.com/wheremyfoodat/Gameboy-logs) — CPU state log format for debugging
- [Blargg's test ROMs](https://github.com/retrio/gb-test-roms) — CPU instruction verification
