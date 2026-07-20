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
    window_title: str
    log_level: str
    log_file: str | None
    grpc_enabled: bool
{sqlite_fields}


def load_app_config() -> AppConfig:
    config = load_config(
        defaults={{
            "app": {{
                "name": "{project_name}",
                "window_title": "{project_name}",
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
        window_title=str(config.get("app.window_title", "{project_name}")),
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
        "    database_path = initialize_database(config.sqlite_path) if config.sqlite_enabled else None\n    if database_path is not None:\n        logger.info(\"sqlite initialized\", extra={\"path\": str(database_path)})\n"
    } else {
        "    database_path = None\n"
    };
    let grpc_note = if spec.grpc {
        "        lines.append(\"gRPC scaffolding is enabled for this project.\")\n"
    } else {
        ""
    };

    format!(
        r#"from __future__ import annotations

import sys

from PySide6.QtWidgets import QApplication, QLabel, QMainWindow, QVBoxLayout, QWidget

{sqlite_import}from .logging import get_logger, setup_logging
from .settings import AppConfig, load_app_config


class MainWindow(QMainWindow):
    def __init__(self, config: AppConfig, database_path: str | None = None) -> None:
        super().__init__()
        self.setWindowTitle(config.window_title)

        body = QWidget(self)
        layout = QVBoxLayout(body)
        lines = [
            f"project={{config.app_name}}",
            "PySide6 template is ready.",
        ]
        if database_path is not None:
            lines.append(f"sqlite={{database_path}}")
{grpc_note}        label = QLabel("\n".join(lines), parent=body)
        layout.addWidget(label)
        self.setCentralWidget(body)


def main() -> int:
    config = load_app_config()
    setup_logging(config)
    logger = get_logger(__name__)
    logger.info("starting gui application")
{sqlite_init}

    app = QApplication(sys.argv)
    window = MainWindow(config, str(database_path) if database_path is not None else None)
    window.resize(720, 240)
    window.show()
    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
"#,
        sqlite_import = sqlite_import,
        grpc_note = grpc_note,
        sqlite_init = sqlite_init,
    )
}

pub(crate) fn build_smoke_test(spec: &ProjectSpec) -> String {
    format!(
        r#"from __future__ import annotations

from {package_name}.settings import load_app_config


def test_load_app_config_defaults():
    config = load_app_config()
    assert config.app_name
"#,
        package_name = spec.package_name,
    )
}
