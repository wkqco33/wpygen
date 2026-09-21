mod cli;
mod cmds;
mod error;
mod models;
mod report;
mod services;
mod templates;
#[cfg(test)]
mod testing;

use wrcli::WrCliError;

fn main() {
    if let Err(error) = cli::build_cli().execute() {
        eprintln!("error: {error}");
        std::process::exit(exit_code(&error));
    }
}

/// wrcli 에러를 종료 코드로 매핑한다. 사용자가 인자를 바꾸면 해결되는 오류는 2,
/// 그 외 환경/시스템 오류는 1. 앱 고유 오류(`error::Error`)만 자체 규칙으로 판정하고,
/// 나머지는 wrcli의 분류(`is_usage_error`)를 그대로 따른다.
fn exit_code(error: &WrCliError) -> i32 {
    if let WrCliError::UserError(e) = error
        && let Some(app) = e.downcast_ref::<error::Error>()
    {
        return app.exit_code();
    }

    error.exit_code()
}
