use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::Error;

/// 자식 프로세스의 표준 출력을 어떻게 다룰지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildOutput {
    /// 부모의 stdout/stderr를 그대로 물려준다.
    Inherit,
    /// 자식의 stdout을 부모의 stderr로 돌린다. 부모 stdout을 기계 판독용으로
    /// 유지해야 할 때(`--json`) 쓴다.
    RedirectStdoutToStderr,
}

/// `dir`를 작업 디렉터리로 삼아 외부 명령을 실행한다. 명령을 아예 실행할 수 없으면
/// (예: 실행 파일이 없음) `Error::CommandSpawnFailed`로, 실행은 됐지만 실패 종료
/// 코드를 반환하면 `Error::CommandFailed`로 매핑한다.
pub fn run(dir: &Path, program: &str, args: &[&str], output: ChildOutput) -> Result<(), Error> {
    let command = format!("{program} {}", args.join(" "));
    let mut child = Command::new(program);
    child.args(args).current_dir(dir);

    let status = match output {
        ChildOutput::Inherit => child.status(),
        ChildOutput::RedirectStdoutToStderr => {
            child.stdout(Stdio::piped()).output().map(|captured| {
                forward_to_stderr(&captured.stdout);
                forward_to_stderr(&captured.stderr);
                captured.status
            })
        }
    }
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

/// 자식 프로세스의 출력을 stderr로 전달한다. 이미 닫힌 파이프 등으로 실패하면
/// 무시한다(출력 전달 실패로 명령 자체의 결과를 바꾸지 않는다).
fn forward_to_stderr(bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    let _ = io::stderr().write_all(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unique_temp_dir;

    #[test]
    fn missing_program_mentions_the_program_name() {
        let dir = unique_temp_dir("process-missing-program");
        std::fs::create_dir_all(&dir).unwrap();

        let error = run(
            &dir,
            "wpygen-definitely-missing-program",
            &["--version"],
            ChildOutput::Inherit,
        )
        .unwrap_err();

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
    fn redirected_output_still_reports_success() {
        let dir = unique_temp_dir("process-redirect");
        std::fs::create_dir_all(&dir).unwrap();

        let result = run(
            &dir,
            "sh",
            &["-c", "echo noise; exit 0"],
            ChildOutput::RedirectStdoutToStderr,
        );

        assert!(result.is_ok(), "{result:?}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn failing_program_reports_its_exit_code() {
        let dir = unique_temp_dir("process-failing-program");
        std::fs::create_dir_all(&dir).unwrap();

        let error = run(&dir, "sh", &["-c", "exit 3"], ChildOutput::Inherit).unwrap_err();

        let message = error.to_string();
        assert!(message.contains("exit code 3"), "{message}");
        assert!(message.contains("sh -c exit 3"), "{message}");
        assert_eq!(error.exit_code(), 1);

        let _ = std::fs::remove_dir_all(dir);
    }
}
