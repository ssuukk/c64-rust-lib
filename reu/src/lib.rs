#![no_std] // nie ładuj biblioteki std
#![feature(panic_info_message)]

pub mod ram_expansion_unit;
pub mod reu_allocator;
pub mod reu_array;

pub use reu_array::REUArray;
pub use ram_expansion_unit::RamExpanstionUnit;
pub use reu_allocator::WAllocator;
