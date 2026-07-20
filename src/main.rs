mod cli;
mod cmds;
mod error;
mod models;
mod services;
mod templates;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    if let Err(error) = cmds::run(cli) {
        eprintln!("error: {error}");
        std::process::exit(error.exit_code());
    }
}
