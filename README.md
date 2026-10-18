# Commodore 64 Rust Libraries

Various C64 Rust utilities that I coded while learning Rust.

# REU library

The default demo does not require a REU. Build the REU smoke-test image with `cargo build -p c64-rust-lib --bin demo --features reu-hardware-tests`; the demo initializes an 8 MiB allocator, so configure the emulator with an 8 MiB REU before running it.

## Basic REU wrapper

Some obvious REU operations wrapped into Rust functions:

```Rust
let reu = ram_expansion_unit::reu();
reu.set_range(1024, 0x050000, 1000);  // prepare REU working range
reu.pull(); // get data from REU into RAM
reu.push(); // put data from RAM to REU
reu.swap(); // swap RAM and REU
reu.fill(1024, 1000, 32); // clear screen using REU DMA
reu.fill_reu(0x030000, 10000, 0); // fill some REU address with 0s
```

## REU allocator

A simple memory allocator returning 24-bit pointer that knows its block size, for cleaner syntax. Allocated chunks get properly dropped. Minimum allocation size = 256 bytes.

```Rust
let reu = ram_expansion_unit::reu();
    reu.init_allocator(8 * 1024 * 1024);  // initialize the BAM for an 8 MiB REU
let screen_memory = reu.alloc(1000);  // alloc 1000 bytes somewhere in REU
screen_memory.push(1024);  // push RAM starting at 1024 into REU chunk
...
screen_memory.pull(1024);  // restore REU chunk into RAM starting at 1024
// screen_memory will be deallocated properly
```

## Array stored in REU

Fixed-capacity array stored in REU and paged through a small RAM cache. Values must implement `Copy`; use `get` and `set` for element access, or `with_mut` for scoped mutation. `for_each_mut` visits the initialized elements without returning references into the cache.

```Rust
#[derive(Clone, Copy)]
struct GameUnit {
    number: u8,
    speed: u8,
    x: u16,
    y: u16,
}

pub fn test_reu_array() {
    // Cache ten elements at a time while storing up to 100,000 in REU.
    let mut array = REUArray::<GameUnit>::with_capacity(100_000, 10);

    for i in 0..100 {
        array.push(GameUnit {
            number: i as u8,
            speed: 1,
            x: 3,
            y: 4,
        });
    }

    array.with_mut(50, |unit| {
        unit.speed = 0xa;
    });

    array.for_each_mut(|_, unit| {
        unit.x += 1;
    });
}
```

# Ultimate 64 speed registers

Set clock speed of Ultimate 64:

```Rust
    ultimate_64::get().set_speed(ultimate_64::Timings::MHZ_48.bits());
    ultimate_64::get().set_enable(ultimate_64::Turbo::ENABLE.bits());
```

# A Hires screen library

```Rust
pub fn test_hires() {
    const HIRES: *const C64HiresScreen = 0x2000 as _;
    const COLORS: *const C64TextScreen = 1024 as _;

    let vic = c64::vic2();

    unsafe {
        vic.border_color.write(COLOR_BLACK);

        (*COLORS).clear(32);

        plotek::show(cia2::VicBankSelect::VIC_0000, ScreenBank::AT_0400);

        (*HIRES).clear(0);

        for i in 0..40 {
            (*HIRES).line((0, 0), (i * 8, 199));
        }
    }

    wait_for_return();

    plotek::hide();
}
```

# Editor setup (VS Code + rust-analyzer)

The project pins [`channel = "mos-stable"`](rust-toolchain.toml); rust-mos 1.99 does **not** ship its own `rust-analyzer` or `rust-analyzer-proc-macro-srv` inside its sysroot. The official stable component owns both binaries, so the editor needs them too. Run once after any rust-mos reinstall:

```sh
./scripts/setup-rust-analyzer-sysroot.sh
```

The script creates a single `rust-analyzer-proc-macro-srv` symlink inside the rust-mos sysroot that points at the stable toolchain's proc-macro server. Nothing else is touched. The matching `.vscode/settings.json` then tells the rust-analyzer extension to:

- load the editor-side rust-analyzer from `stable` (which is where its bundle is installed),
- analyse the workspace against the `mos-c64-none` target declared in `.cargo/config.toml`,
- leave proc-macro expansion enabled (the symlink above is what makes that work).

If you need to undo the sysroot patch:

```sh
./scripts/setup-rust-analyzer-sysroot.sh --unlink
```
