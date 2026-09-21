# PACKAGE_GUIDE.md — 배포 가이드

이 문서는 `wpygen` 바이너리를 GitHub Releases와 `ppm`으로 배포할 때 지켜야 할
규칙을 정의합니다. 릴리스 워크플로우는 `.github/workflows/release.yml`에 있습니다.

## 버전 규칙

- 버전의 단일 출처는 `Cargo.toml`의 `[package] version`이며, `Cargo.lock`도 함께
  갱신합니다(실행 바이너리이므로 커밋 대상).
- git 태그는 `v` + `Cargo.toml` 버전과 **정확히** 일치해야 합니다(예: `0.3.0` →
  `v0.3.0`). `verify-version` 잡이 태그와 `Cargo.toml` 버전을 비교해 다르면
  빌드 전에 실패시킵니다.
- `--version`/`-V`는 `CARGO_PKG_VERSION`을 그대로 출력하므로, 버전을 올리지 않고
  태그하면 릴리스된 바이너리가 이전 버전을 보고하게 됩니다.
- `CHANGELOG.md`의 `Unreleased` 항목을 새 버전 섹션으로 옮기고 `YYYY-MM-DD` 날짜를
  적습니다. 같은 커밋에서 `Cargo.toml` 버전, `Cargo.lock`, 태그를 함께 맞춥니다.

## 릴리스 절차

```bash
# 1. 버전 올리기 (Cargo.toml) 후 lockfile 갱신
cargo build                     # Cargo.lock의 wpygen 버전이 갱신된다

# 2. 검증
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo test --test generated_projects -- --ignored   # uv/PyPI 필요

# 3. 커밋 후 태그 푸시 (태그 푸시가 release 워크플로우를 트리거한다)
git commit -am "chore(release): v0.3.0"
git push origin master
git tag v0.3.0
git push origin v0.3.0
```

## 아티팩트 명명 규칙

`wpygen_{os}_{arch}.{ext}` 형식을 유지합니다.

| 플랫폼 | runner | target | 자산 이름 |
| --- | --- | --- | --- |
| linux amd64 | `ubuntu-latest` | `x86_64-unknown-linux-musl` | `wpygen_linux_amd64.tar.gz` |
| darwin amd64 | `macos-15-intel` | `x86_64-apple-darwin` | `wpygen_darwin_amd64.tar.gz` |
| darwin arm64 | `macos-14` | `aarch64-apple-darwin` | `wpygen_darwin_arm64.tar.gz` |
| windows amd64 | `windows-latest` | `x86_64-pc-windows-msvc` | `wpygen_windows_amd64.zip` |

- 아카이브 안의 실행 파일 이름은 플랫폼과 무관하게 `wpygen`(`.zip`은 `wpygen.exe`)입니다.
- 모든 자산마다 `<자산 이름>.sha256`을 함께 업로드합니다.
- 체크섬은 `publish` 잡(ubuntu)에서 `sha256sum`으로 한 번만 만듭니다. macOS 러너에는
  `sha256sum`이 없으므로 빌드 잡에서 체크섬을 만들지 않습니다.

## 릴리스 잡 구조

```text
verify-version (태그 == Cargo.toml 버전 확인)
  └─ build (4개 플랫폼 병렬, 각자 아카이브를 아티팩트로 업로드)
       └─ publish (아티팩트 수집 → 필수 자산 확인 → 체크섬 → provenance → 릴리스 업로드)
```

- 릴리스 업로드는 `publish` 잡에서 **한 번만** 수행합니다. 빌드 잡이 각자 릴리스에
  업로드하면, 한 플랫폼이 실패했을 때 나머지 플랫폼만 담긴 릴리스가 공개될 수 있습니다.
- `publish`는 필수 자산 목록(플랫폼 4종)을 확인하고 하나라도 없으면 실패합니다.
  플랫폼을 의도적으로 빼는 경우 이 목록도 함께 수정해야 합니다.
- `publish`는 `contents: write` 외에 `id-token: write`, `attestations: write` 권한이
  필요합니다(빌드 출처 증명 생성). 그 외 잡은 읽기 권한만 씁니다.

## 지원 매트릭스와 CI

`.github/workflows/ci.yml`이 릴리스와 같은 4개 플랫폼에서 `cargo test --locked`를
실행합니다(linux amd64, darwin amd64, darwin arm64, windows amd64). 포맷·린트는
ubuntu에서 한 번만, 생성된 프로젝트 검증(`--ignored`, uv/PyPI 필요)도 ubuntu에서
한 번만 실행합니다.

플랫폼을 추가하거나 뺄 때는 다음을 함께 고칩니다.

1. `ci.yml`의 `test.strategy.matrix.os`
2. `release.yml`의 `build.strategy.matrix.include`
3. `release.yml` `publish`의 필수 자산 목록
4. `README.md`의 지원 플랫폼 표와 `CHANGELOG.md`

GitHub Actions 버전은 `@v7`처럼 floating major 태그로 쓰는 것을 기본으로 합니다.
`astral-sh/setup-uv`는 v8부터 immutable release만 발행해 floating major 태그가 없으므로
`@v10.1.0`처럼 정확한 버전을 고정합니다. 갱신은 `.github/dependabot.yml`이 만드는
PR로 처리합니다.

## 검증 방법

```bash
# 체크섬
shasum -a 256 -c wpygen_darwin_arm64.tar.gz.sha256   # macOS
sha256sum -c wpygen_linux_amd64.tar.gz.sha256        # Linux

# 빌드 출처 증명 (SLSA Build provenance)
gh attestation verify wpygen_darwin_arm64.tar.gz --repo wkqco33/wpygen
```

## Deprecation 정책

- 플래그·서브커맨드 변경은 additive하게 유지합니다. 제거하거나 의미를 바꿀 때는
  최소 한 개 MINOR 버전 동안 deprecation 경고(stderr)를 출력한 뒤 MAJOR(또는 `0.y.z`
  구간에서는 MINOR)에서 제거하고 `CHANGELOG.md`의 `Deprecated`/`Removed`에 기록합니다.
- `--json` 스키마는 키 추가만 허용합니다. 키 제거·이름 변경·타입 변경은 위 절차를
  따릅니다.
- `--plain`은 파일 경로를 한 줄에 하나씩 출력하는 계약이며, `--json`과 배타입니다.
- 종료 코드(0/1/2)는 계약이므로 바꾸지 않습니다. 새 오류가 생기면 기존 규칙(입력 오류
  2, 환경 오류 1)에 맞춰 분류합니다. 파서 오류는 wrcli의 분류를 따르므로 새 검증
  오류도 자동으로 2가 됩니다.

## ppm 설정

`ppm.json`의 다음 값을 유지합니다.

```json
{
  "description": "uv 기반 Python 프로젝트 템플릿 생성기",
  "author": "wkqco",
  "homepage": "https://github.com/wkqco33/wpygen",
  "bin_name": "wpygen"
}
```

- `bin_name`은 아카이브 안의 실행 파일 이름 및 `Cargo.toml`의 패키지 이름과 같아야
  합니다.
- `homepage`는 실제 릴리스가 올라가는 저장소를 가리켜야 합니다.
- 커밋하지 않는 것: `target/`, `.env`, 생성된 프로젝트 산출물.
