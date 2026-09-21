# wpygen

Rust로 만든 Python 프로젝트 템플릿 생성기입니다.  
`CLI`, `GUI`, `SERVER` 템플릿을 만들 수 있고, 옵션으로 `gRPC`, `SQLite` 구성을 함께 넣을 수 있습니다.

생성되는 프로젝트는 `uv` 기준으로 구성되며, 공식 PyPI에서 패키지를 설치합니다.

## 지원 템플릿

| 템플릿 | 기본 구성 | 선택 옵션 |
| --- | --- | --- |
| `cli` | `wpycli`, `wpyconf`, `wpylog` | `gRPC`, `SQLite` |
| `gui` | `PySide6`, `wpyconf`, `wpylog` | `gRPC`, `SQLite` |
| `server` | `FastAPI`, `uvicorn`, `wpyconf`, `wpylog` | `gRPC`, `SQLite` |

## 생성되는 기본 설정

- Python 버전: `>=3.12`
- 패키지 관리: `uv`
- 빌드 백엔드: `hatchling`
- 패키지 인덱스: 공식 PyPI

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
  "homepage": "https://github.com/wkqco33/wpygen",
  "bin_name": "wpygen"
}
```

릴리스 워크플로우는 태그 푸시 시 아래 형식의 아티팩트와 각각의 SHA-256 체크섬을
올리도록 구성했습니다.

- `wpygen_linux_amd64.tar.gz` (+ `.sha256`)
- `wpygen_darwin_amd64.tar.gz` (+ `.sha256`)
- `wpygen_darwin_arm64.tar.gz` (+ `.sha256`)
- `wpygen_windows_amd64.zip` (+ `.sha256`)

태그 예시:

```bash
# Cargo.toml 의 version 과 태그가 일치해야 합니다 (v0.3.0 ↔ 0.3.0)
git tag v0.3.0
git push origin v0.3.0
```

빌드는 4개 플랫폼에서 병렬로 수행하고, 릴리스 업로드는 모든 빌드가 끝난 뒤
`publish` 잡에서 한 번만 수행합니다. 따라서 특정 플랫폼이 실패하면 자산이 일부만
담긴 릴리스가 공개되지 않고 워크플로우 자체가 실패합니다.

버전·태그·아티팩트 규칙은 [`PACKAGE_GUIDE.md`](PACKAGE_GUIDE.md)를 참고하세요.

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
| `-v, --verbose` | 생성 진행 상황을 상세히 출력 |
| `--dry-run` | 실제로 쓰지 않고 생성될 파일 목록만 출력 (`--git`/`--sync`/`--lock`과 동시 사용 불가) |
| `--git` | 생성 후 `git init` + 최초 커밋 실행 |
| `--sync` | 생성 후 `uv sync` 실행 |
| `--lock` | 생성 후 `uv lock` 실행 (`--sync`와 함께 사용하면 lock 후 sync) |

## 쉘 자동완성

```bash
# bash
wpygen completions bash > /etc/bash_completion.d/wpygen
# zsh
wpygen completions zsh > "${fpath[1]}/_wpygen"
```

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
- `wpyconf` + `wpylog` 런타임 설정 (`wconfig` + `wlogger`로 import)
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
├── testing.rs      # 테스트 전용 헬퍼 (cfg(test))
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

에이전트/개발자를 위한 개발 방식(TDD, 코드 구조, 코딩 규칙)은
[`AGENTS.md`](AGENTS.md)를 참고하세요.

### 테스트

```bash
cargo test
```

### 포맷

```bash
cargo fmt
```

## 기여

버그 리포트, 기능 제안, 기여 방법은 [`CONTRIBUTING.md`](CONTRIBUTING.md)를
참고하세요. 보안 취약점은 [`SECURITY.md`](SECURITY.md)의 절차를 따르세요.

## 라이선스

이 프로젝트는 MIT 라이선스로 배포됩니다. 자세한 내용은
[`LICENSE`](LICENSE)를 참고하세요.

## 참고

- `wpycli`, `wpyconf`, `wpylog`는 공식 PyPI에서 설치됩니다.
- 릴리스 버전·태그·아티팩트 규칙은 [`PACKAGE_GUIDE.md`](PACKAGE_GUIDE.md)를 참고하세요.
- 재현 가능한 의존성 설치가 필요하면 생성 시 `--lock`을 사용하고 `uv.lock`을 커밋합니다.
- 생성된 프로젝트 안에서도 `uv sync` 기준으로 바로 사용할 수 있게 구성됨
- `ppm`용 릴리스 아티팩트는 GitHub Actions `release` 워크플로우에서 생성됨
