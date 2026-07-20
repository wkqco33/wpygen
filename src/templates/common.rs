use crate::models::{ProjectSpec, TemplateKind};

pub(crate) fn build_pyproject(spec: &ProjectSpec) -> String {
    let mut dependencies = vec!["\"wconfig\"".to_string(), "\"wlogger\"".to_string()];
    let mut dev_dependencies = vec!["\"ruff\"".to_string(), "\"pytest\"".to_string()];

    match spec.template {
        TemplateKind::Cli => {
            dependencies.push("\"wpycli\"".to_string());
        }
        TemplateKind::Gui => {
            dependencies.push("\"PySide6\"".to_string());
        }
        TemplateKind::Server => {
            dependencies.push("\"fastapi\"".to_string());
            dependencies.push("\"uvicorn\"".to_string());
            dev_dependencies.push("\"httpx\"".to_string());
        }
    }

    if spec.grpc {
        dependencies.push("\"grpcio\"".to_string());
        dev_dependencies.push("\"grpcio-tools\"".to_string());
    }

    let script_name = spec.project_name.replace('_', "-");
    let script_section = format!(
        "[project.scripts]\n{script_name} = \"{}.main:main\"\n",
        spec.package_name
    );
    let dev_group_section = if dev_dependencies.is_empty() {
        String::new()
    } else {
        format!(
            "\n[dependency-groups]\ndev = [{}]\n",
            dev_dependencies.join(", ")
        )
    };

    format!(
        r#"[project]
name = "{project_name}"
version = "0.1.0"
description = "{description}"
readme = "README.md"
requires-python = ">=3.12"
dependencies = [
    {dependencies}
]

{script_section}
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.metadata]
allow-direct-references = true

[tool.uv]
package = true

[[tool.uv.index]]
name = "wkqcosoft"
url = "{index_url}"

[tool.ruff]
target-version = "py312"
line-length = 100

[tool.ruff.lint]
select = ["E4", "E7", "E9", "F", "I"]
{dev_group_section}"#,
        project_name = spec.project_name,
        description = project_description(spec.template),
        dependencies = dependencies.join(",\n    "),
        script_section = script_section.trim_end(),
        index_url = spec.index_url,
        dev_group_section = dev_group_section,
    )
}

pub(crate) fn build_readme(spec: &ProjectSpec) -> String {
    let run_command = match spec.template {
        TemplateKind::Cli => format!(
            "uv run {} hello Copilot",
            spec.project_name.replace('_', "-")
        ),
        TemplateKind::Gui => format!("uv run {}", spec.project_name.replace('_', "-")),
        TemplateKind::Server => format!("uv run {}", spec.project_name.replace('_', "-")),
    };
    let extra_notes = match spec.template {
        TemplateKind::Cli => {
            "- `wpycli` 기반 커맨드 구조와 `wconfig`/`wlogger` 런타임 연동이 포함되어 있습니다.\n"
        }
        TemplateKind::Gui => "- `PySide6` 기반 메인 윈도우 예제가 포함되어 있습니다.\n",
        TemplateKind::Server => {
            "- `FastAPI` 기반 `/health` 엔드포인트와 `uvicorn` 실행 진입점이 포함되어 있습니다.\n"
        }
    };
    let sqlite_notes = if spec.sqlite {
        format!(
            "\n## SQLite\n- 표준 라이브러리 `sqlite3` 기반 초기화 헬퍼가 포함됩니다.\n- 기본 DB 경로: `data/{package}.db`\n",
            package = spec.package_name
        )
    } else {
        String::new()
    };
    let grpc_notes = if spec.grpc {
        format!(
            "\n## gRPC\n- `proto/{package}.proto` 가 생성됩니다.\n- 스텁 생성: `uv sync --group dev && uv run python tools/generate_grpc.py`\n",
            package = spec.package_name
        )
    } else {
        String::new()
    };

    format!(
        "# {name}\n\n{description}\n\n## 시작하기\n1. `uv sync`\n2. `{run_command}`\n\n(`.env` 파일은 기본값으로 이미 생성되어 있습니다. 필요하면 직접 수정하세요.)\n\n## 구성\n- 공통 의존성: `wconfig`, `wlogger`\n{extra_notes}- 사설 인덱스: `{index_url}`\n- 패키지 경로: `src/{package_name}`\n{sqlite_notes}{grpc_notes}\n## 개발\n- 개발 의존성 설치: `uv sync --group dev`\n- 린트: `uv run ruff check .`\n- 테스트: `uv run pytest`\n",
        name = spec.project_name,
        description = readme_description(spec.template),
        run_command = run_command,
        extra_notes = extra_notes,
        index_url = spec.index_url,
        package_name = spec.package_name,
        sqlite_notes = sqlite_notes,
        grpc_notes = grpc_notes,
    )
}

pub(crate) fn build_gitignore(spec: &ProjectSpec) -> String {
    let sqlite_lines = if spec.sqlite { "data/\n*.db\n" } else { "" };

    format!(
        r#".venv/
__pycache__/
*.py[cod]
*.egg-info/
dist/
build/
.coverage
.pytest_cache/
.ruff_cache/
.env
{sqlite_lines}"#
    )
}

pub(crate) fn build_env_example(spec: &ProjectSpec) -> String {
    let prefix = env_prefix(spec);
    let mut body = match spec.template {
        TemplateKind::Cli => {
            format!("{prefix}_APP__DEFAULT_TARGET=world\n{prefix}_LOGGING__LEVEL=INFO\n")
        }
        TemplateKind::Gui => format!(
            "{prefix}_APP__WINDOW_TITLE={title}\n{prefix}_LOGGING__LEVEL=INFO\n",
            title = spec.project_name
        ),
        TemplateKind::Server => format!(
            "{prefix}_APP__HOST=127.0.0.1\n{prefix}_APP__PORT=8000\n{prefix}_LOGGING__LEVEL=INFO\n"
        ),
    };

    if spec.sqlite {
        body.push_str(&format!(
            "{prefix}_SQLITE__PATH=data/{package}.db\n",
            package = spec.package_name
        ));
    }

    body
}

pub(crate) fn build_config_toml(spec: &ProjectSpec) -> String {
    let sqlite_section = if spec.sqlite {
        format!(
            r#"
[sqlite]
enabled = true
path = "data/{package}.db"
"#,
            package = spec.package_name
        )
    } else {
        String::new()
    };

    let base = match spec.template {
        TemplateKind::Cli => format!(
            r#"[app]
name = "{name}"
default_target = "world"
grpc_enabled = {grpc}

[logging]
level = "INFO"
"#,
            name = spec.project_name,
            grpc = toml_bool(spec.grpc),
        ),
        TemplateKind::Gui => format!(
            r#"[app]
name = "{name}"
window_title = "{name}"
grpc_enabled = {grpc}

[logging]
level = "INFO"
"#,
            name = spec.project_name,
            grpc = toml_bool(spec.grpc),
        ),
        TemplateKind::Server => format!(
            r#"[app]
name = "{name}"
debug = false
host = "127.0.0.1"
port = 8000
grpc_enabled = {grpc}

[logging]
level = "INFO"
"#,
            name = spec.project_name,
            grpc = toml_bool(spec.grpc),
        ),
    };

    base + &sqlite_section
}

pub(crate) fn build_project_ci_workflow() -> String {
    r#"name: ci

on:
  push:
  pull_request:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install uv
        uses: astral-sh/setup-uv@v3

      - name: Install dependencies
        run: uv sync --group dev

      - name: Lint
        run: uv run ruff check .

      - name: Test
        run: uv run pytest
"#
    .to_string()
}

pub(crate) fn build_shared_logging(spec: &ProjectSpec) -> String {
    format!(
        r#"from __future__ import annotations

import wlogger

from .settings import AppConfig


def setup_logging(config: AppConfig) -> None:
    wlogger.setup(
        level=config.log_level,
        log_file=config.log_file,
    )


def get_logger(name: str):
    return wlogger.get_logger(name or "{package_name}")
"#,
        package_name = spec.package_name,
    )
}

pub(crate) fn env_prefix(spec: &ProjectSpec) -> String {
    spec.package_name.to_ascii_uppercase()
}

pub(crate) fn toml_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

pub(crate) fn python_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

fn project_description(template: TemplateKind) -> &'static str {
    match template {
        TemplateKind::Cli => "wpycli 기반 CLI 프로젝트 템플릿",
        TemplateKind::Gui => "PySide6 기반 GUI 프로젝트 템플릿",
        TemplateKind::Server => "FastAPI 기반 서버 프로젝트 템플릿",
    }
}

fn readme_description(template: TemplateKind) -> &'static str {
    match template {
        TemplateKind::Cli => "wpycli, wconfig, wlogger 조합으로 시작하는 CLI 프로젝트입니다.",
        TemplateKind::Gui => "wconfig, wlogger, PySide6 조합으로 시작하는 GUI 프로젝트입니다.",
        TemplateKind::Server => "wconfig, wlogger, FastAPI 조합으로 시작하는 서버 프로젝트입니다.",
    }
}
