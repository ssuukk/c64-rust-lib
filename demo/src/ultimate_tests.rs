pub fn turbo() {
    ultimate64::get().set_speed(ultimate64::Timings::MHZ_48.bits());
    ultimate64::get().set_enable(ultimate64::Turbo::ENABLE.bits());
}
