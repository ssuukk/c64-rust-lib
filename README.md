# `c64-rust-lib`

Various C64 Rust utilities:

- A Hires screen library

```
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

- Basic REU functions (swap_in, swap_out, swap)
- REUArray (up to 16mb array transparently swapped into C64 memory as needed)

```
#[derive(Clone)]
struct GameUnit {
    speed: u8,
    health: u8,
    x: u16,
    y: u16,
}

fn test_reu_slice() {
    // Allocate 100,000 GameUnit elements plus a cache that can hold 10 elements at a time
    let mut array = REUArray::<GameUnit>::new(100000, 10);

    println!("Setting Unit 1");
    array[1]=GameUnit { speed: 1, health: 2, x: 3, y: 4 };
    println!("Setting Unit 8");
    array[8]=GameUnit { speed: 3, health: 20, x: 10, y: 20 };

    println!("Getting health at index 1: {}", array[1].health);

    println!("Setting x of Element 70000 to 100");
    array[70000].x = 100;
    println!("Getting speed at index 8: {}", array[8].speed);
    println!("Getting x at index 70000: {}", array[70000].x);

    // find all ead units
    let dead_units = array.into_iter().filter(|unit| unit.health == 0);

    // print coords of dead units
    for u in dead_units {
        println!("Died at: ({},{})", u.x, u.y);
    }
}
```

- A simple REU allocator (minimum allocation unit - 256 bytes) plus 24-bit pointer

```
        let ptr: Ptr24 = reu::reu().alloc(70000);
```

- Ultimate 64 speed registers

```
    ultimate_64::get().set_speed(ultimate_64::Timings::MHZ_48.bits());
    ultimate_64::get().set_enable(ultimate_64::Turbo::ENABLE.bits());
```