use std::path::Path;
use std::process::Command;

use crate::error::Error;

/// `dir`를 작업 디렉터리로 삼아 외부 명령을 실행한다. 명령을 아예 실행할 수 없으면
/// (예: 실행 파일이 없음) `Error::CommandSpawnFailed`로, 실행은 됐지만 실패 종료
/// 코드를 반환하면 `Error::CommandFailed`로 매핑한다.
pub fn run(dir: &Path, program: &str, args: &[&str]) -> Result<(), Error> {
    let command = format!("{program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|source| Error::CommandSpawnFailed {
            command: command.clone(),
            source,
        })?;

    if !status.success() {
        return Err(Error::CommandFailed {
            command,
            status: status.code(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unique_temp_dir;

    #[test]
    fn missing_program_mentions_the_program_name() {
        let dir = unique_temp_dir("process-missing-program");
        std::fs::create_dir_all(&dir).unwrap();

        let error = run(&dir, "wpygen-definitely-missing-program", &["--version"]).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("wpygen-definitely-missing-program"),
            "실행 실패 메시지에 프로그램 이름이 포함되어야 한다: {error}"
        );
        assert_eq!(error.exit_code(), 1);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn failing_program_reports_its_exit_code() {
        let dir = unique_temp_dir("process-failing-program");
        std::fs::create_dir_all(&dir).unwrap();

        let error = run(&dir, "sh", &["-c", "exit 3"]).unwrap_err();

        let message = error.to_string();
        assert!(message.contains("exit code 3"), "{message}");
        assert!(message.contains("sh -c exit 3"), "{message}");
        assert_eq!(error.exit_code(), 1);

        let _ = std::fs::remove_dir_all(dir);
    }
}
