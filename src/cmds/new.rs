use crate::cli::NewArgs;
use crate::error::Error;
use crate::models::ProjectSpec;
use crate::services::writer;

pub fn run(args: NewArgs) -> Result<(), Error> {
    let package_name = normalize_package_name(args.package_name.as_deref().unwrap_or(&args.name))?;
    let project_name = args.name;
    let output_root = args.output;

    let spec = ProjectSpec {
        project_name: project_name.clone(),
        package_name,
        template: args.template,
        grpc: args.grpc,
        sqlite: args.sqlite,
        index_url: args.index_url,
    };

    let target_dir = output_root.join(&project_name);
    let file_count = writer::create_project(&target_dir, &spec, args.force)?;

    println!(
        "생성 완료: {} (template={}, grpc={}, sqlite={}, files={})",
        target_dir.display(),
        spec.template.as_str(),
        if spec.grpc { "on" } else { "off" },
        if spec.sqlite { "on" } else { "off" },
        file_count
    );
    Ok(())
}

pub fn normalize_package_name(raw: &str) -> Result<String, Error> {
    let normalized = raw
        .trim()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' | '_' => ch,
            'A'..='Z' => ch.to_ascii_lowercase(),
            '-' | ' ' => '_',
            _ => '\0',
        })
        .collect::<String>();

    if normalized.is_empty()
        || normalized.starts_with(|ch: char| ch.is_ascii_digit())
        || normalized.contains('\0')
        || !normalized
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(Error::InvalidPackageName(raw.to_string()));
    }

    Ok(normalized)
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
    fn creates_cli_project_files() {
        let target_dir = unique_temp_dir("cli");
        let spec = ProjectSpec {
            project_name: "demo-app".to_string(),
            package_name: "demo_app".to_string(),
            template: TemplateKind::Cli,
            grpc: true,
            sqlite: true,
            index_url: "https://pypi.wkqcosoft.cloud".to_string(),
        };

        let file_count = writer::create_project(&target_dir, &spec, false).unwrap();

        assert!(file_count >= 7);
        assert!(target_dir.join("pyproject.toml").exists());
        assert!(target_dir.join("README.md").exists());
        assert!(target_dir.join("src/demo_app/main.py").exists());
        assert!(target_dir.join("proto/demo_app.proto").exists());
        assert!(target_dir.join("src/demo_app/database.py").exists());

        let _ = fs::remove_dir_all(target_dir);
    }
}
