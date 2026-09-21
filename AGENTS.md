# AGENTS.md — 개발 가이드

이 문서는 이 저장소에서 작업하는 에이전트(및 인간 개발자)가 반드시 따라야 할
개발 방식을 정의합니다. 작업을 시작하기 전에 먼저 읽으세요.

## 프로젝트 개요

`wpygen`은 Rust로 작성된 Python 프로젝트 템플릿 생성기입니다. `cli`/`gui`/`server`
템플릿을 만들고 옵션으로 `gRPC`/`SQLite` 구성을 추가할 수 있습니다. 생성되는
프로젝트는 `uv` 기준이며 공식 PyPI에서 패키지를 설치합니다.

- CLI 프레임워크: `wrcli` (clap 대체)
- Rust edition: 2024
- 최소 Python 버전(생성 대상): 3.12

## 개발 방식 (TDD)

이 프로젝트는 **테스트 우선(TDD)** 방식으로 개발합니다. 모든 기능 변경은 다음
순서를 따릅니다.

1. **실패하는 테스트를 먼저 작성**한다. 기존 동작을 바꾸는 경우에도 회귀를 막기
   위해 해당 동작을 검증하는 테스트를 먼저 추가한다.
2. **테스트가 통과하도록 최소한의 코드를 작성**한다.
3. **리팩터링**한다. 이때 테스트가 여전히 통과하는지 확인한다.

### 테스트 계층

- **단위 테스트**: 각 모듈의 `#[cfg(test)] mod tests` 블록에 둔다. 순수 로직
  (이름 정규화, 템플릿 렌더링, 대상 디렉터리 검증 등)은 파일시스템/외부 명령에
  의존하지 않게 테스트한다.
- **통합 테스트**: `tests/generated_projects.rs`에서 실제 바이너리를 실행해
  생성된 프로젝트가 Python으로 컴파일되는지 검증한다. `uv`/PyPI 접근이 필요한
  테스트는 `#[ignore]`로 표시한다. `tests/cli_contract.rs`는 출력 스트림 분리,
  종료 코드, `--json` 출력, 도움말 내용을 검증한다.

### 테스트 실행

```bash
cargo test                 # 단위 + 통합 테스트
cargo test -- --ignored    # uv/PyPI가 필요한 통합 테스트
```

## 코드 구조

역할 분리는 다음 기준을 따릅니다. 새 코드도 이 구조를 유지하세요.

- `src/cli.rs` — CLI 트리 정의와 인자 파싱(`NewArgs`)
- `src/cmds/` — 커맨드 실행 로직
- `src/models/` — 데이터 구조(`ProjectSpec`, `TemplateKind`, `GeneratedFile`)
- `src/report.rs` — 결과 리포트(텍스트/`--json`) 생성
- `src/services/` — 파일 생성/쓰기(`writer`), 외부 명령 실행(`process`)
- `src/templates/` — 템플릿 문자열 생성
- `src/error.rs` — 오류 타입과 종료 코드 매핑

## 코딩 규칙

- **포맷**: `cargo fmt` 결과를 유지한다. CI에서 `cargo fmt --check`로 검증한다.
- **린트**: `cargo clippy --all-targets --locked -- -D warnings`를 통과해야 한다.
- **주석**: 장황한 설명, 에이전트의 독백, 자명한 코드에 대한 주석을 달지 않는다.
  주석은 "왜(why)"를 설명할 때만 짧게 단다. "무엇(what)"은 코드가 스스로
  드러내야 한다.
- **오류 처리**: 사용자가 인자를 바꾸면 해결되는 오류는 `Error::exit_code() == 2`,
  환경/시스템 오류는 `1`을 반환한다. 새 오류도 이 규칙을 따른다.
- **출력 스트림**: 결과(생성 요약, dry-run 목록, `--json` JSON)는 stdout, 진행·상태·
  오류는 stderr로 보낸다. `--json` 모드에서는 자식 프로세스 stdout도 stderr로
  돌려 stdout을 JSON 전용으로 유지한다.
- **CLI 계약**: 플래그, 종료 코드, `--json` 스키마를 바꾸면
  `tests/cli_contract.rs`와 `CHANGELOG.md`를 함께 갱신한다.
- **문자열 템플릿**: 생성되는 Python 코드는 `format!`의 `{{ }}` 이스케이프에
  주의한다. Python 불리언은 `True`/`False`(소문자 아님)여야 한다.

## 커밋 규칙

- 커밋 메시지는 한국어로 간결하게 작성한다.
- `Cargo.lock`은 실행 바이너리이므로 커밋한다.
- `target/`, `.env`, 생성 산출물은 커밋하지 않는다.

## CI

`.github/workflows/ci.yml`에서 다음을 검증한다.

1. gitleaks 시크릿 스캔
2. `cargo fmt --check`
3. `cargo clippy -- -D warnings`
4. `cargo test` (linux/darwin amd64·arm64/windows 4개 플랫폼)
5. 생성된 프로젝트의 Python 컴파일 및 uv 검증(`--ignored`, ubuntu에서 1회)

의존성은 `.github/dependabot.yml`이 주간으로 갱신 PR을 만든다(cargo, github-actions).

## 릴리스

`.github/workflows/release.yml`이 태그 푸시 시 플랫폼별 바이너리를 빌드하고
GitHub Release에 업로드합니다. `ppm.json`의 `bin_name`(`wpygen`)과 아카이브
명명 규칙(`wpygen_{os}_{arch}.{ext}`)을 유지하세요. 자세한 규칙은
`PACKAGE_GUIDE.md`를 참고하세요.

- `Cargo.toml`의 `version`과 git 태그가 일치해야 합니다(`0.4.0` ↔ `v0.4.0`).
  워크플로우의 `verify-version` 잡이 이를 강제하므로 버전 변경 시 `Cargo.lock`과
  `CHANGELOG.md`도 함께 커밋합니다.
- 릴리스 업로드는 모든 플랫폼 빌드가 끝난 뒤 `publish` 잡에서 한 번만 수행합니다.
  특정 플랫폼 자산이 없으면 릴리스가 나오지 않도록 필수 자산 검사를 유지하세요.
- 릴리스 자산에는 빌드 출처(provenance) attestation이 함께 생성됩니다
  (`gh attestation verify`로 검증). `publish` 잡의 `id-token: write`,
  `attestations: write` 권한을 제거하지 마세요.
