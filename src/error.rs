use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    InvalidPackageName(String),
    InvalidProjectName(String),
    InvalidTemplate(String),
    ConflictingFlags,
    TargetPathIsFile(PathBuf),
    TargetDirectoryNotEmpty(PathBuf),
    Io {
        path: PathBuf,
        source: io::Error,
    },
    CommandSpawnFailed {
        command: String,
        source: io::Error,
    },
    RestoreFailed {
        target: PathBuf,
        replace_source: io::Error,
        restore_source: io::Error,
    },
    CommandFailed {
        command: String,
        status: Option<i32>,
    },
    InvalidAiProvider(String),
    AiMissingApiKey(String),
    AiInvalidEndpoint(String),
    PathTraversalViolation(PathBuf),
    EmptyPrompt,
    AiHttpFailure {
        status: u16,
        message: String,
    },
    AiInvalidResponse(String),
    InvalidConfigKey(String),
    ConfigFileAlreadyExists(PathBuf),
    ConfigHomeNotFound,
    ConfigParseFailure(String),
}

impl Error {
    /// 잘못된 인자/대상 상태로 인한 오류(사용자가 인자를 바꾸면 해결 가능)는 clap의
    /// usage error와 동일하게 2, 그 외 환경/시스템 오류(IO, 외부 명령 실패 등)는 1을 반환한다.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidPackageName(_)
            | Self::InvalidProjectName(_)
            | Self::InvalidTemplate(_)
            | Self::ConflictingFlags
            | Self::TargetPathIsFile(_)
            | Self::TargetDirectoryNotEmpty(_)
            | Self::InvalidAiProvider(_)
            | Self::AiMissingApiKey(_)
            | Self::AiInvalidEndpoint(_)
            | Self::PathTraversalViolation(_)
            | Self::EmptyPrompt
            | Self::InvalidConfigKey(_)
            | Self::ConfigFileAlreadyExists(_) => 2,
            Self::Io { .. }
            | Self::RestoreFailed { .. }
            | Self::CommandSpawnFailed { .. }
            | Self::CommandFailed { .. }
            | Self::AiHttpFailure { .. }
            | Self::AiInvalidResponse(_)
            | Self::ConfigHomeNotFound
            | Self::ConfigParseFailure(_) => 1,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPackageName(name) => write!(
                f,
                "유효하지 않은 Python 패키지명입니다: {name:?} (영문자/숫자/언더스코어만 허용, 첫 글자는 숫자 불가)"
            ),
            Self::InvalidProjectName(name) => write!(
                f,
                "유효하지 않은 프로젝트 이름입니다: {name:?} (영문자/숫자와 '-', '_', '.'만 허용하며, \
                 첫 글자와 마지막 글자는 영문자/숫자여야 합니다. pyproject.toml의 배포 패키지명으로도 \
                 그대로 쓰이기 때문입니다)"
            ),
            Self::InvalidTemplate(name) => write!(
                f,
                "유효하지 않은 템플릿 종류입니다: {name:?} (허용: cli, gui, server)"
            ),
            Self::ConflictingFlags => write!(
                f,
                "--dry-run 은 --git / --sync / --lock 와 함께 사용할 수 없습니다."
            ),
            Self::TargetPathIsFile(path) => {
                write!(
                    f,
                    "대상 경로가 디렉터리가 아니라 파일입니다: {}",
                    path.display()
                )
            }
            Self::TargetDirectoryNotEmpty(path) => write!(
                f,
                "대상 디렉터리가 이미 존재하고 비어있지 않습니다: {} (--force 로 덮어쓰기 가능)",
                path.display()
            ),
            Self::Io { path, source } => {
                write!(f, "{}: {}", path.display(), source)
            }
            Self::RestoreFailed {
                target,
                replace_source,
                restore_source,
            } => write!(
                f,
                "대상 디렉터리 교체에 실패했고 원래 디렉터리 복원도 실패했습니다: {} \
                 (교체 오류: {}; 복원 오류: {})",
                target.display(),
                replace_source,
                restore_source
            ),
            Self::CommandSpawnFailed { command, source } => {
                write!(f, "명령을 실행할 수 없습니다: {command} ({source})")
            }
            Self::CommandFailed { command, status } => match status {
                Some(code) => write!(f, "명령 실행 실패: {command} (exit code {code})"),
                None => write!(
                    f,
                    "명령 실행 실패: {command} (종료 코드 없음, 시그널로 종료됨)"
                ),
            },
            Self::InvalidAiProvider(provider) => write!(
                f,
                "지원하지 않는 AI 프로바이더입니다: {provider:?} (허용: ollama, openai, azure-openai)"
            ),
            Self::AiMissingApiKey(provider) => write!(
                f,
                "{provider} 프로바이더 사용을 위한 API 키가 설정되지 않았습니다. --api-key 플래그 또는 환경 변수를 설정하세요."
            ),
            Self::AiInvalidEndpoint(msg) => {
                write!(f, "AI 엔드포인트 URL이 올바르지 않습니다: {msg}")
            }
            Self::PathTraversalViolation(path) => {
                write!(
                    f,
                    "보안 위반: AI가 생성한 경로가 허용된 범위를 벗어납니다: {}",
                    path.display()
                )
            }
            Self::EmptyPrompt => write!(f, "AI 프롬프트가 비어있습니다. 프롬프트를 입력하세요."),
            Self::AiHttpFailure { status, message } => {
                write!(f, "AI API 요청 실패 (HTTP {status}): {message}")
            }
            Self::AiInvalidResponse(msg) => {
                write!(f, "AI 응답 파싱 실패: {msg}")
            }
            Self::InvalidConfigKey(key) => write!(
                f,
                "유효하지 않은 설정 키입니다: {key:?} (허용 키: ai.provider, ai.model, ai.endpoint, ai.api_key, defaults.output)"
            ),
            Self::ConfigFileAlreadyExists(path) => write!(
                f,
                "설정 파일이 이미 존재합니다: {} (--force 로 덮어쓰기 가능)",
                path.display()
            ),
            Self::ConfigHomeNotFound => write!(
                f,
                "사용자 설정 디렉터리를 찾을 수 없습니다 ($HOME 또는 $XDG_CONFIG_HOME 확인 필요)"
            ),
            Self::ConfigParseFailure(msg) => {
                write!(f, "설정 파일 파싱 실패: {msg}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::CommandSpawnFailed { source, .. } => Some(source),
            Self::RestoreFailed { replace_source, .. } => Some(replace_source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_user_errors_exit_with_code_2() {
        assert_eq!(Error::InvalidAiProvider("unknown".into()).exit_code(), 2);
        assert_eq!(Error::AiMissingApiKey("openai".into()).exit_code(), 2);
        assert_eq!(
            Error::AiInvalidEndpoint("invalid url".into()).exit_code(),
            2
        );
        assert_eq!(
            Error::PathTraversalViolation(PathBuf::from("../bad")).exit_code(),
            2
        );
        assert_eq!(Error::EmptyPrompt.exit_code(), 2);
        assert_eq!(Error::InvalidConfigKey("bad.key".into()).exit_code(), 2);
        assert_eq!(
            Error::ConfigFileAlreadyExists(PathBuf::from("config.toml")).exit_code(),
            2
        );
    }

    #[test]
    fn ai_system_errors_exit_with_code_1() {
        assert_eq!(
            Error::AiHttpFailure {
                status: 500,
                message: "Internal error".into(),
            }
            .exit_code(),
            1
        );
        assert_eq!(Error::AiInvalidResponse("json error".into()).exit_code(), 1);
        assert_eq!(Error::ConfigHomeNotFound.exit_code(), 1);
        assert_eq!(
            Error::ConfigParseFailure("parse error".into()).exit_code(),
            1
        );
    }
}
