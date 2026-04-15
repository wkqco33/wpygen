# wpygen

Rust로 만든 Python 프로젝트 템플릿 생성기입니다.  
`CLI`, `GUI`, `SERVER` 템플릿을 만들 수 있고, 옵션으로 `gRPC`, `SQLite` 구성을 함께 넣을 수 있습니다.

생성되는 프로젝트는 `uv` 기준으로 구성되며, 내부 패키지 인덱스 `https://pypi.wkqcosoft.cloud` 를 사용합니다.

## 지원 템플릿

| 템플릿 | 기본 구성 | 선택 옵션 |
| --- | --- | --- |
| `cli` | `wpycli`, `wconfig`, `wlogger` | `gRPC`, `SQLite` |
| `gui` | `PySide6`, `wconfig`, `wlogger` | `gRPC`, `SQLite` |
| `server` | `FastAPI`, `uvicorn`, `wconfig`, `wlogger` | `gRPC`, `SQLite` |

## 생성되는 기본 설정

- Python 버전: `>=3.12`
- 패키지 관리: `uv`
- 빌드 백엔드: `hatchling`
- 내부 인덱스:

```toml
[[tool.uv.index]]
name = "wkqcosoft"
url = "https://pypi.wkqcosoft.cloud"
```

## ppm 배포 설정

이 저장소는 `ppm`에서 사용할 수 있도록 다음 구성을 포함합니다.

- 레포지토리/바이너리 이름: `wpygen`
- 루트 메타데이터 파일: `ppm.json`
- GitHub Releases용 빌드 워크플로우: `.github/workflows/release.yml`

`ppm.json`에는 다음 정보가 들어갑니다.

```json
{
  "description": "uv 기반 Python 프로젝트 템플릿 생성기",
  "author": "wkqco",
  "bin_name": "wpygen"
}
```

릴리스 워크플로우는 태그 푸시 시 아래 형식의 아티팩트를 올리도록 구성했습니다.

- `wpygen_linux_amd64.tar.gz`
- `wpygen_darwin_amd64.tar.gz`
- `wpygen_darwin_arm64.tar.gz`
- `wpygen_windows_amd64.zip`

태그 예시:

```bash
git tag v0.1.0
git push origin v0.1.0
```

이후 `ppm install <owner>/wpygen` 형태로 사용할 수 있는 릴리스 자산 구조를 맞추는 용도입니다.

## 사용 방법

### 빌드

```bash
cargo build
```

### 실행

```bash
cargo run -- new -t cli my_app
```

### 도움말

```bash
cargo run -- --help
cargo run -- new --help
```

## 명령 형식

```bash
wpygen new [OPTIONS] --template <cli|gui|server> <NAME>
```

### 옵션

| 옵션 | 설명 |
| --- | --- |
| `-t, --template` | 생성할 템플릿 종류 |
| `--grpc` | gRPC 세팅 추가 |
| `--sqlite` | SQLite 세팅 추가 |
| `-o, --output` | 생성할 상위 디렉터리 |
| `--package-name` | Python 패키지명 직접 지정 |
| `--force` | 대상 디렉터리가 비어있지 않아도 생성 |
| `--index-url` | 사설 패키지 인덱스 URL 변경 |

## 예시

### CLI 템플릿 생성

```bash
cargo run -- new -t cli test_cli
```

### CLI + SQLite

```bash
cargo run -- new -t cli --sqlite test_cli
```

### SERVER + gRPC + SQLite

```bash
cargo run -- new -t server --grpc --sqlite test_server
```

### GUI 템플릿 생성 경로 지정

```bash
cargo run -- new -t gui -o ./examples my_gui
```

## 생성 결과 예시

CLI 템플릿 기준:

```text
test_cli/
├── .env.example
├── .gitignore
├── .python-version
├── config.toml
├── pyproject.toml
├── README.md
└── src/
    └── test_cli/
        ├── __init__.py
        └── main.py
```

옵션에 따라 추가 파일이 생깁니다.

- `--sqlite`: `src/<package>/database.py`
- `--grpc`: `proto/<package>.proto`, `tools/generate_grpc.py`, `src/<package>/grpc/__init__.py`

## 템플릿별 특징

### CLI

- `wpycli` 기반 명령 구조
- `wconfig` + `wlogger` 런타임 설정
- `--sqlite` 사용 시 `db-init` 명령 포함

### GUI

- `PySide6` 기반 메인 윈도우 예제
- 설정 로더와 로깅 초기화 포함
- `--sqlite` 사용 시 앱 시작 시 DB 초기화

### SERVER

- `FastAPI` + `uvicorn` 구성
- `/health` 엔드포인트 포함
- `--sqlite` 사용 시 서버 시작 시 DB 초기화
- `--grpc` 사용 시 proto 및 코드 생성 스크립트 포함

## 현재 소스 구조

```text
src/
├── cli.rs
├── error.rs
├── main.rs
├── cmds/
│   ├── mod.rs
│   └── new.rs
├── models/
│   ├── generated_file.rs
│   ├── mod.rs
│   └── project_spec.rs
├── services/
│   ├── mod.rs
│   └── writer.rs
└── templates/
    ├── cli.rs
    ├── common.rs
    ├── grpc.rs
    ├── gui.rs
    ├── mod.rs
    ├── server.rs
    └── sqlite.rs
```

역할 분리는 다음 기준입니다.

- `cmds`: 커맨드 실행 로직
- `models`: 데이터 구조
- `services`: 파일 생성/쓰기
- `templates`: 템플릿 문자열 생성

## 개발

### 테스트

```bash
cargo test
```

### 포맷

```bash
cargo fmt
```

## 참고

- 기본 내부 인덱스 URL은 `https://pypi.wkqcosoft.cloud`
- CLI 템플릿은 현재 `wpycli>=0.1.1` 기준으로 생성
- 생성된 프로젝트 안에서도 `uv sync` 기준으로 바로 사용할 수 있게 구성됨
- `ppm`용 릴리스 아티팩트는 GitHub Actions `release` 워크플로우에서 생성됨
