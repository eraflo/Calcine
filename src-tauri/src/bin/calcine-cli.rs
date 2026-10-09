//! `calcine-cli`: Calcine's API without the window. A console program, so
//! terminals wait for it and show its output (the app itself is a windowed
//! program).

fn main() {
    std::process::exit(calcine_lib::cli_main());
}
