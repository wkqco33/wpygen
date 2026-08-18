use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};

const BINARY: &str = env!("CARGO_BIN_EXE_wpygen");

fn unique_temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    std::env::temp_dir().join(format!("wpygen-generated-{suffix}"))
}

fn run(dir: &Path, program: &str, args: &[&str]) -> ExitStatus {
    Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap_or_else(|error| panic!("failed to run {program}: {error}"))
}

fn python_command() -> &'static str {
    if cfg!(windows) { "python" } else { "python3" }
}

fn generate(root: &Path, template: &str, name: &str, extra_args: &[&str]) -> PathBuf {
    let status = Command::new(BINARY)
        .args(["new", "--template", template, "--output"])
        .arg(root)
        .args(extra_args)
        .arg(name)
        .status()
        .expect("failed to run wpygen");
    assert!(status.success(), "wpygen failed for {template}");
    root.join(name)
}

fn assert_success(status: ExitStatus, description: &str) {
    assert!(status.success(), "{description} failed with {status}");
}

#[test]
fn generated_templates_compile_as_python() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create test directory");

    for template in ["cli", "gui"] {
        let project = generate(&root, template, &format!("demo_{template}"), &[]);
        assert_success(
            run(&project, python_command(), &["-m", "compileall", "-q", "."]),
            &format!("{template} Python compilation"),
        );
    }

    let server = generate(&root, "server", "demo_server", &["--grpc", "--sqlite"]);
    assert_success(
        run(&server, python_command(), &["-m", "compileall", "-q", "."]),
        "server Python compilation",
    );

    fs::remove_dir_all(root).expect("failed to clean up test directory");
}

#[test]
#[ignore = "requires uv and access to PyPI"]
fn generated_templates_pass_uv_checks() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("failed to create test directory");

    let projects = [
        (generate(&root, "cli", "demo_cli", &[]), "demo_cli", false),
        (generate(&root, "gui", "demo_gui", &[]), "demo_gui", false),
        (
            generate(&root, "server", "demo_server", &["--grpc", "--sqlite"]),
            "demo_server",
            true,
        ),
    ];

    for (project, package_name, grpc_enabled) in projects {
        assert_success(run(&project, "uv", &["lock"]), "uv lock");
        assert_success(
            run(&project, "uv", &["sync", "--locked", "--group", "dev"]),
            "uv sync --locked",
        );
        assert_success(
            run(&project, "uv", &["run", "ruff", "check", "."]),
            "ruff check",
        );
        assert_success(run(&project, "uv", &["run", "pytest", "-q"]), "pytest");

        if grpc_enabled {
            assert_success(
                run(&project, "uv", &["run", "python", "tools/generate_grpc.py"]),
                "gRPC code generation",
            );
            let status = Command::new("uv")
                .args([
                    "run",
                    "python",
                    "-c",
                    &format!("import {package_name}.grpc.{package_name}_pb2_grpc"),
                ])
                .env("PYTHONPATH", "src")
                .current_dir(&project)
                .status()
                .expect("failed to run generated gRPC import");
            assert_success(status, "generated gRPC import");
        }
    }

    fs::remove_dir_all(root).expect("failed to clean up test directory");
}
