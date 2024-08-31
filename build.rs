fn main() {
    cc::Build::new()
        .compiler("clang")
        .target("mos-c64")
        .file("reu/src/fake_interrupt.c")
        .compile("fake_interrupt");
}