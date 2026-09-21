use std::path::{Path, PathBuf};

use crate::cli::NewArgs;
use crate::error::Error;
use crate::models::ProjectSpec;
use crate::report::NewReport;
use crate::services::process::{self, ChildOutput};
use crate::services::writer;
use crate::templates;

pub fn run(args: NewArgs) -> Result<(), Error> {
    let NewArgs {
        name,
        template,
        grpc,
        sqlite,
        output,
        package_name,
        force,
        verbose,
        dry_run,
        json,
        quiet,
        git,
        sync,
        lock,
    } = args;

    validate_project_name(&name)?;
    let package_name = normalize_package_name(package_name.as_deref().unwrap_or(&name))?;

    let spec = ProjectSpec {
        project_name: name.clone(),
        package_name,
        template,
        grpc,
        sqlite,
    };
    let target_dir = output.join(&name);
    let policy = CommandPolicy {
        child_output: if json {
            ChildOutput::RedirectStdoutToStderr
        } else {
            ChildOutput::Inherit
        },
        quiet,
    };

    if dry_run {
        let files = writer::preview(&target_dir, &spec, force)?;
        print_report(&spec, &target_dir, &files, true, json);
        return Ok(());
    }

    let files = templates::project_file_paths(&spec);
    let file_count = writer::create_project(&target_dir, &spec, force, verbose && !quiet)?;
    assert_eq!(
        file_count,
        files.len(),
        "생성한 파일 수와 계획한 파일 수가 다릅니다 (템플릿 목록 불일치)"
    );

    if git {
        init_git_repository(&target_dir, policy)?;
    }
    if sync {
        uv_sync(&target_dir, lock, policy)?;
    } else if lock {
        uv_lock(&target_dir, policy)?;
    }

    print_report(&spec, &target_dir, &files, false, json);
    Ok(())
}

/// 외부 명령 실행과 상태 메시지 출력 방침.
#[derive(Debug, Clone, Copy)]
struct CommandPolicy {
    /// 기계 판독 모드에서는 자식 stdout을 stderr로 돌려 stdout을 JSON만 남긴다.
    child_output: ChildOutput,
    /// 진행·상태 메시지를 생략한다.
    quiet: bool,
}

/// 결과 요약은 stdout으로, 진행·상태 메시지는 stderr로 보낸다.
fn print_report(
    spec: &ProjectSpec,
    target_dir: &Path,
    files: &[PathBuf],
    dry_run: bool,
    json: bool,
) {
    let report = NewReport {
        spec,
        target_dir,
        files,
        dry_run,
    };
    println!(
        "{}",
        if json {
            report.to_json()
        } else {
            report.to_text()
        }
    );
}

/// 결과가 아닌 진행·상태 알림. `--quiet`면 출력하지 않는다.
fn status_message(policy: CommandPolicy, message: &str) {
    if !policy.quiet {
        eprintln!("{message}");
    }
}

fn init_git_repository(target_dir: &Path, policy: CommandPolicy) -> Result<(), Error> {
    process::run(target_dir, "git", &["init"], policy.child_output)?;
    process::run(target_dir, "git", &["add", "-A"], policy.child_output)?;
    process::run(
        target_dir,
        "git",
        &["commit", "-m", "wpygen init"],
        policy.child_output,
    )?;
    status_message(policy, "git 저장소 초기화 및 최초 커밋 완료");
    Ok(())
}

/// `--sync`와 `--lock`이 함께 지정되면 lockfile을 먼저 확정한 뒤 그 lockfile로만
/// 동기화한다(`uv sync --locked`).
fn uv_sync(target_dir: &Path, lock: bool, policy: CommandPolicy) -> Result<(), Error> {
    if lock {
        process::run(target_dir, "uv", &["lock"], policy.child_output)?;
    }
    let sync_args: &[&str] = if lock {
        &["sync", "--locked"]
    } else {
        &["sync"]
    };
    process::run(target_dir, "uv", sync_args, policy.child_output)?;
    status_message(policy, "uv sync 완료");
    Ok(())
}

fn uv_lock(target_dir: &Path, policy: CommandPolicy) -> Result<(), Error> {
    process::run(target_dir, "uv", &["lock"], policy.child_output)?;
    status_message(policy, "uv lock 완료");
    Ok(())
}

pub fn normalize_package_name(raw: &str) -> Result<String, Error> {
    let mut normalized = String::with_capacity(raw.len());

    // 허용되지 않는 문자는 즉시 거부하고, 구분자(`-`, 공백)는 `_`로 통일한다.
    // 연속 구분자가 `__`로 남지 않도록 합치고, 앞뒤 `_`는 만들지 않는다
    // (예: "My  App--CLI-" -> "my_app_cli").
    for ch in raw.trim().chars() {
        let mapped = match ch {
            'a'..='z' | '0'..='9' => ch,
            'A'..='Z' => ch.to_ascii_lowercase(),
            '-' | ' ' | '_' => '_',
            _ => return Err(Error::InvalidPackageName(raw.to_string())),
        };

        if mapped == '_' && (normalized.is_empty() || normalized.ends_with('_')) {
            continue;
        }
        normalized.push(mapped);
    }

    let normalized = normalized.trim_end_matches('_').to_string();
    if normalized.is_empty() || normalized.starts_with(|ch: char| ch.is_ascii_digit()) {
        return Err(Error::InvalidPackageName(raw.to_string()));
    }

    Ok(normalized)
}

/// `project_name`은 그대로 생성된 `pyproject.toml`의 `[project] name`(배포 패키지명)으로
/// 쓰이므로, PEP 508 배포명 규칙(첫/끝 글자는 영문자·숫자, 나머지는 `-`, `_`, `.`도 허용)을
/// 만족하는지 미리 검증한다. 그렇지 않으면 `uv sync` 시점에야 실패하게 된다.
pub fn validate_project_name(name: &str) -> Result<(), Error> {
    let is_alphanumeric = |ch: char| ch.is_ascii_alphanumeric();
    let boundary_ok = matches!(
        (name.chars().next(), name.chars().last()),
        (Some(first), Some(last)) if is_alphanumeric(first) && is_alphanumeric(last)
    );
    let chars_ok = name
        .chars()
        .all(|ch| is_alphanumeric(ch) || matches!(ch, '-' | '_' | '.'));

    if boundary_ok && chars_ok {
        Ok(())
    } else {
        Err(Error::InvalidProjectName(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    use crate::models::TemplateKind;
    use crate::testing::unique_temp_dir;

    fn new_args(name: &str, output: PathBuf) -> NewArgs {
        NewArgs {
            name: name.to_string(),
            template: TemplateKind::Cli,
            grpc: false,
            sqlite: false,
            output,
            package_name: None,
            force: false,
            verbose: false,
            dry_run: false,
            json: false,
            quiet: false,
            git: false,
            sync: false,
            lock: false,
        }
    }

    #[test]
    fn normalizes_package_name() {
        assert_eq!(normalize_package_name("My App-CLI").unwrap(), "my_app_cli");
    }

    #[test]
    fn collapses_consecutive_and_trims_boundary_underscores() {
        assert_eq!(
            normalize_package_name("My  App--CLI-").unwrap(),
            "my_app_cli"
        );
        assert_eq!(normalize_package_name("-demo-app-").unwrap(), "demo_app");
    }

    #[test]
    fn rejects_invalid_package_name() {
        assert!(matches!(
            normalize_package_name("123-app"),
            Err(Error::InvalidPackageName(_))
        ));
        assert!(matches!(
            normalize_package_name("bad.name"),
            Err(Error::InvalidPackageName(_))
        ));
    }

    #[test]
    fn trims_leading_separators_and_rejects_separator_only_names() {
        assert_eq!(normalize_package_name("___demo").unwrap(), "demo");
        assert_eq!(normalize_package_name("  demo  ").unwrap(), "demo");
        assert!(matches!(
            normalize_package_name("--"),
            Err(Error::InvalidPackageName(_))
        ));
        assert!(matches!(
            normalize_package_name("   "),
            Err(Error::InvalidPackageName(_))
        ));
    }

    #[test]
    fn accepts_valid_project_names() {
        assert!(validate_project_name("demo-app").is_ok());
        assert!(validate_project_name("demo_app").is_ok());
        assert!(validate_project_name("demo.app").is_ok());
        assert!(validate_project_name("a").is_ok());
    }

    #[test]
    fn rejects_invalid_project_names() {
        assert!(matches!(
            validate_project_name("demo_app_"),
            Err(Error::InvalidProjectName(_))
        ));
        assert!(matches!(
            validate_project_name("-demo"),
            Err(Error::InvalidProjectName(_))
        ));
        assert!(matches!(
            validate_project_name("demo app"),
            Err(Error::InvalidProjectName(_))
        ));
        assert!(matches!(
            validate_project_name(""),
            Err(Error::InvalidProjectName(_))
        ));
    }

    #[test]
    fn creates_cli_project_files() {
        let target_dir = unique_temp_dir("new-cli");
        let spec = ProjectSpec {
            project_name: "demo-app".to_string(),
            package_name: "demo_app".to_string(),
            template: TemplateKind::Cli,
            grpc: true,
            sqlite: true,
        };

        let file_count = writer::create_project(&target_dir, &spec, false, false).unwrap();

        assert!(file_count >= 9);
        assert!(target_dir.join("pyproject.toml").exists());
        assert!(target_dir.join("README.md").exists());
        assert!(target_dir.join("src/demo_app/main.py").exists());
        assert!(target_dir.join("proto/demo_app.proto").exists());
        assert!(target_dir.join("src/demo_app/database.py").exists());
        assert!(target_dir.join("tests/test_smoke.py").exists());
        assert!(target_dir.join(".github/workflows/ci.yml").exists());

        let _ = fs::remove_dir_all(target_dir);
    }

    #[test]
    fn dry_run_generates_no_files() {
        let root = unique_temp_dir("new-dry-run");
        fs::create_dir_all(&root).unwrap();

        let mut args = new_args("demo-app", root.clone());
        args.dry_run = true;
        args.grpc = true;
        args.sqlite = true;

        run(args).unwrap();

        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_project_name_fails_before_touching_the_file_system() {
        let root = unique_temp_dir("new-invalid-name");
        fs::create_dir_all(&root).unwrap();

        let error = run(new_args("bad name", root.clone())).unwrap_err();

        assert_eq!(error.exit_code(), 2);
        assert!(matches!(error, Error::InvalidProjectName(_)));
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        let _ = fs::remove_dir_all(root);
    }
}
