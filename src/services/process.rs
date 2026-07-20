use std::path::Path;
use std::process::Command;

use crate::error::Error;

/// `dir`를 작업 디렉터리로 삼아 외부 명령을 실행한다. 명령을 아예 실행할 수 없으면
/// (예: 실행 파일이 없음) `Error::Io`로, 실행은 됐지만 실패 종료 코드를 반환하면
/// `Error::CommandFailed`로 매핑한다.
pub fn run(dir: &Path, program: &str, args: &[&str]) -> Result<(), Error> {
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|source| Error::Io {
            path: dir.to_path_buf(),
            source,
        })?;

    if !status.success() {
        return Err(Error::CommandFailed {
            command: format!("{program} {}", args.join(" ")),
            status: status.code(),
        });
    }

    Ok(())
}
