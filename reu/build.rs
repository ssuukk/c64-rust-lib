fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    if target.starts_with("mos-c64") {
        cc::Build::new()
            .compiler("mos-c64-clang")
            .file("src/fake_interrupt.S")
            .compile("reu_interrupt");
    }
}