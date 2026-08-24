mod cli;
mod cmds;
mod error;
mod models;
mod services;
mod templates;

use wrcli::WrCliError;

fn main() {
    if let Err(error) = cli::build_cli().execute() {
        eprintln!("error: {error}");
        std::process::exit(exit_code(&error));
    }
}

/// wrcli 에러를 종료 코드로 매핑한다. 사용자가 인자를 바꾸면 해결되는 오류는 2,
/// 그 외 환경/시스템 오류는 1.
fn exit_code(error: &WrCliError) -> i32 {
    if let WrCliError::UserError(e) = error
        && let Some(app) = e.downcast_ref::<error::Error>()
    {
        return app.exit_code();
    }

    match error {
        WrCliError::UnknownFlag { .. }
        | WrCliError::UnknownSubcommand { .. }
        | WrCliError::MissingRequiredFlag(_)
        | WrCliError::MissingFlagValue(_)
        | WrCliError::InvalidFlagValue { .. }
        | WrCliError::ArgValidationFailed(_)
        | WrCliError::CommandHasNoRunner(_)
        | WrCliError::UnsupportedCompletionShell(_) => 2,
        _ => 1,
    }
}
