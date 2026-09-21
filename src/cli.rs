use std::path::PathBuf;

use wrcli::args::exact_args;
use wrcli::style::{Color, Panel, Style, stdout_is_styled};
use wrcli::{Command, CommandContext, Flag, FlagValue, OutputFormat, WrCliError};

use crate::cmds;
use crate::error::Error;
use crate::models::TemplateKind;

const DOCS_URL: &str = "https://github.com/wkqco33/wpygen#readme";
const ISSUES_URL: &str = "https://github.com/wkqco33/wpygen/issues";

/// `wpygen` CLI 트리를 빌드한다.
///
/// clig.dev 표준 플래그 묶음(`standard_flags`)을 루트에 등록해 `-q/--quiet`,
/// `-f/--force`, `--json`, `--plain`, `--no-color`, `--color <when>`, `--no-input`,
/// `--confirm`이 모든 서브커맨드에 전파되게 한다. 출력 규칙은 결과=stdout,
/// 진행·상태·오류=stderr이며, `--json`/`--plain`은 stdout을 기계 판독 출력 전용으로
/// 쓴다.
pub fn build_cli() -> Command {
    with_standard_flags(
        Command::new("wpygen")
            .short("Python 프로젝트 템플릿(CLI/GUI/SERVER) 생성기")
            .long("uv 기반 Python 프로젝트 템플릿(CLI/GUI/SERVER)을 생성합니다.")
            .docs_url(DOCS_URL)
            .support_url(ISSUES_URL)
            .example("wpygen new -t cli my_app")
            .example("wpygen completions bash")
            .flag(Flag::new("version", FlagValue::Bool(false), "버전 정보를 출력한다.").short('V'))
            .on_run(|ctx| {
                if ctx.flags.get_bool("version").unwrap_or(false) {
                    print_version();
                } else {
                    print_usage();
                }
            }),
    )
    .subcommand(
        Command::new("new")
            .short("새 Python 프로젝트 템플릿을 생성한다.")
            .long(
                "지정한 이름으로 프로젝트 디렉터리를 만들고 템플릿 파일을 생성합니다.\n\
                     인자: <NAME> 프로젝트(디렉터리) 이름",
            )
            .example("wpygen new -t cli my_app")
            .example("wpygen new -t server --grpc --sqlite my_service")
            .example("wpygen new -t gui --dry-run my_app")
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
            .flag(
                Flag::new(
                    "verbose",
                    FlagValue::Bool(false),
                    "생성 진행 상황을 상세히 출력한다.",
                )
                .short('v'),
            )
            .flag(
                Flag::new(
                    "dry-run",
                    FlagValue::Bool(false),
                    "실제로 파일을 쓰지 않고, 생성될 파일 경로만 출력한다.",
                )
                .short('n'),
            )
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
            .example("wpygen completions bash")
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
    pub format: OutputFormat,
    pub quiet: bool,
    pub git: bool,
    pub sync: bool,
    pub lock: bool,
}

impl NewArgs {
    /// wrcli가 파싱한 `CommandContext`에서 `NewArgs`를 조립한다. 표준 플래그
    /// (`--force`, `--json`, `--plain`, `--quiet`)는 컨텍스트 접근자로 읽고,
    /// 템플릿 종류와 `--dry-run` 충돌 검증만 여기서 수행한다.
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
        let force = ctx.is_force();
        let verbose = ctx.flags.get_bool("verbose").unwrap_or(false);
        let dry_run = ctx.flags.get_bool("dry-run").unwrap_or(false);
        let format = ctx.output_format();
        let quiet = ctx.is_quiet();
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
            format,
            quiet,
            git,
            sync,
            lock,
        })
    }
}

/// clig.dev 표준 플래그를 루트 커맨드에 persistent로 등록한다. 라이브러리의
/// `standard_flags()` 번들은 설명이 영어라 직접 등록하고, wpygen에 의미가 없는
/// `--confirm`은 넣지 않는다. `--plain`과 `--json`은 함께 쓸 수 없다.
fn with_standard_flags(command: Command) -> Command {
    command
        .persistent_flag(
            Flag::new(
                "quiet",
                FlagValue::Bool(false),
                "진행·상태 메시지를 출력하지 않는다.",
            )
            .short('q'),
        )
        .persistent_flag(
            Flag::new(
                "force",
                FlagValue::Bool(false),
                "대상 디렉터리가 비어있지 않아도 덮어쓴다.",
            )
            .short('f'),
        )
        .persistent_flag(Flag::new(
            "no-input",
            FlagValue::Bool(false),
            "입력을 요구하는 프롬프트를 쓰지 않는다.",
        ))
        .persistent_flag(Flag::new(
            "no-color",
            FlagValue::Bool(false),
            "색상 출력을 끈다.",
        ))
        .persistent_flag(Flag::new(
            "color",
            FlagValue::String("auto".to_owned()),
            "색상 사용 시점 (auto|always|never)",
        ))
        .persistent_flag(Flag::new(
            "plain",
            FlagValue::Bool(false),
            "파일 경로를 한 줄에 하나씩 출력한다(기계 판독용).",
        ))
        .persistent_flag(Flag::new(
            "json",
            FlagValue::Bool(false),
            "결과를 JSON으로 출력한다.",
        ))
        .mutually_exclusive(&["plain", "json"])
}

/// `--version` 출력: wrcli의 `Panel`로 박스 배너를 그린다. TTY가 아니거나
/// `NO_COLOR`/`--no-color`가 적용되면 색상 없이 테두리만 유지된다.
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
