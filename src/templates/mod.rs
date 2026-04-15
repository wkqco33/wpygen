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
        file("config.toml", common::build_config_toml(spec)),
        file(
            format!("src/{}/__init__.py", spec.package_name),
            "__all__ = []\n".to_string(),
        ),
    ];

    match spec.template {
        TemplateKind::Cli => files.push(file(
            format!("src/{}/main.py", spec.package_name),
            cli::build_main(spec),
        )),
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
        }
    }

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
        assert!(rendered.contains("\"wconfig>=0.1.0\""));
        assert!(rendered.contains("\"wlogger>=0.2.7\""));
        assert!(rendered.contains("\"wpycli>=0.1.1\""));
        assert!(rendered.contains("[[tool.uv.index]]"));
        assert!(rendered.contains("url = \"https://pypi.wkqcosoft.cloud\""));
    }

    #[test]
    fn server_grpc_scaffold_contains_proto_and_codegen() {
        let mut project = spec(TemplateKind::Server, true);
        project.sqlite = true;
        let rendered = render_project(&project);
        assert!(
            rendered
                .iter()
                .any(|file| file.relative_path == PathBuf::from("proto/demo_app.proto"))
        );
        assert!(
            rendered
                .iter()
                .any(|file| file.relative_path == PathBuf::from("tools/generate_grpc.py"))
        );
        assert!(
            rendered
                .iter()
                .any(|file| file.relative_path == PathBuf::from("src/demo_app/database.py"))
        );
    }

    #[test]
    fn gui_template_contains_pyside6_dependency() {
        let rendered = common::build_pyproject(&spec(TemplateKind::Gui, false));
        assert!(rendered.contains("\"PySide6>=6.7.0\""));
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
