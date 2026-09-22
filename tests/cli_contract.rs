//! CLI 계약을 실제 바이너리로 검증한다. 표준 출력/에러 분리, 기계 판독 출력,
//! 종료 코드, 도움말 내용이 대상이다.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const BINARY: &str = env!("CARGO_BIN_EXE_wpygen");

fn unique_temp_dir() -> PathBuf {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    std::env::temp_dir().join(format!(
        "wpygen-cli-contract-{}-{nanos}-{sequence}",
        std::process::id()
    ))
}

fn run_wpygen(args: &[&str]) -> Output {
    Command::new(BINARY)
        .args(args)
        .output()
        .expect("failed to run wpygen")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// `git commit`이 사용자 전역 설정에 의존하지 않도록 최소 신원을 주입한다.
fn run_wpygen_with_git_identity(args: &[&str]) -> Output {
    Command::new(BINARY)
        .args(args)
        .env("GIT_AUTHOR_NAME", "wpygen test")
        .env("GIT_AUTHOR_EMAIL", "wpygen@example.com")
        .env("GIT_COMMITTER_NAME", "wpygen test")
        .env("GIT_COMMITTER_EMAIL", "wpygen@example.com")
        .output()
        .expect("failed to run wpygen")
}

#[test]
fn json_dry_run_prints_only_json_to_stdout() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create temp dir");
    let root_arg = root.to_str().expect("temp path is not utf-8");

    let output = run_wpygen(&[
        "new",
        "-t",
        "cli",
        "--dry-run",
        "--json",
        "-o",
        root_arg,
        "demo_app",
    ]);

    assert!(output.status.success(), "{}", stderr(&output));
    let out = stdout(&output);
    assert!(out.trim_start().starts_with('{'), "{out}");
    assert!(out.trim_end().ends_with('}'), "{out}");
    assert!(out.contains("\"dry_run\": true"), "{out}");
    assert!(out.contains("\"project_name\": \"demo_app\""), "{out}");
    assert!(out.contains("\"template\": \"cli\""), "{out}");
    assert!(out.contains("\"pyproject.toml\""), "{out}");
    assert!(
        !out.contains("dry-run:"),
        "사람용 문구가 stdout에 섞였다: {out}"
    );
    assert!(!root.join("demo_app").exists(), "dry-run이 파일을 만들었다");
    assert_eq!(stderr(&output), "");

    fs::remove_dir_all(root).expect("failed to clean up");
}

#[test]
fn json_mode_keeps_child_process_output_off_stdout() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create temp dir");
    let root_arg = root.to_str().expect("temp path is not utf-8");

    let output = run_wpygen_with_git_identity(&[
        "new", "-t", "cli", "--json", "--git", "-o", root_arg, "demo_git",
    ]);

    assert!(output.status.success(), "{}", stderr(&output));
    let out = stdout(&output);
    assert!(out.trim_start().starts_with('{'), "{out}");
    assert!(out.trim_end().ends_with('}'), "{out}");
    assert!(out.contains("\"dry_run\": false"), "{out}");
    assert!(
        !out.contains("Initialized empty Git repository"),
        "git 출력이 stdout을 오염시켰다: {out}"
    );
    assert!(root.join("demo_git").join(".git").exists());

    fs::remove_dir_all(root).expect("failed to clean up");
}

#[test]
fn quiet_suppresses_status_messages_but_keeps_the_result() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create temp dir");
    let root_arg = root.to_str().expect("temp path is not utf-8");

    let output = run_wpygen_with_git_identity(&[
        "new",
        "-t",
        "cli",
        "--quiet",
        "--git",
        "-o",
        root_arg,
        "demo_quiet",
    ]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("생성 완료: "));
    // `--quiet`는 wpygen 자신의 상태 메시지만 생략한다. 자식 프로세스(git/uv)의 출력은
    // 그대로 전달되므로 stderr가 비어 있다고 단정하지 않는다.
    assert!(
        !stderr(&output).contains("git 저장소 초기화 및 최초 커밋 완료"),
        "quiet인데 wpygen 상태 메시지가 남았다: {}",
        stderr(&output)
    );

    fs::remove_dir_all(root).expect("failed to clean up");
}

#[test]
fn dry_run_short_flag_works() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create temp dir");
    let root_arg = root.to_str().expect("temp path is not utf-8");

    let output = run_wpygen(&["new", "-t", "gui", "-n", "-o", root_arg, "demo_short"]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("dry-run: "));
    assert!(!root.join("demo_short").exists());

    fs::remove_dir_all(root).expect("failed to clean up");
}

#[test]
fn user_input_errors_exit_with_code_2() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create temp dir");
    let root_arg = root.to_str().expect("temp path is not utf-8");

    let invalid_template = run_wpygen(&["new", "-t", "bogus", "-o", root_arg, "demo_bad"]);
    assert_eq!(invalid_template.status.code(), Some(2));
    assert!(stderr(&invalid_template).contains("유효하지 않은 템플릿"));

    let invalid_name = run_wpygen(&["new", "-t", "cli", "-o", root_arg, "bad name"]);
    assert_eq!(invalid_name.status.code(), Some(2));

    let unknown_flag = run_wpygen(&["new", "-t", "cli", "--nope", "demo_bad"]);
    assert_eq!(unknown_flag.status.code(), Some(2));

    let conflicting_output_flags =
        run_wpygen(&["new", "-t", "cli", "--dry-run", "--git", "demo_bad"]);
    assert_eq!(conflicting_output_flags.status.code(), Some(2));

    fs::remove_dir_all(root).expect("failed to clean up");
}

#[cfg(unix)]
#[test]
fn missing_external_command_exits_with_code_1_and_names_the_command() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create temp dir");
    let root_arg = root.to_str().expect("temp path is not utf-8");

    // PATH를 비워 `git` 실행 자체가 실패하게 만든다.
    let output = Command::new(BINARY)
        .args(["new", "-t", "cli", "--git", "-o", root_arg, "demo_nogit"])
        .env("PATH", "")
        .output()
        .expect("failed to run wpygen");

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("명령을 실행할 수 없습니다: git"),
        "{}",
        stderr(&output)
    );

    fs::remove_dir_all(root).expect("failed to clean up");
}

#[test]
fn subcommand_help_documents_usage_examples_and_links() {
    let output = run_wpygen(&["new", "--help"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("Examples:"), "{out}");
    assert!(out.contains("wpygen new -t cli my_app"), "{out}");
    assert!(out.contains("<NAME>"), "{out}");
    assert!(out.contains("Documentation:"), "{out}");
    assert!(
        out.contains("https://github.com/wkqco33/wpygen#readme"),
        "{out}"
    );
    assert!(out.contains("Support:"), "{out}");
    assert!(
        out.contains("https://github.com/wkqco33/wpygen/issues"),
        "{out}"
    );
}

#[test]
fn builtin_help_subcommand_prints_subcommand_help() {
    let output = run_wpygen(&["help", "new"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("wpygen new [flags]"), "{out}");
    assert!(out.contains("-t, --template"), "{out}");
}

#[test]
fn plain_output_lists_file_paths_one_per_line() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create temp dir");
    let root_arg = root.to_str().expect("temp path is not utf-8");

    let output = run_wpygen(&[
        "new",
        "-t",
        "cli",
        "-n",
        "--plain",
        "-o",
        root_arg,
        "demo_plain",
    ]);

    assert!(output.status.success(), "{}", stderr(&output));
    let out = stdout(&output);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 9, "파일 수가 너무 적다: {out}");
    assert!(
        lines.iter().all(|line| line.starts_with(root_arg)),
        "모든 줄이 대상 경로 아래여야 한다: {out}"
    );
    assert!(out.contains("pyproject.toml"), "{out}");
    assert!(!out.contains("dry-run:"), "{out}");
    assert!(!out.contains("생성 완료"), "{out}");

    fs::remove_dir_all(root).expect("failed to clean up");
}

#[test]
fn plain_and_json_are_mutually_exclusive() {
    let output = run_wpygen(&["new", "-t", "cli", "--plain", "--json", "demo_conflict"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
}

#[test]
fn color_flags_control_ansi_output() {
    let forced = run_wpygen(&["--color", "always", "--version"]);
    assert!(forced.status.success(), "{}", stderr(&forced));
    assert!(
        stdout(&forced).contains('\u{1b}'),
        "--color always 인데 ANSI 코드가 없다"
    );

    let disabled = run_wpygen(&["--no-color", "--version"]);
    assert!(disabled.status.success(), "{}", stderr(&disabled));
    assert!(
        !stdout(&disabled).contains('\u{1b}'),
        "--no-color 인데 ANSI 코드가 있다"
    );

    let default = run_wpygen(&["--version"]);
    assert!(
        !stdout(&default).contains('\u{1b}'),
        "파이프 출력은 무색이어야 한다"
    );
}

#[test]
fn completions_print_to_stdout() {
    let output = run_wpygen(&["completions", "bash"]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("wpygen"));
}

#[test]
fn ai_subcommand_help_documents_flags_and_examples() {
    let output = run_wpygen(&["ai", "--help"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("wpygen ai [flags]"), "{out}");
    assert!(out.contains("--name"), "{out}");
    assert!(out.contains("--provider"), "{out}");
    assert!(out.contains("--model"), "{out}");
    assert!(out.contains("Examples:"), "{out}");
    assert!(out.contains("wpygen ai --name"), "{out}");
}

#[test]
fn ai_requires_name_flag() {
    let output = run_wpygen(&["ai", "fastapi service"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("name") || stderr(&output).contains("required"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn ai_invalid_provider_exits_with_code_2() {
    let output = run_wpygen(&["ai", "--name", "my_app", "--provider", "invalid_p", "test"]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("지원하지 않는 AI 프로바이더"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn ai_conflicting_flags_exits_with_code_2() {
    let output = run_wpygen(&[
        "ai",
        "--name",
        "my_app",
        "--dry-run",
        "--git",
        "test prompt",
    ]);

    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(stderr(&output).contains("--dry-run"), "{}", stderr(&output));
}
