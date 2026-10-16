#![no_std] // nie ładuj biblioteki std

pub mod ram_expansion_unit;
pub mod reu_allocator;
pub mod reu_array;
pub mod vectors;

pub use ram_expansion_unit::RamExpanstionUnit;
pub use reu_array::REUArray;

extern "C" {
    fn __enable_mi();
    fn __disable_mi();
}
