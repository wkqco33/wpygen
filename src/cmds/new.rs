use std::path::Path;

use crate::cli::NewArgs;
use crate::error::Error;
use crate::models::ProjectSpec;
use crate::services::{process, writer};

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

    if dry_run {
        return print_dry_run(&target_dir, &spec, force);
    }

    let file_count = writer::create_project(&target_dir, &spec, force, verbose)?;

    println!(
        "생성 완료: {} (template={}, grpc={}, sqlite={}, files={})",
        target_dir.display(),
        spec.template.as_str(),
        on_off(spec.grpc),
        on_off(spec.sqlite),
        file_count
    );

    if git {
        init_git_repository(&target_dir)?;
    }
    if sync {
        uv_sync(&target_dir, lock)?;
    } else if lock {
        uv_lock(&target_dir)?;
    }

    Ok(())
}

fn print_dry_run(target_dir: &Path, spec: &ProjectSpec, force: bool) -> Result<(), Error> {
    let files = writer::preview(target_dir, spec, force)?;
    println!("dry-run: {} 아래 생성될 파일 목록", target_dir.display());
    for path in &files {
        println!("  {}", path.display());
    }
    println!("(dry-run 모드이므로 실제 파일은 생성되지 않았습니다)");
    Ok(())
}

fn init_git_repository(target_dir: &Path) -> Result<(), Error> {
    process::run(target_dir, "git", &["init"])?;
    process::run(target_dir, "git", &["add", "-A"])?;
    process::run(target_dir, "git", &["commit", "-m", "wpygen init"])?;
    println!("git 저장소 초기화 및 최초 커밋 완료");
    Ok(())
}

/// `--sync`와 `--lock`이 함께 지정되면 lockfile을 먼저 확정한 뒤 그 lockfile로만
/// 동기화한다(`uv sync --locked`).
fn uv_sync(target_dir: &Path, lock: bool) -> Result<(), Error> {
    if lock {
        process::run(target_dir, "uv", &["lock"])?;
    }
    let sync_args: &[&str] = if lock {
        &["sync", "--locked"]
    } else {
        &["sync"]
    };
    process::run(target_dir, "uv", sync_args)?;
    println!("uv sync 완료");
    Ok(())
}

fn uv_lock(target_dir: &Path) -> Result<(), Error> {
    process::run(target_dir, "uv", &["lock"])?;
    println!("uv lock 완료");
    Ok(())
}

fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
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
