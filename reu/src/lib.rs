#![no_std] // nie ładuj biblioteki std
#![feature(panic_info_message)]

pub mod ram_expansion_unit;
pub mod reu_allocator;
pub mod reu_array;

pub use ram_expansion_unit::RamExpanstionUnit;
pub use reu_array::REUArray;

extern "C" {
    // defined in c to allow assembly and interrupt attribute
    fn fake_irq_function();
}