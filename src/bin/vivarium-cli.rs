#[path = "../cli/mod.rs"]
mod cli;
#[path = "../core/mod.rs"]
mod core;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}