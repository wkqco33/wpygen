mod cli;
mod common;
mod grpc;
mod gui;
mod server;
mod sqlite;

use std::path::PathBuf;

use crate::models::{GeneratedFile, ProjectSpec, TemplateKind};

/// 파일 하나의 상대 경로와, 그 내용을 만드는 함수.
struct FilePlan {
    path: PathBuf,
    render: fn(&ProjectSpec) -> String,
}

/// 생성할 파일 목록을 순서까지 포함해 한 곳에서만 정의한다. 이 순서가 `--dry-run`
/// 출력 순서와 실제 쓰기 순서가 되며, `render_project`와 `project_file_paths`가
/// 서로 어긋날 수 없도록 두 함수 모두 이 목록을 쓴다.
fn file_plan(spec: &ProjectSpec) -> Vec<FilePlan> {
    let package_name = &spec.package_name;
    let mut plans = vec![
        plan("pyproject.toml", common::build_pyproject),
        plan("README.md", common::build_readme),
        plan(".gitignore", common::build_gitignore),
        plan(".python-version", |_| "3.12\n".to_string()),
        plan(".env.example", common::build_env_example),
        plan(".env", common::build_env_example),
        plan("config.toml", common::build_config_toml),
        plan(format!("src/{package_name}/__init__.py"), |_| {
            "__all__ = []\n".to_string()
        }),
    ];

    match spec.template {
        TemplateKind::Cli => {
            plans.push(plan(format!("src/{package_name}/main.py"), cli::build_main));
            plans.push(plan("tests/test_smoke.py", cli::build_smoke_test));
        }
        TemplateKind::Gui => push_settings_based_plans(
            &mut plans,
            spec,
            gui::build_settings,
            gui::build_main,
            gui::build_smoke_test,
        ),
        TemplateKind::Server => push_settings_based_plans(
            &mut plans,
            spec,
            server::build_settings,
            server::build_main,
            server::build_smoke_test,
        ),
    }

    plans.push(plan(".github/workflows/ci.yml", |_| {
        common::build_project_ci_workflow()
    }));

    if spec.sqlite {
        plans.push(plan(
            format!("src/{package_name}/database.py"),
            sqlite::build_database_module,
        ));
    }

    if spec.grpc {
        plans.push(plan(
            format!("proto/{package_name}.proto"),
            grpc::build_proto,
        ));
        plans.push(plan(format!("src/{package_name}/grpc/__init__.py"), |_| {
            "__all__ = []\n".to_string()
        }));
        plans.push(plan("tools/generate_grpc.py", grpc::build_codegen_script));
    }

    plans
}

/// 템플릿을 렌더링해 생성할 파일 전체(경로 + 내용)를 돌려준다.
pub fn render_project(spec: &ProjectSpec) -> Vec<GeneratedFile> {
    file_plan(spec)
        .into_iter()
        .map(|entry| GeneratedFile {
            relative_path: entry.path,
            contents: (entry.render)(spec),
        })
        .collect()
}

/// 생성될 파일의 상대 경로만 돌려준다. 내용을 렌더링하지 않으므로 `--dry-run`처럼
/// 경로만 필요한 경우에 쓴다.
pub fn project_file_paths(spec: &ProjectSpec) -> Vec<PathBuf> {
    file_plan(spec)
        .into_iter()
        .map(|entry| entry.path)
        .collect()
}

fn plan(path: impl Into<PathBuf>, render: fn(&ProjectSpec) -> String) -> FilePlan {
    FilePlan {
        path: path.into(),
        render,
    }
}

/// GUI/SERVER 템플릿이 공통으로 만드는 파일과 순서(`settings.py`, `logging.py`,
/// `main.py`, smoke test)를 추가한다. 서로 다른 부분은 각 템플릿의 함수를 넘겨받는다.
fn push_settings_based_plans(
    plans: &mut Vec<FilePlan>,
    spec: &ProjectSpec,
    build_settings: fn(&ProjectSpec) -> String,
    build_main: fn(&ProjectSpec) -> String,
    build_smoke_test: fn(&ProjectSpec) -> String,
) {
    let package_name = &spec.package_name;
    plans.push(plan(
        format!("src/{package_name}/settings.py"),
        build_settings,
    ));
    plans.push(plan(
        format!("src/{package_name}/logging.py"),
        common::build_shared_logging,
    ));
    plans.push(plan(format!("src/{package_name}/main.py"), build_main));
    plans.push(plan("tests/test_smoke.py", build_smoke_test));
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
        }
    }

    #[test]
    fn cli_pyproject_uses_official_pypi_packages() {
        let rendered = common::build_pyproject(&spec(TemplateKind::Cli, false));
        assert!(rendered.contains("\"wpyconf\""));
        assert!(rendered.contains("\"wpylog\""));
        assert!(rendered.contains("\"wpycli\""));
        assert!(!rendered.contains("[[tool.uv.index]]"));
        assert!(!rendered.contains("\"wconfig\""));
        assert!(!rendered.contains("\"wlogger\""));
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
    fn project_file_paths_matches_render_project() {
        for template in [TemplateKind::Cli, TemplateKind::Gui, TemplateKind::Server] {
            for grpc in [false, true] {
                for sqlite in [false, true] {
                    let mut project = spec(template, grpc);
                    project.sqlite = sqlite;
                    let rendered: Vec<PathBuf> = render_project(&project)
                        .into_iter()
                        .map(|file| file.relative_path)
                        .collect();
                    assert_eq!(project_file_paths(&project), rendered);
                }
            }
        }
    }

    #[test]
    fn cli_grpc_sqlite_file_paths_keep_their_order() {
        let mut project = spec(TemplateKind::Cli, true);
        project.sqlite = true;

        assert_eq!(
            project_file_paths(&project),
            vec![
                PathBuf::from("pyproject.toml"),
                PathBuf::from("README.md"),
                PathBuf::from(".gitignore"),
                PathBuf::from(".python-version"),
                PathBuf::from(".env.example"),
                PathBuf::from(".env"),
                PathBuf::from("config.toml"),
                PathBuf::from("src/demo_app/__init__.py"),
                PathBuf::from("src/demo_app/main.py"),
                PathBuf::from("tests/test_smoke.py"),
                PathBuf::from(".github/workflows/ci.yml"),
                PathBuf::from("src/demo_app/database.py"),
                PathBuf::from("proto/demo_app.proto"),
                PathBuf::from("src/demo_app/grpc/__init__.py"),
                PathBuf::from("tools/generate_grpc.py"),
            ]
        );
    }

    #[test]
    fn gui_and_server_settings_share_the_common_scaffold() {
        let gui = gui::build_settings(&spec(TemplateKind::Gui, false));
        let server = server::build_settings(&spec(TemplateKind::Server, false));

        for rendered in [&gui, &server] {
            assert!(rendered.contains("ROOT_DIR = Path(__file__).resolve().parents[2]"));
            assert!(rendered.contains("def _as_bool(value: object) -> bool:"));
            assert!(rendered.contains("env_prefix=\"DEMO_APP\""));
        }

        assert!(gui.contains("    window_title: str\n"));
        assert!(!gui.contains("    host: str\n"));
        assert!(server.contains("    host: str\n"));
        assert!(!server.contains("    window_title: str\n"));
    }

    #[test]
    fn grpc_codegen_rewrites_generated_import_to_relative() {
        let script = grpc::build_codegen_script(&spec(TemplateKind::Server, true));
        assert!(script.contains("PB2_GRPC_FILE"));
        assert!(script.contains("from . import demo_app_pb2 as "));
        assert!(script.contains("generated.replace(absolute_import, relative_import, 1)"));
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
