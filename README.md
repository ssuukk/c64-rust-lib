# Commodore 64 Rust Libraries

Various C64 Rust utilities that I coded while learning Rust.

# REU library

## Basic REU wrapper

Some obvious REU operations wrapped into Rust functions:

```Rust
let reu = ram_expansion_unit::reu();
reu.set_range(1024, 0x050000, 1000);  // prepare REU working range
reu.pull(); // get data from REU into RAM
reu.push(); // put data from RAM to REU
reu.swap(); // swap RAM and REU
reu.fill(1024,1000,32); // clear screen using REU DMA
reu.fill_reu(0x030000, 10000, 0); // fill some REU address with 0s
```

## REU allocator

A simple memory allocator returning 24-bit pointer that knows its block size, for cleaner syntax. Allocated chunks get properly dropped. Minimum allocation size = 256 bytes.

```Rust
let reu = ram_expansion_unit::reu();
reu.init_allocator();  // prepare BAM in REU
let screen_memory = reu.alloc(1000);  // alloc 1000 bytes somewhere in REU
screen_memory.push(1024);  // push RAM starting at 1024 into REU chunk
...
screen_memory.pull(1024);  // restore REU chunk into RAM starting at 1024
// screen_memory will be deallocated properly
```

## Array stored in REU

`u32` indexable array that is kept in REU, with all Rust goodies. The size of the array is limited only by REU size.

```Rust
#[derive(Clone)]
struct GameUnit {
    number: u8,
    speed: u8,
    health: u8,
    x: u16,
    y: u16,
}

pub fn test_reu_array() {
    // Allocate 100,000 GameUnit elements 10 elements will be cached in RAM
    let mut array = REUArray::<GameUnit>::with_capacity(100_000, 10);

    for i in 0..100 {
        array.push(GameUnit {
            number: i as u8,
            speed: 1,
            health: 2,
            x: 3,
            y: 4,
        });
    }

    array[50].speed = 0xa;
    array[50].health = 0xb;
    array[50].x = 69;
    array[50].y = 11;

    let sixty_nine = array.iter_mut().filter(|unit| unit.x == 69);

    for u in sixty_nine {
        println!("{} Unit at x=69: ({},{})", u.number, u.x, u.y);
    }
}
```

# Ultimate 64 speed registers

Set clock speed of Ultimate 64:

```
    ultimate_64::get().set_speed(ultimate_64::Timings::MHZ_48.bits());
    ultimate_64::get().set_enable(ultimate_64::Turbo::ENABLE.bits());
```

# A Hires screen library

```Rust
pub const HIRES: *const C64HiresScreen = (0xa000) as _;
pub const COLORS: *const C64TextScreen = (0x8400) as _;

fn test_hires() {
    unsafe {
        (*HIRES).clear(0);
        (*COLORS).clear(33);
        //reu::reu().fill(0xa000, 8000, 0);
        plotek::show(VicBankSelect::VIC_8000, ScreenBank::AT_0400);

        for i in 0..32 {
            (*HIRES).line((0,0),(i*10,199));
        }
    
        wait_for_return();

        plotek::hide();
    }
}
```