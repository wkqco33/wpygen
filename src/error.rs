use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    InvalidPackageName(String),
    TargetPathIsFile(PathBuf),
    TargetDirectoryNotEmpty(PathBuf),
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPackageName(name) => write!(
                f,
                "유효하지 않은 Python 패키지명입니다: {name:?} (영문자/숫자/언더스코어만 허용, 첫 글자는 숫자 불가)"
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
        }
    }
}

impl std::error::Error for Error {}
