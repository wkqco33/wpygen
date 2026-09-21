use crate::models::{ProjectSpec, TemplateKind};

pub(crate) fn build_pyproject(spec: &ProjectSpec) -> String {
    let mut dependencies = vec!["\"wpyconf\"".to_string(), "\"wpylog\"".to_string()];
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
            "- `wpycli` 기반 커맨드 구조와 `wpyconf`/`wpylog` 런타임 연동이 포함되어 있습니다.\n"
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
        "# {name}\n\n{description}\n\n## 시작하기\n1. `uv sync`\n2. `{run_command}`\n\n(`.env` 파일은 기본값으로 이미 생성되어 있습니다. 필요하면 직접 수정하세요.)\n\n## 구성\n- 공통 의존성: `wpyconf`, `wpylog` (import: `wconfig`, `wlogger`)\n{extra_notes}- 패키지 인덱스: 공식 PyPI\n- 패키지 경로: `src/{package_name}`\n{sqlite_notes}{grpc_notes}\n## 개발\n- 개발 의존성 설치: `uv sync --group dev`\n- 린트: `uv run ruff check .`\n- 테스트: `uv run pytest`\n",
        name = spec.project_name,
        description = readme_description(spec.template),
        run_command = run_command,
        extra_notes = extra_notes,
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
        run: |
          if [ -f uv.lock ]; then
            uv sync --locked --group dev
          else
            uv sync --group dev
          fi

      - name: Lint
        run: uv run ruff check .

      - name: Test
        run: uv run pytest
"#
    .to_string()
}

/// GUI/SERVER 템플릿이 공유하는 `settings.py` 본문을 만든다. 두 템플릿의 차이는
/// `AppConfig`에 추가되는 필드와 그 기본값·설정 조회뿐이므로 그 조각들만 인자로 받고,
/// 공통 뼈대(import, `_as_bool`, `load_config` 호출, 로깅/grpc 처리)는 여기서 한 번만
/// 관리한다. 조각들은 `app_name` 다음, 로깅/grpc 앞에 순서대로 들어간다.
pub(crate) fn build_settings_module(
    spec: &ProjectSpec,
    extra_fields: &str,
    extra_defaults: &str,
    extra_arguments: &str,
) -> String {
    let project_name = &spec.project_name;
    let grpc_enabled = python_bool(spec.grpc);
    let sqlite = SqliteSettingsFragments::new(spec);

    format!(
        r#"from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from wconfig import load_config

ROOT_DIR = Path(__file__).resolve().parents[2]


def _as_bool(value: object) -> bool:
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.strip().lower() in {{"1", "true", "t", "yes", "y", "on"}}
    return bool(value)


@dataclass(slots=True)
class AppConfig:
    app_name: str
{extra_fields}    log_level: str
    log_file: str | None
    grpc_enabled: bool
{sqlite_fields}


def load_app_config() -> AppConfig:
    config = load_config(
        defaults={{
            "app": {{
                "name": "{project_name}",
{extra_defaults}                "grpc_enabled": {grpc_enabled},
            }},
            "logging": {{"level": "INFO", "file": None}}{sqlite_defaults},
        }},
        files=(ROOT_DIR / "config.toml",),
        dotenv=ROOT_DIR / ".env",
        env_prefix="{env_prefix}",
    )
    log_file = config.get("logging.file")
    return AppConfig(
        app_name=str(config.get("app.name", "{project_name}")),
{extra_arguments}        log_level=str(config.get("logging.level", "INFO")),
        log_file=str(log_file) if log_file else None,
        grpc_enabled=_as_bool(config.get("app.grpc_enabled", {grpc_enabled})){sqlite_arguments},
    )
"#,
        extra_fields = extra_fields,
        extra_defaults = extra_defaults,
        extra_arguments = extra_arguments,
        project_name = project_name,
        grpc_enabled = grpc_enabled,
        env_prefix = env_prefix(spec),
        sqlite_fields = sqlite.fields,
        sqlite_defaults = sqlite.defaults,
        sqlite_arguments = sqlite.arguments,
    )
}

/// sqlite가 켜졌을 때만 `settings.py`에 덧붙는 조각들.
struct SqliteSettingsFragments {
    fields: String,
    defaults: String,
    arguments: String,
}

impl SqliteSettingsFragments {
    fn new(spec: &ProjectSpec) -> Self {
        if !spec.sqlite {
            return Self {
                fields: String::new(),
                defaults: String::new(),
                arguments: String::new(),
            };
        }

        let package = &spec.package_name;
        Self {
            fields: "    sqlite_enabled: bool\n    sqlite_path: str\n".to_string(),
            defaults: format!(
                ",\n            \"sqlite\": {{\"enabled\": True, \"path\": \"data/{package}.db\"}}"
            ),
            arguments: format!(
                ",\n        sqlite_enabled=_as_bool(config.get(\"sqlite.enabled\", True)),\n        sqlite_path=str(config.get(\"sqlite.path\", \"data/{package}.db\"))"
            ),
        }
    }
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
        TemplateKind::Cli => "wpycli, wpyconf, wpylog 조합으로 시작하는 CLI 프로젝트입니다.",
        TemplateKind::Gui => "wpyconf, wpylog, PySide6 조합으로 시작하는 GUI 프로젝트입니다.",
        TemplateKind::Server => "wpyconf, wpylog, FastAPI 조합으로 시작하는 서버 프로젝트입니다.",
    }
}
