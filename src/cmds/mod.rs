pub mod new;

use crate::cli::{Cli, Commands};
use crate::error::Error;

pub fn run(cli: Cli) -> Result<(), Error> {
    match cli.command {
        Commands::New(args) => new::run(args),
    }
}
