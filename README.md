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
    // Allocate 80000 u32 elements plus a cache that can hold 10 elements at a time
    let mut array = REUArray::<u32>::new(80000, 10);

    println!("Setting Element 1 = 69");
    array[1]=69;
    println!("Setting Element 8 = 666");
    array[8]=666;

    println!("Getting at index 1: {}", array[1]);

    println!("Setting Element 70000 = 999");
    array[70000] = 999;
    println!("Getting at index 8: {}", array[8]);
    println!("Getting at index 70000: {}", array[70000]);
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