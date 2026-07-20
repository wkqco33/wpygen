use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Error;
use crate::models::{GeneratedFile, ProjectSpec};
use crate::templates;

pub fn create_project(
    target_dir: &Path,
    spec: &ProjectSpec,
    force: bool,
    verbose: bool,
) -> Result<usize, Error> {
    validate_target_dir(target_dir, force)?;

    let files = templates::render_project(spec);
    let staging_dir = create_staging_dir(target_dir)?;
    if verbose {
        eprintln!("스테이징 디렉터리 생성: {}", staging_dir.display());
    }

    if let Err(err) = write_files(&staging_dir, &files, verbose) {
        let _ = fs::remove_dir_all(&staging_dir);
        return Err(err);
    }

    if verbose {
        eprintln!("대상 디렉터리로 교체 중: {}", target_dir.display());
    }
    if let Err(err) = replace_target_dir(target_dir, &staging_dir) {
        let _ = fs::remove_dir_all(&staging_dir);
        return Err(err);
    }

    Ok(files.len())
}

/// 실제로 쓰지 않고, 생성될 파일의 상대 경로 목록만 돌려준다. 대상 디렉터리 검증은
/// `create_project`와 동일하게 수행하므로 `--force` 없이 비어있지 않은 디렉터리를
/// 대상으로 하면 여기서도 동일하게 실패한다.
pub fn preview(target_dir: &Path, spec: &ProjectSpec, force: bool) -> Result<Vec<PathBuf>, Error> {
    validate_target_dir(target_dir, force)?;
    Ok(templates::render_project(spec)
        .into_iter()
        .map(|file| file.relative_path)
        .collect())
}

/// 대상 경로가 파일이 아니고, 비어있거나 `force`가 지정됐는지 확인한다.
/// 실제 쓰기는 스테이징 디렉터리에서 이뤄지므로 여기서는 검증만 한다.
fn validate_target_dir(target_dir: &Path, force: bool) -> Result<(), Error> {
    if !target_dir.exists() {
        return Ok(());
    }
    if !target_dir.is_dir() {
        return Err(Error::TargetPathIsFile(target_dir.to_path_buf()));
    }

    let is_empty = fs::read_dir(target_dir)
        .map_err(io_err(target_dir))?
        .next()
        .is_none();
    if !is_empty && !force {
        return Err(Error::TargetDirectoryNotEmpty(target_dir.to_path_buf()));
    }
    Ok(())
}

/// `target_dir`와 같은 부모 디렉터리 아래에 임시 스테이징 디렉터리를 만든다.
/// 같은 부모 아래 두는 이유는 이후 `fs::rename`으로 같은 파일시스템 내에서
/// 원자적으로 옮기기 위함이다.
fn create_staging_dir(target_dir: &Path) -> Result<PathBuf, Error> {
    let parent = match target_dir.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    fs::create_dir_all(parent).map_err(io_err(parent))?;

    let name = target_dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string());
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let staging_dir = parent.join(format!(".{name}.wpygen-tmp-{unique}"));

    fs::create_dir_all(&staging_dir).map_err(io_err(&staging_dir))?;
    Ok(staging_dir)
}

/// 렌더링된 파일을 전부 스테이징 디렉터리에 쓴다. 도중에 실패하면 이미 쓰인
/// 파일들은 스테이징 디렉터리 안에만 존재하므로 대상 디렉터리는 오염되지 않는다.
fn write_files(staging_dir: &Path, files: &[GeneratedFile], verbose: bool) -> Result<(), Error> {
    for file in files {
        if verbose {
            eprintln!("  쓰는 중: {}", file.relative_path.display());
        }
        let path = staging_dir.join(&file.relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_err(parent))?;
        }
        fs::write(&path, &file.contents).map_err(io_err(&path))?;
    }
    Ok(())
}

/// 스테이징 디렉터리를 최종 대상 경로로 옮긴다. 대상이 이미 존재하면(=`force`로
/// 허용된 경우) 통째로 지운 뒤 옮겨서, 이전 생성 결과의 잔존 파일이 섞이지 않게 한다.
fn replace_target_dir(target_dir: &Path, staging_dir: &Path) -> Result<(), Error> {
    if target_dir.exists() {
        fs::remove_dir_all(target_dir).map_err(io_err(target_dir))?;
    }
    fs::rename(staging_dir, target_dir).map_err(io_err(target_dir))
}

fn io_err(path: &Path) -> impl FnOnce(io::Error) -> Error {
    let path = path.to_path_buf();
    move |source| Error::Io { path, source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TemplateKind;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("wpygen-writer-{name}-{suffix}"))
    }

    fn cli_spec() -> ProjectSpec {
        ProjectSpec {
            project_name: "demo-app".to_string(),
            package_name: "demo_app".to_string(),
            template: TemplateKind::Cli,
            grpc: false,
            sqlite: false,
            index_url: "https://pypi.wkqcosoft.cloud".to_string(),
        }
    }

    #[test]
    fn rejects_when_target_is_a_file() {
        let target_dir = unique_temp_dir("target-is-file");
        fs::write(&target_dir, b"not a directory").unwrap();

        let result = create_project(&target_dir, &cli_spec(), false, false);

        assert!(matches!(result, Err(Error::TargetPathIsFile(_))));
        let _ = fs::remove_file(&target_dir);
    }

    #[test]
    fn rejects_nonempty_target_without_force() {
        let target_dir = unique_temp_dir("nonempty");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("existing.txt"), b"keep me").unwrap();

        let result = create_project(&target_dir, &cli_spec(), false, false);

        assert!(matches!(result, Err(Error::TargetDirectoryNotEmpty(_))));
        assert!(target_dir.join("existing.txt").exists());
        let _ = fs::remove_dir_all(&target_dir);
    }

    #[test]
    fn force_replaces_stale_files_from_previous_generation() {
        let target_dir = unique_temp_dir("force-replace");
        fs::create_dir_all(&target_dir).unwrap();
        // 이전 세대에서 남은, 이번 스펙에는 없는 파일(예: sqlite 없이 재생성)
        fs::write(target_dir.join("stale_database.py"), b"stale").unwrap();

        let file_count = create_project(&target_dir, &cli_spec(), true, false).unwrap();

        assert!(file_count > 0);
        assert!(!target_dir.join("stale_database.py").exists());
        assert!(target_dir.join("pyproject.toml").exists());
        let _ = fs::remove_dir_all(&target_dir);
    }

    #[test]
    fn failure_does_not_leave_partial_files_at_target() {
        // target_dir의 부모를 (디렉터리가 아닌) 파일로 만들어서 스테이징 디렉터리
        // 생성 자체가 실패하도록 유도한다. 이 경우 target_dir는 절대 생성되면 안 된다.
        let bogus_parent = unique_temp_dir("rollback-parent");
        fs::write(&bogus_parent, b"i am a file, not a directory").unwrap();
        let target_under_file = bogus_parent.join("child-project");

        let result = create_project(&target_under_file, &cli_spec(), false, false);

        assert!(result.is_err());
        assert!(!target_under_file.exists());
        let _ = fs::remove_file(&bogus_parent);
    }
}
