use std::path::{Path, PathBuf};

use wrcli::OutputFormat;
use wrcli::style::pager;

use crate::cmds::new::{normalize_package_name, validate_project_name};
use crate::error::Error;
use crate::models::{AiProvider, GeneratedFile};
use crate::report::AiReport;
use crate::services::ai::{
    AiSpinner, HttpClient, LlmClient, build_system_prompt, build_user_prompt,
    parse_and_validate_plan,
};
use crate::services::process::{self, ChildOutput};
use crate::services::writer;

#[derive(Debug)]
pub struct AiArgs {
    pub prompt: String,
    pub name: String,
    pub provider: AiProvider,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
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

pub fn run(args: AiArgs) -> Result<(), Error> {
    let client = HttpClient::new(
        args.provider,
        args.endpoint.clone(),
        args.model.clone(),
        args.api_key.clone(),
    )?;
    run_with_client(args, &client)
}

pub fn run_with_client<C: LlmClient>(args: AiArgs, client: &C) -> Result<(), Error> {
    let AiArgs {
        prompt,
        name,
        provider,
        model,
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
        ..
    } = args;

    if prompt.trim().is_empty() {
        return Err(Error::EmptyPrompt);
    }

    validate_project_name(&name)?;
    let package_name = normalize_package_name(package_name.as_deref().unwrap_or(&name))?;

    let resolved_model = model.unwrap_or_else(|| provider.default_model().to_string());

    let spinner = if !quiet && format == OutputFormat::Human {
        Some(AiSpinner::start(provider.as_str(), &resolved_model))
    } else {
        None
    };

    let system_prompt = build_system_prompt(&name, &package_name);
    let user_prompt = build_user_prompt(&prompt, &name, &package_name);

    let raw_response = client.complete(&system_prompt, &user_prompt);
    if let Some(s) = spinner {
        s.finish();
    }
    let raw_response = raw_response?;
    let plan = parse_and_validate_plan(&raw_response, &name, &package_name)?;

    let target_dir = output.join(&name);
    let file_paths: Vec<PathBuf> = plan.files.iter().map(|f| f.path.clone()).collect();

    let policy = CommandPolicy {
        child_output: if format == OutputFormat::Human {
            ChildOutput::Inherit
        } else {
            ChildOutput::RedirectStdoutToStderr
        },
        quiet,
    };

    if dry_run {
        let files = writer::preview_from_paths(&target_dir, &file_paths, force)?;
        let report = AiReport {
            project_name: &name,
            package_name: &package_name,
            description: &plan.description,
            provider: provider.as_str(),
            model: &resolved_model,
            target_dir: &target_dir,
            files: &files,
            dry_run: true,
        };
        print_report(&report, format, quiet);
        return Ok(());
    }

    let generated_files: Vec<GeneratedFile> = plan
        .files
        .into_iter()
        .map(|f| GeneratedFile {
            relative_path: f.path,
            contents: f.content,
        })
        .collect();

    let file_count =
        writer::create_project_from_files(&target_dir, &generated_files, force, verbose && !quiet)?;

    assert_eq!(file_count, file_paths.len());

    if git {
        init_git_repository(&target_dir, policy)?;
    }
    if sync {
        uv_sync(&target_dir, lock, policy)?;
    } else if lock {
        uv_lock(&target_dir, policy)?;
    }

    let report = AiReport {
        project_name: &name,
        package_name: &package_name,
        description: &plan.description,
        provider: provider.as_str(),
        model: &resolved_model,
        target_dir: &target_dir,
        files: &file_paths,
        dry_run: false,
    };
    print_report(&report, format, quiet);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct CommandPolicy {
    child_output: ChildOutput,
    quiet: bool,
}

fn print_report(report: &AiReport, format: OutputFormat, quiet: bool) {
    match format {
        OutputFormat::Json => println!("{}", report.to_json()),
        OutputFormat::Plain => {
            let text = report.to_plain();
            if !text.is_empty() {
                println!("{text}");
            }
        }
        OutputFormat::Human => {
            let text = report.to_text();
            if report.dry_run && !quiet {
                let _ = pager::page(&format!("{text}\n"));
            } else {
                println!("{text}");
            }
        }
    }
}

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
        &["commit", "-m", "wpygen ai init"],
        policy.child_output,
    )?;
    status_message(policy, "git 저장소 초기화 및 최초 커밋 완료");
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::MockClient;
    use crate::testing::unique_temp_dir;
    use std::fs;

    fn sample_ai_args(name: &str, output: PathBuf) -> AiArgs {
        AiArgs {
            prompt: "FastAPI payment service".into(),
            name: name.into(),
            provider: AiProvider::Ollama,
            model: None,
            endpoint: None,
            api_key: None,
            output,
            package_name: None,
            force: false,
            verbose: false,
            dry_run: false,
            format: OutputFormat::Human,
            quiet: false,
            git: false,
            sync: false,
            lock: false,
        }
    }

    #[test]
    fn run_with_mock_client_creates_expected_project_files() {
        let temp = unique_temp_dir("ai-create");
        let mock_json = r#"{
            "project_name": "payment-api",
            "package_name": "payment_api",
            "description": "FastAPI service",
            "files": [
                {
                    "path": "pyproject.toml",
                    "content": "[project]\nname = \"payment-api\"\nversion = \"0.1.0\"\nrequires-python = \">=3.12\"\ndependencies = [\"fastapi\"]\n"
                },
                {
                    "path": "src/payment_api/main.py",
                    "content": "from fastapi import FastAPI\napp = FastAPI()\n"
                }
            ]
        }"#;

        let client = MockClient::success(mock_json);
        let args = sample_ai_args("payment-api", temp.clone());

        run_with_client(args, &client).unwrap();

        let target = temp.join("payment-api");
        assert!(target.join("pyproject.toml").exists());
        assert!(target.join("src/payment_api/main.py").exists());
        assert!(target.join(".python-version").exists());

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn run_ai_dry_run_does_not_create_files() {
        let temp = unique_temp_dir("ai-dry-run");
        let mock_json = r#"{
            "files": [
                { "path": "pyproject.toml", "content": "" },
                { "path": "main.py", "content": "" }
            ]
        }"#;

        let client = MockClient::success(mock_json);
        let mut args = sample_ai_args("demo-ai", temp.clone());
        args.dry_run = true;

        run_with_client(args, &client).unwrap();

        assert_eq!(fs::read_dir(&temp).map(|d| d.count()).unwrap_or(0), 0);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn empty_prompt_fails_with_exit_code_2() {
        let temp = unique_temp_dir("ai-empty-prompt");
        let client = MockClient::success("{}");
        let mut args = sample_ai_args("demo-ai", temp.clone());
        args.prompt = "   ".into();

        let err = run_with_client(args, &client).unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(matches!(err, Error::EmptyPrompt));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn invalid_project_name_fails_before_calling_client() {
        let temp = unique_temp_dir("ai-invalid-name");
        let client = MockClient::failure("should not be called");
        let args = sample_ai_args("-invalid-name-", temp.clone());

        let err = run_with_client(args, &client).unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(matches!(err, Error::InvalidProjectName(_)));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn run_ai_uses_resolved_model_in_report() {
        let temp = unique_temp_dir("ai-resolved-model");
        let mock_json = r#"{
            "files": [{ "path": "pyproject.toml", "content": "" }]
        }"#;
        let client = MockClient::success(mock_json);
        let mut args = sample_ai_args("demo-custom", temp.clone());
        args.model = Some("custom-coder".into());
        args.dry_run = true;

        assert!(run_with_client(args, &client).is_ok());
        let _ = fs::remove_dir_all(temp);
    }
}
