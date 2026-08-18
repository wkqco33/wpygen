use crate::cli::NewArgs;
use crate::error::Error;
use crate::models::ProjectSpec;
use crate::services::{process, writer};

pub fn run(args: NewArgs) -> Result<(), Error> {
    validate_project_name(&args.name)?;
    let package_name = normalize_package_name(args.package_name.as_deref().unwrap_or(&args.name))?;
    let project_name = args.name;
    let output_root = args.output;

    let spec = ProjectSpec {
        project_name: project_name.clone(),
        package_name,
        template: args.template,
        grpc: args.grpc,
        sqlite: args.sqlite,
    };

    let target_dir = output_root.join(&project_name);

    if args.dry_run {
        let files = writer::preview(&target_dir, &spec, args.force)?;
        println!("dry-run: {} 아래 생성될 파일 목록", target_dir.display());
        for path in &files {
            println!("  {}", path.display());
        }
        println!("(dry-run 모드이므로 실제 파일은 생성되지 않았습니다)");
        return Ok(());
    }

    let file_count = writer::create_project(&target_dir, &spec, args.force, args.verbose)?;

    println!(
        "생성 완료: {} (template={}, grpc={}, sqlite={}, files={})",
        target_dir.display(),
        spec.template.as_str(),
        if spec.grpc { "on" } else { "off" },
        if spec.sqlite { "on" } else { "off" },
        file_count
    );

    if args.git {
        process::run(&target_dir, "git", &["init"])?;
        process::run(&target_dir, "git", &["add", "-A"])?;
        process::run(&target_dir, "git", &["commit", "-m", "wpygen init"])?;
        println!("git 저장소 초기화 및 최초 커밋 완료");
    }

    if args.sync {
        if args.lock {
            process::run(&target_dir, "uv", &["lock"])?;
        }
        let sync_args: &[&str] = if args.lock {
            &["sync", "--locked"]
        } else {
            &["sync"]
        };
        process::run(&target_dir, "uv", sync_args)?;
        println!("uv sync 완료");
    } else if args.lock {
        process::run(&target_dir, "uv", &["lock"])?;
        println!("uv lock 완료");
    }

    Ok(())
}

pub fn normalize_package_name(raw: &str) -> Result<String, Error> {
    let mapped = raw
        .trim()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' | '_' => ch,
            'A'..='Z' => ch.to_ascii_lowercase(),
            '-' | ' ' => '_',
            _ => '\0',
        })
        .collect::<String>();

    if mapped.contains('\0') {
        return Err(Error::InvalidPackageName(raw.to_string()));
    }

    // 연속된 `-`/공백이 `_`로 겹쳐 치환되면서 `__`가 생기지 않도록 인접 언더바를 합치고,
    // 앞뒤 언더바는 잘라낸다 (예: "my--app-" -> "my_app").
    let mut normalized = String::with_capacity(mapped.len());
    for ch in mapped.chars() {
        if ch == '_' && normalized.ends_with('_') {
            continue;
        }
        normalized.push(ch);
    }
    let normalized = normalized.trim_matches('_').to_string();

    if normalized.is_empty() || normalized.starts_with(|ch: char| ch.is_ascii_digit()) {
        return Err(Error::InvalidPackageName(raw.to_string()));
    }

    Ok(normalized)
}

/// `project_name`은 그대로 생성된 `pyproject.toml`의 `[project] name`(배포 패키지명)으로
/// 쓰이므로, PEP 508 배포명 규칙(첫/끝 글자는 영문자·숫자, 나머지는 `-`, `_`, `.`도 허용)을
/// 만족하는지 미리 검증한다. 그렇지 않으면 `uv sync` 시점에야 실패하게 된다.
pub fn validate_project_name(name: &str) -> Result<(), Error> {
    let boundary_ok = name
        .chars()
        .next()
        .zip(name.chars().last())
        .is_some_and(|(first, last)| first.is_ascii_alphanumeric() && last.is_ascii_alphanumeric());
    let chars_ok = name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));

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
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::models::TemplateKind;

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("wpygen-{name}-{suffix}"))
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
        let target_dir = unique_temp_dir("cli");
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
}
