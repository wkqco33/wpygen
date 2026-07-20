use crate::models::ProjectSpec;

use super::common;

pub(crate) fn build_settings(spec: &ProjectSpec) -> String {
    let sqlite_fields = if spec.sqlite {
        "    sqlite_enabled: bool\n    sqlite_path: str\n"
    } else {
        ""
    };
    let sqlite_defaults = if spec.sqlite {
        format!(
            ",\n            \"sqlite\": {{\"enabled\": True, \"path\": \"data/{package}.db\"}}",
            package = spec.package_name
        )
    } else {
        String::new()
    };
    let sqlite_return = if spec.sqlite {
        format!(
            ",\n        sqlite_enabled=_as_bool(config.get(\"sqlite.enabled\", True)),\n        sqlite_path=str(config.get(\"sqlite.path\", \"data/{package}.db\"))",
            package = spec.package_name
        )
    } else {
        String::new()
    };

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
    debug: bool
    host: str
    port: int
    log_level: str
    log_file: str | None
    grpc_enabled: bool
{sqlite_fields}


def load_app_config() -> AppConfig:
    config = load_config(
        defaults={{
            "app": {{
                "name": "{project_name}",
                "debug": False,
                "host": "127.0.0.1",
                "port": 8000,
                "grpc_enabled": {grpc_enabled},
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
        debug=_as_bool(config.get("app.debug", False)),
        host=str(config.get("app.host", "127.0.0.1")),
        port=int(config.get("app.port", 8000)),
        log_level=str(config.get("logging.level", "INFO")),
        log_file=str(log_file) if log_file else None,
        grpc_enabled=_as_bool(config.get("app.grpc_enabled", {grpc_enabled})){sqlite_return},
    )
"#,
        project_name = spec.project_name,
        grpc_enabled = common::python_bool(spec.grpc),
        env_prefix = common::env_prefix(spec),
        sqlite_fields = sqlite_fields,
        sqlite_defaults = sqlite_defaults,
        sqlite_return = sqlite_return,
    )
}

pub(crate) fn build_main(spec: &ProjectSpec) -> String {
    let sqlite_import = if spec.sqlite {
        "from .database import initialize_database\n"
    } else {
        ""
    };
    let sqlite_init = if spec.sqlite {
        "SQLITE_PATH = initialize_database(CONFIG.sqlite_path) if CONFIG.sqlite_enabled else None\nif SQLITE_PATH is not None:\n    LOGGER.info(\"sqlite initialized\", extra={\"path\": str(SQLITE_PATH)})\n"
    } else {
        "SQLITE_PATH = None\n"
    };
    let sqlite_health = if spec.sqlite {
        "        \"sqlite_enabled\": CONFIG.sqlite_enabled,\n        \"sqlite_path\": str(SQLITE_PATH) if SQLITE_PATH is not None else None,\n"
    } else {
        ""
    };

    format!(
        r#"from __future__ import annotations

import uvicorn
from fastapi import FastAPI

{sqlite_import}from .logging import get_logger, setup_logging
from .settings import load_app_config

CONFIG = load_app_config()
setup_logging(CONFIG)
LOGGER = get_logger(__name__)
{sqlite_init}
app = FastAPI(title=CONFIG.app_name, debug=CONFIG.debug)


@app.get("/health")
def health() -> dict[str, object]:
    LOGGER.info("health endpoint called")
    return {{
        "status": "ok",
        "service": CONFIG.app_name,
        "grpc_enabled": CONFIG.grpc_enabled,
{sqlite_health}    }}


def main() -> None:
    LOGGER.info("starting fastapi server", extra={{"host": CONFIG.host, "port": CONFIG.port}})
    uvicorn.run(app, host=CONFIG.host, port=CONFIG.port)


if __name__ == "__main__":
    raise SystemExit(main())
"#,
        sqlite_import = sqlite_import,
        sqlite_init = sqlite_init,
        sqlite_health = sqlite_health,
    )
}

pub(crate) fn build_smoke_test(spec: &ProjectSpec) -> String {
    format!(
        r#"from __future__ import annotations

from fastapi.testclient import TestClient

from {package_name}.main import app

client = TestClient(app)


def test_health():
    response = client.get("/health")
    assert response.status_code == 200
"#,
        package_name = spec.package_name,
    )
}
