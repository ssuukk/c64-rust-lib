use volatile_register::{RO, RW};

#[repr(C, packed)]
pub struct InterruptVectors {
    // FFFA-B = NMI
    pub nmi: RW<u16>,
    // FFFC-D = RESET
    pub reset: RW<u16>,
    // FFFE-F = IRQ
    pub irq: RW<u16>,
}

pub const INTERRUPT_VECTORS: *const InterruptVectors = (0xFFFA) as _;