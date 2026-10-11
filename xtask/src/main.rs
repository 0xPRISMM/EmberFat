fn main() {
    if let Err(error) = nih_plug_xtask::main() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
