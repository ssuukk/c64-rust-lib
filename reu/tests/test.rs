#![no_std] // nie ładuj biblioteki std
#![no_main] // No main function is required

extern crate mos_alloc;

use core::panic::PanicInfo;

// Provide a panic handler function
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// https://github.com/mrk-its/mos-test
// https://crates.io/crates/defmt-test
// state shared across unit tests
struct MyState {
    flag: bool,
}

#[mos_test::tests]
mod tests {
    use core::mem::size_of;
    use reu::ram_expansion_unit::{Command, Control, RamExpanstionUnit, Status};

    #[init]
    fn init() -> super::MyState {
        // state initial value
        super::MyState { flag: true }
    }

    // This function is called before each test case.
    // It accesses the state created in `init`,
    // though like with `test`, state access is optional.
    #[before_each]
    fn before_each(state: &mut super::MyState) {
        //mos_test::println!("State flag before is {}", state.flag);
    }

    // This function is called after each test
    #[after_each]
    fn after_each(state: &mut super::MyState) {
        //defmt::println!("State flag after is {}", state.flag);
    }

    // this unit test doesn't access the state
    #[test]
    fn assert_true() {
        assert!(true);
    }

    // but this test does
    #[test]
    fn assert_flag(state: &mut super::MyState) {
        assert!(state.flag);
        state.flag = false;
    }

    #[test]
    fn reu_register_block_has_expected_size() {
        assert!(size_of::<RamExpanstionUnit>() == 11);
    }

    #[test]
    fn reu_status_bits_match_hardware_register() {
        assert!(Status::INTERRUPT_PENDING.bits() == 0x80);
        assert!(Status::END_OF_BLOCK.bits() == 0x40);
        assert!(Status::FAULT.bits() == 0x20);
        assert!(Status::SIZE.bits() == 0x10);
    }

    #[test]
    fn reu_command_bits_match_hardware_register() {
        assert!(Command::EXECUTE.bits() == 0x80);
        assert!(Command::AUTOLOAD.bits() == 0x20);
        assert!(Command::NO_FF00_DECODE.bits() == 0x10);
        assert!(Command::TO_REU.bits() == 0);
        assert!(Command::FROM_REU.bits() == 1);
        assert!(Command::SWAP.bits() == 2);
        assert!(Command::VERIFY.bits() == 3);
    }

    #[test]
    fn reu_address_control_bits_match_hardware_register() {
        assert!(Control::FIX_C64.bits() == 0x80);
        assert!(Control::FIX_REU.bits() == 0x40);
        assert!(Control::NONE.bits() == 0);
    }
}
