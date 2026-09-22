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

## 설치

### ppm

```bash
ppm install wkqco33/wpygen
```

### GitHub Releases

릴리스 자산은 `wpygen_<os>_<arch>.{tar.gz,zip}` 형식이며, 각 자산마다
`.sha256` 체크섬이 함께 올라갑니다. 빌드는 GitHub Actions에서만 수행합니다.

```bash
# 예: darwin arm64
curl -LO https://github.com/wkqco33/wpygen/releases/latest/download/wpygen_darwin_arm64.tar.gz
curl -LO https://github.com/wkqco33/wpygen/releases/latest/download/wpygen_darwin_arm64.tar.gz.sha256
shasum -a 256 -c wpygen_darwin_arm64.tar.gz.sha256
tar -xzf wpygen_darwin_arm64.tar.gz
install -m 755 wpygen /usr/local/bin/wpygen
```

빌드 출처 증명(provenance)은 다음 명령으로 검증합니다.

```bash
gh attestation verify wpygen_darwin_arm64.tar.gz --repo wkqco33/wpygen
```

### 소스에서 빌드

Rust 1.85 이상(edition 2024)이 필요합니다.

```bash
cargo install --git https://github.com/wkqco33/wpygen --locked
```

## 지원 플랫폼

릴리스와 CI가 함께 검증하는 조합입니다.

| OS | arch | Rust target | 자산 |
| --- | --- | --- | --- |
| linux | amd64 | `x86_64-unknown-linux-musl` | `wpygen_linux_amd64.tar.gz` |
| darwin | amd64 | `x86_64-apple-darwin` | `wpygen_darwin_amd64.tar.gz` |
| darwin | arm64 | `aarch64-apple-darwin` | `wpygen_darwin_arm64.tar.gz` |
| windows | amd64 | `x86_64-pc-windows-msvc` | `wpygen_windows_amd64.zip` |

그 외 플랫폼은 소스 빌드로 사용할 수 있지만 릴리스 자산은 제공하지 않습니다.

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

### `new` 커맨드 (기본 템플릿 생성)

```bash
wpygen new [OPTIONS] --template <cli|gui|server> <NAME>
```

#### 옵션

| 옵션 | 설명 |
| --- | --- |
| `-t, --template` | 생성할 템플릿 종류 (cli\|gui\|server) [필수] |
| `--grpc` | gRPC 세팅 추가 |
| `--sqlite` | SQLite 세팅 추가 |
| `-o, --output` | 생성할 상위 디렉터리 (기본값: `.`) |
| `--package-name` | Python 패키지명 직접 지정 |
| `-v, --verbose` | 생성 진행 상황을 상세히 출력 (stderr) |
| `-n, --dry-run` | 실제로 쓰지 않고 생성될 파일 경로만 출력 (`--git`/`--sync`/`--lock`과 동시 사용 불가) |
| `--git` | 생성 후 `git init` + 최초 커밋 실행 |
| `--sync` | 생성 후 `uv sync` 실행 |
| `--lock` | 생성 후 `uv lock` 실행 (`--sync`와 함께 사용하면 lock 후 sync) |

### `ai` 커맨드 (AI 맞춤형 템플릿 생성)

자연어 요구사항을 바탕으로 Ollama, OpenAI, Azure OpenAI 모델을 통해 프로젝트 구조와 코드를 설계하고 생성합니다.

```bash
wpygen ai [OPTIONS] --name <NAME> "<PROMPT>"
```

#### 예시

```bash
# 로컬 Ollama를 사용한 프로젝트 생성 (기본값)
wpygen ai --name order_svc "FastAPI와 Redis, PostgreSQL을 쓰는 주문 처리 서비스"

# OpenAI 모델 지정
wpygen ai --name worker_app --provider openai --model gpt-4o-mini "Celery 비동기 작업 큐"

# Azure OpenAI 엔드포인트 지정
wpygen ai --name enterprise_app --provider azure-openai --endpoint https://my-res.openai.azure.com --model gpt-4o "엔터프라이즈 데이터 파이프라인"

# 생성될 파일 미리보기
wpygen ai --name test_app --dry-run "간단한 웹 스크래퍼 CLI"
```

#### 옵션

| 옵션 | 설명 |
| --- | --- |
| `--name` | 생성할 프로젝트(디렉터리) 이름 [필수] |
| `--provider` | AI 프로바이더 (`ollama`\|`openai`\|`azure-openai`, 기본값: `ollama`) |
| `--model` | 사용할 모델 이름 (기본값: ollama는 `llama3.2`, openai는 `gpt-4o-mini`) |
| `--endpoint` | API 엔드포인트 URL (기본값: ollama는 `http://localhost:11434`, openai는 `https://api.openai.com/v1`) |
| `--api-key` | AI API 인증 키 (`OPENAI_API_KEY`, `AZURE_OPENAI_API_KEY`, `WPYGEN_AI_KEY` 환경 변수로도 설정 가능) |
| `-o, --output` | 생성할 상위 디렉터리 (기본값: `.`) |
| `--package-name` | Python 패키지명 직접 지정 |
| `-v, --verbose` | 생성 진행 상황 상세 출력 |
| `-n, --dry-run` | 실제로 쓰지 않고 생성될 파일 경로만 출력 |
| `--git` | 생성 후 `git init` + 최초 커밋 실행 |
| `--sync` | 생성 후 `uv sync` 실행 |
| `--lock` | 생성 후 `uv lock` 실행 |

### `config` 커맨드 (설정 관리)

플랫폼별 기본 설정 디렉터리에 `config.toml`을 저장하여, 매번 옵션을 입력하지 않고도 기본 AI 모델, 프로바이더, 엔드포인트 등을 관리할 수 있습니다.

- **Linux/Unix**: `~/.config/wpygen/config.toml`
- **macOS**: `~/Library/Application Support/wpygen/config.toml`
- **Windows**: `%APPDATA%\wpygen\config.toml`

```bash
# 기본 템플릿으로 설정 파일 생성 (--force 로 덮어쓰기 가능)
wpygen config init

# 현재 설정 파일 내용 출력 (--json 지원)
wpygen config show

# 설정 파일의 전체 절대 경로 출력
wpygen config path

# 설정값 변경 및 저장
wpygen config set ai.provider openai
wpygen config set ai.model gpt-4o-mini
wpygen config set ai.api_key sk-...

# 설정값 단일 조회
wpygen config get ai.provider
```

#### 지원하는 설정 키

| 키 | 설명 | 기본값 |
| --- | --- | --- |
| `ai.provider` | 기본 AI 프로바이더 (`ollama`\|`openai`\|`azure-openai`) | `ollama` |
| `ai.model` | 기본 AI 모델명 | `llama3.2` |
| `ai.endpoint` | 기본 API 엔드포인트 URL | 프로바이더 기본 URL |
| `ai.api_key` | 기본 API 키 | `""` |
| `defaults.output` | 기본 생성 상위 디렉터리 | `"."` |

### 전역 플래그

모든 서브커맨드에서 사용할 수 있습니다(clig.dev 표준 묶음).

| 옵션 | 설명 |
| --- | --- |
| `-q, --quiet` | wpygen의 진행·상태 메시지를 출력하지 않음 (자식 명령 출력은 그대로 전달) |
| `-f, --force` | 대상 디렉터리가 비어있지 않아도 덮어씀 |
| `--json` | 결과를 JSON으로 stdout에 출력 (기계 판독) |
| `--plain` | 파일 경로를 한 줄에 하나씩 stdout에 출력 (기계 판독, `--json`과 배타) |
| `--no-color` | 색상 출력을 끔 |
| `--color <auto\|always\|never>` | 색상 사용 시점 (기본값 `auto`) |
| `--no-input` | 프롬프트를 쓰지 않음 (wpygen은 항상 비대화형이라 동작 변화는 없음) |

## 출력과 종료 코드

- **stdout**: 결과. 생성 요약, `--dry-run` 파일 목록, `--plain` 경로 목록,
  `--json` JSON 객체.
- **stderr**: 진행·상태 알림(`-v` 상세 로그, `git`/`uv` 완료), 오류 메시지.
- `--json`/`--plain`은 stdout을 기계 판독 출력 전용으로 씁니다. 자식 프로세스
  (`git`, `uv`)의 stdout도 stderr로 돌려 결과를 오염시키지 않습니다.
- `--plain`은 생성/생성 예정 파일의 경로를 한 줄에 하나씩 출력합니다(스크립트용).
- 사람이 읽는 긴 dry-run 목록은 stdout이 터미널일 때만 pager(`PAGER`, 기본
  `less -FIRX`)로 넘깁니다. 파이프/CI에서는 그대로 출력되고, `--quiet`면 pager를
  쓰지 않습니다.
- 색상은 `auto`일 때 stdout/stderr가 터미널이고 `NO_COLOR`·`TERM=dumb`이 아닐 때만
  켜집니다. `--color=always`로 강제하고 `--no-color`/`--color=never`로 끌 수 있습니다.

| 종료 코드 | 의미 |
| --- | --- |
| `0` | 성공 |
| `1` | 환경·시스템 오류 (파일 I/O 실패, 외부 명령 실행 실패 등) |
| `2` | 입력 오류 (잘못된 이름/템플릿, 충돌하는 플래그, 대상 디렉터리 상태, `--plain`+`--json` 동시 사용, 미인식 플래그) |

`--json`/`--plain` 출력 계약과 종료 코드는 `CHANGELOG.md`에 변경을 기록하며,
플래그 변경은 additive하게 유지합니다.

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

### 생성 계획을 JSON으로 받기

```bash
wpygen new -t server --grpc --sqlite --dry-run --json my_service | jq '.files[]'
```

### 생성될 파일 경로를 줄 단위로 받기

```bash
wpygen new -t cli --dry-run --plain -o ./examples my_app
```

### 자동화에서 조용히 생성하기

```bash
wpygen new -t cli -q --git --sync my_app
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
├── report.rs       # 결과 리포트(텍스트/JSON) 생성
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
- 버전별 변경 사항은 [`CHANGELOG.md`](CHANGELOG.md)에 기록합니다. `0.y.z` 구간에서는
  CLI 표면이 아직 안정적이지 않으므로, 하위 호환을 깨는 변경은 MINOR 버전에서
  일어날 수 있습니다.
- 재현 가능한 의존성 설치가 필요하면 생성 시 `--lock`을 사용하고 `uv.lock`을 커밋합니다.
- 생성된 프로젝트 안에서도 `uv sync` 기준으로 바로 사용할 수 있게 구성됨
- `ppm`용 릴리스 아티팩트는 GitHub Actions `release` 워크플로우에서 생성됨
