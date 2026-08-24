use std::path::PathBuf;

use wrcli::args::exact_args;
use wrcli::style::{Color, Panel, Style, stdout_is_styled};
use wrcli::{Command, CommandContext, Flag, FlagValue, WrCliError};

use crate::cmds;
use crate::error::Error;
use crate::models::TemplateKind;

/// `wpygen` CLI 트리를 빌드한다. 각 서브커맨드의 `on_run_e` 콜백에서 파싱된
/// `CommandContext`를 앱의 실행 로직(`cmds::*`)으로 연결한다.
pub fn build_cli() -> Command {
    Command::new("wpygen")
        .short("Python 프로젝트 템플릿(CLI/GUI/SERVER) 생성기")
        .flag(Flag::new("version", FlagValue::Bool(false), "버전 정보를 출력한다.").short('V'))
        .on_run(|ctx| {
            if ctx.flags.get_bool("version").unwrap_or(false) {
                print_version();
            } else {
                print_usage();
            }
        })
        .subcommand(
            Command::new("new")
                .short("새 Python 프로젝트 템플릿을 생성한다.")
                .args(exact_args(1))
                .flag(
                    Flag::new(
                        "template",
                        FlagValue::String(String::new()),
                        "생성할 템플릿 종류 (cli|gui|server)",
                    )
                    .short('t')
                    .required(),
                )
                .flag(Flag::new(
                    "grpc",
                    FlagValue::Bool(false),
                    "gRPC 세팅을 함께 생성한다.",
                ))
                .flag(Flag::new(
                    "sqlite",
                    FlagValue::Bool(false),
                    "SQLite 세팅을 함께 생성한다.",
                ))
                .flag(
                    Flag::new(
                        "output",
                        FlagValue::String(".".to_owned()),
                        "프로젝트를 생성할 상위 디렉터리",
                    )
                    .short('o'),
                )
                .flag(Flag::new(
                    "package-name",
                    FlagValue::String(String::new()),
                    "Python 패키지명 (기본값: 프로젝트명 기반 정규화)",
                ))
                .flag(Flag::new(
                    "force",
                    FlagValue::Bool(false),
                    "대상 디렉터리가 비어있지 않아도 파일을 덮어쓴다.",
                ))
                .flag(
                    Flag::new(
                        "verbose",
                        FlagValue::Bool(false),
                        "생성 진행 상황을 상세히 출력한다.",
                    )
                    .short('v'),
                )
                .flag(Flag::new(
                    "dry-run",
                    FlagValue::Bool(false),
                    "실제로 파일을 쓰지 않고, 생성될 파일 목록만 출력한다.",
                ))
                .flag(Flag::new(
                    "git",
                    FlagValue::Bool(false),
                    "생성 후 `git init` 및 최초 커밋을 실행한다.",
                ))
                .flag(Flag::new(
                    "sync",
                    FlagValue::Bool(false),
                    "생성 후 `uv sync` 를 실행한다.",
                ))
                .flag(Flag::new(
                    "lock",
                    FlagValue::Bool(false),
                    "생성 후 `uv lock` 을 실행한다.",
                ))
                .on_run_e(|ctx| {
                    let args = NewArgs::from_ctx(ctx).map_err(WrCliError::user)?;
                    cmds::new::run(args).map_err(WrCliError::user)
                }),
        )
        .subcommand(
            Command::new("completions")
                .short("쉘 자동완성 스크립트를 표준 출력으로 생성한다.")
                .args(exact_args(1))
                .on_run_e(|ctx| {
                    let shell = &ctx.args[0];
                    let script = build_cli().gen_completion(shell)?;
                    print!("{script}");
                    Ok(())
                }),
        )
}

/// `new` 서브커맨드의 파싱 결과.
#[derive(Debug)]
pub struct NewArgs {
    pub name: String,
    pub template: TemplateKind,
    pub grpc: bool,
    pub sqlite: bool,
    pub output: PathBuf,
    pub package_name: Option<String>,
    pub force: bool,
    pub verbose: bool,
    pub dry_run: bool,
    pub git: bool,
    pub sync: bool,
    pub lock: bool,
}

impl NewArgs {
    /// wrcli가 파싱한 `CommandContext`에서 `NewArgs`를 조립한다. clap의
    /// `value_enum`/`conflicts_with_all`에 해당하는 검증(템플릿 종류, `--dry-run`과
    /// `--git`/`--sync`/`--lock`의 충돌)을 여기서 수행한다.
    fn from_ctx(ctx: &CommandContext) -> Result<Self, Error> {
        let name = ctx.args.first().cloned().unwrap_or_default();
        let template = TemplateKind::from_str(ctx.flags.get_string("template").unwrap_or(""))?;
        let grpc = ctx.flags.get_bool("grpc").unwrap_or(false);
        let sqlite = ctx.flags.get_bool("sqlite").unwrap_or(false);
        let output = PathBuf::from(ctx.flags.get_string("output").unwrap_or("."));
        let package_name = ctx
            .flags
            .get_string("package-name")
            .map(str::to_owned)
            .filter(|s| !s.is_empty());
        let force = ctx.flags.get_bool("force").unwrap_or(false);
        let verbose = ctx.flags.get_bool("verbose").unwrap_or(false);
        let dry_run = ctx.flags.get_bool("dry-run").unwrap_or(false);
        let git = ctx.flags.get_bool("git").unwrap_or(false);
        let sync = ctx.flags.get_bool("sync").unwrap_or(false);
        let lock = ctx.flags.get_bool("lock").unwrap_or(false);

        if dry_run && (git || sync || lock) {
            return Err(Error::ConflictingFlags);
        }

        Ok(NewArgs {
            name,
            template,
            grpc,
            sqlite,
            output,
            package_name,
            force,
            verbose,
            dry_run,
            git,
            sync,
            lock,
        })
    }
}

/// `--version` 출력: wrcli의 `Panel`로 박스 배너를 그린다. TTY가 아니거나
/// `NO_COLOR`가 설정되면 색상 없이 테두리만 유지된다.
fn print_version() {
    let styled = stdout_is_styled();
    let content = format!(
        "wpygen  v{}\nPython 프로젝트 템플릿(CLI/GUI/SERVER) 생성기",
        env!("CARGO_PKG_VERSION")
    );
    let panel = Panel::new(&content)
        .title("wpygen")
        .border_style(Style::new().fg(Color::Cyan))
        .title_style(Style::new().fg(Color::Cyan).bold());
    print!("{}", panel.render(styled));
}

/// 서브커맨드 없이 실행됐을 때의 간단한 사용법 안내.
fn print_usage() {
    println!("Usage: wpygen [command]");
    println!("       wpygen [flags]");
    println!();
    println!("Run 'wpygen --help' for more information.");
}
