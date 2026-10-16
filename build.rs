fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    
    // Skip cc compilation for MOS 6502 targets
    if target.contains("mos-c64") || target.contains("mos-") {
        return;
    }
    
    cc::Build::new()
        .compiler("clang")
        .target(&target)
        //.file("reu/src/fake_interrupt.c")
        .file("reu/src/fake_interrupt.S")
        .compile("fake_interrupt");
}
