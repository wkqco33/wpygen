pub mod new;

use clap::CommandFactory;

use crate::cli::{Cli, Commands};
use crate::error::Error;

pub fn run(cli: Cli) -> Result<(), Error> {
    match cli.command {
        Commands::New(args) => new::run(args),
        Commands::Completions { shell } => {
            generate_completions(shell);
            Ok(())
        }
    }
}

fn generate_completions(shell: clap_complete::Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
}
