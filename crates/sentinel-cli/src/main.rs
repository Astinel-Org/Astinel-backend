fn main() {
    let exit_code = sentinel_cli::app::run();
    std::process::exit(exit_code.to_i32());
}
