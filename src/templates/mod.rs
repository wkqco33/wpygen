mod cli;
mod common;
mod grpc;
mod gui;
mod server;
mod sqlite;

use std::path::PathBuf;

use crate::models::{GeneratedFile, ProjectSpec, TemplateKind};

pub fn render_project(spec: &ProjectSpec) -> Vec<GeneratedFile> {
    let mut files = vec![
        file("pyproject.toml", common::build_pyproject(spec)),
        file("README.md", common::build_readme(spec)),
        file(".gitignore", common::build_gitignore(spec)),
        file(".python-version", "3.12\n".to_string()),
        file(".env.example", common::build_env_example(spec)),
        file(".env", common::build_env_example(spec)),
        file("config.toml", common::build_config_toml(spec)),
        file(
            format!("src/{}/__init__.py", spec.package_name),
            "__all__ = []\n".to_string(),
        ),
    ];

    let smoke_test = match spec.template {
        TemplateKind::Cli => {
            files.push(file(
                format!("src/{}/main.py", spec.package_name),
                cli::build_main(spec),
            ));
            cli::build_smoke_test(spec)
        }
        TemplateKind::Gui => {
            files.push(file(
                format!("src/{}/settings.py", spec.package_name),
                gui::build_settings(spec),
            ));
            files.push(file(
                format!("src/{}/logging.py", spec.package_name),
                common::build_shared_logging(spec),
            ));
            files.push(file(
                format!("src/{}/main.py", spec.package_name),
                gui::build_main(spec),
            ));
            gui::build_smoke_test(spec)
        }
        TemplateKind::Server => {
            files.push(file(
                format!("src/{}/settings.py", spec.package_name),
                server::build_settings(spec),
            ));
            files.push(file(
                format!("src/{}/logging.py", spec.package_name),
                common::build_shared_logging(spec),
            ));
            files.push(file(
                format!("src/{}/main.py", spec.package_name),
                server::build_main(spec),
            ));
            server::build_smoke_test(spec)
        }
    };
    files.push(file("tests/test_smoke.py", smoke_test));
    files.push(file(
        ".github/workflows/ci.yml",
        common::build_project_ci_workflow(),
    ));

    if spec.sqlite {
        files.push(file(
            format!("src/{}/database.py", spec.package_name),
            sqlite::build_database_module(spec),
        ));
    }

    if spec.grpc {
        files.push(file(
            format!("proto/{}.proto", spec.package_name),
            grpc::build_proto(spec),
        ));
        files.push(file(
            format!("src/{}/grpc/__init__.py", spec.package_name),
            "__all__ = []\n".to_string(),
        ));
        files.push(file(
            "tools/generate_grpc.py",
            grpc::build_codegen_script(spec),
        ));
    }

    files
}

fn file(path: impl Into<PathBuf>, contents: String) -> GeneratedFile {
    GeneratedFile {
        relative_path: path.into(),
        contents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn spec(template: TemplateKind, grpc: bool) -> ProjectSpec {
        ProjectSpec {
            project_name: "demo-app".to_string(),
            package_name: "demo_app".to_string(),
            template,
            grpc,
            sqlite: false,
            index_url: "https://pypi.wkqcosoft.cloud".to_string(),
        }
    }

    #[test]
    fn cli_pyproject_uses_private_packages() {
        let rendered = common::build_pyproject(&spec(TemplateKind::Cli, false));
        assert!(rendered.contains("\"wconfig\""));
        assert!(rendered.contains("\"wlogger\""));
        assert!(rendered.contains("\"wpycli\""));
        assert!(rendered.contains("[[tool.uv.index]]"));
        assert!(rendered.contains("url = \"https://pypi.wkqcosoft.cloud\""));
    }

    #[test]
    fn pyproject_always_includes_ruff_and_pytest_dev_deps() {
        let cli = common::build_pyproject(&spec(TemplateKind::Cli, false));
        assert!(cli.contains("[tool.ruff]"));
        assert!(cli.contains("[dependency-groups]"));
        assert!(cli.contains("\"ruff\""));
        assert!(cli.contains("\"pytest\""));

        let server = common::build_pyproject(&spec(TemplateKind::Server, false));
        assert!(server.contains("\"httpx\""));
    }

    #[test]
    fn render_project_always_includes_smoke_test_and_ci_workflow() {
        for template in [TemplateKind::Cli, TemplateKind::Gui, TemplateKind::Server] {
            let rendered = render_project(&spec(template, false));
            assert!(
                rendered
                    .iter()
                    .any(|file| file.relative_path == Path::new("tests/test_smoke.py")),
                "{template:?} missing tests/test_smoke.py"
            );
            assert!(
                rendered
                    .iter()
                    .any(|file| file.relative_path == Path::new(".github/workflows/ci.yml")),
                "{template:?} missing .github/workflows/ci.yml"
            );
        }
    }

    #[test]
    fn server_grpc_scaffold_contains_proto_and_codegen() {
        let mut project = spec(TemplateKind::Server, true);
        project.sqlite = true;
        let rendered = render_project(&project);
        assert!(
            rendered
                .iter()
                .any(|file| file.relative_path == Path::new("proto/demo_app.proto"))
        );
        assert!(
            rendered
                .iter()
                .any(|file| file.relative_path == Path::new("tools/generate_grpc.py"))
        );
        assert!(
            rendered
                .iter()
                .any(|file| file.relative_path == Path::new("src/demo_app/database.py"))
        );
    }

    #[test]
    fn gui_template_contains_pyside6_dependency() {
        let rendered = common::build_pyproject(&spec(TemplateKind::Gui, false));
        assert!(rendered.contains("\"PySide6\""));
    }

    #[test]
    fn sqlite_config_and_env_are_rendered_when_enabled() {
        let mut project = spec(TemplateKind::Cli, false);
        project.sqlite = true;

        let config = common::build_config_toml(&project);
        let env_example = common::build_env_example(&project);
        let readme = common::build_readme(&project);
        let cli_main = cli::build_main(&project);

        assert!(config.contains("[sqlite]"));
        assert!(config.contains("path = \"data/demo_app.db\""));
        assert!(env_example.contains("DEMO_APP_SQLITE__PATH=data/demo_app.db"));
        assert!(readme.contains("## SQLite"));
        assert!(cli_main.contains("run_db_init"));

        let database_module = sqlite::build_database_module(&project);
        assert!(database_module.contains("with closing(sqlite3.connect(db_path)) as connection:"));
        assert!(!database_module.contains("connection.commit()"));
    }

    #[test]
    fn python_templates_use_python_bools() {
        let cli = cli::build_main(&spec(TemplateKind::Cli, true));
        let gui = gui::build_settings(&spec(TemplateKind::Gui, true));
        let server = server::build_settings(&spec(TemplateKind::Server, true));

        assert!(cli.contains("\"grpc_enabled\": True"));
        assert!(gui.contains("\"grpc_enabled\": True"));
        assert!(server.contains("\"grpc_enabled\": True"));
        assert!(!cli.contains("\"grpc_enabled\": true"));
    }
}
