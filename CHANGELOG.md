# Changelog

이 프로젝트의 주요 변경 사항을 기록합니다. 형식은
[Keep a Changelog](https://keepachangelog.com/ko/1.1.0/)를, 버전 규칙은
[Semantic Versioning](https://semver.org/lang/ko/)을 따릅니다.

`0.y.z` 구간에서는 CLI 표면(플래그, 종료 코드, `--json` 스키마)이 아직 안정적이지
않습니다. 하위 호환을 깨는 변경은 MINOR 버전에서 일어날 수 있으며, 그 경우 이
문서에 마이그레이션 방법을 적습니다.

## [Unreleased]

## [0.5.0] - 2026-09-21

### Added

- clig.dev 전역 플래그를 모든 서브커맨드에서 사용할 수 있습니다: `--plain`(파일
  경로를 줄 단위로 출력), `--no-color`, `--color <auto|always|never>`,
  `--no-input`, `-f/--force`(단축 플래그 추가). `--plain`과 `--json`은 함께 쓸 수
  없습니다(종료 코드 2).
- 도움말에 `Examples:` 섹션과 `Support:`/`Documentation:` 링크가 추가되었습니다.
- 내장 `help` 서브커맨드(`wpygen help new`)가 추가되었습니다.
- 사람이 읽는 긴 dry-run 목록을 터미널에서만 pager(`PAGER`, 기본 `less -FIRX`)로
  넘깁니다.

### Changed

- `wrcli`를 `0.1`에서 `0.5`로 올렸습니다. 파서 오류의 종료 코드 분류를 라이브러리
  기준(`is_usage_error`)으로 따르므로, 새 검증 오류도 자동으로 종료 코드 2가 됩니다.
- 전역 플래그 설명을 한국어로 유지하기 위해 표준 플래그를 직접 등록합니다
  (`--confirm`은 wpygen에 의미가 없어 제외).
- `--force`는 이제 전역 플래그이므로 서브커맨드 앞뒤 어디에나 올 수 있습니다.

### Fixed

- 셸 자동완성이 수정되었습니다: bash 스크립트의 잘못된 닫는 따옴표 제거, zsh/fish
  완성에 플래그·`help` 서브커맨드 추가.
- `--plain`/`--json` 동시 사용이 종료 코드 2(입력 오류)로 처리됩니다(이전에는 1).

## [0.4.0] - 2026-09-21

### Added

- `new --json`: 생성 결과를 기계 판독용 JSON 한 덩어리로 stdout에 출력합니다.
  키 순서를 계약으로 취급하며 `wpygen_version`, `project_name`, `package_name`,
  `template`, `grpc`, `sqlite`, `target_dir`, `dry_run`, `file_count`, `files`를
  포함합니다.
- `new -q, --quiet`: 진행·상태 메시지를 출력하지 않습니다(`--verbose`보다 우선).
- `new -n`: `--dry-run`의 단축 플래그.
- `wpygen --help`, `wpygen new --help`에 전체 실행 경로, 예시, 문서·이슈 링크를
  추가했습니다.
- 릴리스 아티팩트 4종에 빌드 출처(provenance) attestation을 추가했습니다.
  `gh attestation verify <asset> --repo wkqco33/wpygen`으로 검증할 수 있습니다.
- CI가 릴리스와 동일한 4개 플랫폼(linux amd64, darwin amd64, darwin arm64,
  windows amd64)에서 테스트를 실행합니다.

### Changed

- 진행·상태 메시지(`git`/`uv` 완료 알림)를 stdout에서 stderr로 옮겼습니다. stdout에는
  결과(생성 요약, dry-run 목록, `--json` JSON)만 남습니다.
- `--json` 모드에서는 자식 프로세스의 stdout도 stderr로 돌려, stdout이 JSON만
  담도록 합니다.
- 외부 명령 실행 단계가 stdout을 오염시키지 않도록 출력 스트림 처리를 정리했습니다.
- 생성되는 프로젝트의 CI 워크플로우가 `actions/checkout@v7`,
  `astral-sh/setup-uv@v10`을 사용합니다.

### Fixed

- 외부 명령을 실행할 수 없을 때(예: `git`이 PATH에 없음) 엉뚱한 디렉터리 경로가 아니라
  실행하려던 명령 이름과 원인을 보여줍니다.
- 릴리스에서 플랫폼 하나가 실패해도 자산이 일부만 담긴 릴리스가 조용히 공개되던
  문제를 막았습니다(모든 빌드가 끝난 뒤 `publish` 잡에서 한 번만 업로드하고, 필수
  자산 4종이 없으면 실패).
- `0.4.0` 이전 릴리스는 `Cargo.toml` 버전이 `0.1.0`에 멈춰 있어 바이너리가 실제
  릴리스 버전과 다른 값을 보고했습니다. 태그와 `Cargo.toml` 버전을 비교하는
  `verify-version` 잡을 추가해 재발을 막았습니다.

## [0.3.0] - 2026-09-21

### Fixed

- `macos-14` 러너에 `sha256sum`이 없어 darwin arm64 자산이 릴리스에서 빠지던 문제를
  고쳤습니다. 체크섬은 `publish` 잡(ubuntu)에서만 만듭니다.

### Changed

- 생성기 내부 중복을 제거했습니다(템플릿 파일 목록 단일화, GUI/SERVER 설정 모듈의
  공통 뼈대 추출). 생성 결과는 이전 버전과 바이트 단위로 동일합니다.
- 버전을 `0.1.0`에서 `0.3.0`으로 올려 바이너리 버전을 태그와 일치시켰습니다.

## [0.2.0] - 2026-08-24

### Added

- 생성되는 프로젝트가 `wpyconf`/`wpylog`/`wpycli`를 공식 PyPI에서 설치하도록
  전환했습니다.
- `ppm` 배포용 `ppm.json`과 릴리스 워크플로우를 추가했습니다.

### Changed

- `--sqlite`, `--grpc` 옵션 구성과 문서를 정리했습니다.

## [0.1.5] - 2026-08-18

### Changed

- 생성되는 프로젝트의 기본 구성을 정리하고 문서를 보강했습니다.

## [0.1.4] - 2026-07-22

### Changed

- linux 릴리스 타겟을 `x86_64-unknown-linux-musl`로 바꾸고 musl 도구 설치 단계를
  추가했습니다.

## [0.1.3] - 2026-07-21

### Fixed

- 패키지명 정규화(연속 구분자, 앞뒤 구분자 처리)를 개선했습니다.

## [0.1.2] - 2026-07-20

### Added

- 프로젝트 생성 옵션(`--force`, `--dry-run`, `--git`, `--sync`, `--lock`,
  `--package-name`)과 CI 워크플로우를 추가했습니다.

## [0.1.1] - 2026-04-27

### Changed

- README와 템플릿 모듈을 정리했습니다.

## [0.1.0] - 2026-04-15

### Added

- 최초 릴리스. `cli`/`gui`/`server` 템플릿 생성과 `--grpc`/`--sqlite` 옵션을
  제공합니다.
