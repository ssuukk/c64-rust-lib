// https://github.com/mrk-its/mos-test
// https://crates.io/crates/defmt-test
// state shared across unit tests
struct MyState {
    flag: bool,
}

#[mos_test::tests]
mod tests {
    #[init]
    fn init() -> super::MyState {
        // state initial value
        super::MyState {
            flag: true,
        }
    }

    // This function is called before each test case.
    // It accesses the state created in `init`,
    // though like with `test`, state access is optional.
    #[before_each]
    fn before_each(state: &mut super::MyState) {
        defmt::println!("State flag before is {}", state.flag);
    }

    // This function is called after each test
    #[after_each]
    fn after_each(state: &mut super::MyState) {
        defmt::println!("State flag after is {}", state.flag);
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
}