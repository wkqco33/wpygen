use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    InvalidPackageName(String),
    InvalidProjectName(String),
    TargetPathIsFile(PathBuf),
    TargetDirectoryNotEmpty(PathBuf),
    Io {
        path: PathBuf,
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
}

impl Error {
    /// 잘못된 인자/대상 상태로 인한 오류(사용자가 인자를 바꾸면 해결 가능)는 clap의
    /// usage error와 동일하게 2, 그 외 환경/시스템 오류(IO, 외부 명령 실패 등)는 1을 반환한다.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidPackageName(_)
            | Self::InvalidProjectName(_)
            | Self::TargetPathIsFile(_)
            | Self::TargetDirectoryNotEmpty(_) => 2,
            Self::Io { .. } | Self::RestoreFailed { .. } | Self::CommandFailed { .. } => 1,
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
            Self::CommandFailed { command, status } => match status {
                Some(code) => write!(f, "명령 실행 실패: {command} (exit code {code})"),
                None => write!(
                    f,
                    "명령 실행 실패: {command} (종료 코드 없음, 시그널로 종료됨)"
                ),
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::RestoreFailed { replace_source, .. } => Some(replace_source),
            _ => None,
        }
    }
}
