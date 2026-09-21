use crate::models::ProjectSpec;

use super::common;

pub(crate) fn build_settings(spec: &ProjectSpec) -> String {
    common::build_settings_module(
        spec,
        "    debug: bool\n    host: str\n    port: int\n",
        concat!(
            "                \"debug\": False,\n",
            "                \"host\": \"127.0.0.1\",\n",
            "                \"port\": 8000,\n",
        ),
        concat!(
            "        debug=_as_bool(config.get(\"app.debug\", False)),\n",
            "        host=str(config.get(\"app.host\", \"127.0.0.1\")),\n",
            "        port=int(config.get(\"app.port\", 8000)),\n",
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
