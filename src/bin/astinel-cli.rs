fn main() {
    let exit_code = astinel_backend::cli::app::run();
    std::process::exit(exit_code.to_i32());
}
