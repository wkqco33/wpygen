use crate::models::ProjectSpec;

use super::common;

pub(crate) fn build_settings(spec: &ProjectSpec) -> String {
    let project_name = &spec.project_name;

    common::build_settings_module(
        spec,
        "    window_title: str\n",
        &format!("                \"window_title\": \"{project_name}\",\n"),
        &format!(
            "        window_title=str(config.get(\"app.window_title\", \"{project_name}\")),\n"
        ),
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
