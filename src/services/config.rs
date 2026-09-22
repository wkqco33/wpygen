use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::models::AppConfig;

pub const CONFIG_FILE_NAME: &str = "config.toml";

/// wpygen의 설정 파일 경로를 결정한다.
/// 1. `WPYGEN_CONFIG_PATH` 환경 변수 (테스트 및 사용자 지정 우선)
/// 2. OS별 표준 설정 디렉터리 하위 `wpygen/config.toml`
pub fn config_path() -> Result<PathBuf, Error> {
    if let Ok(custom) = std::env::var("WPYGEN_CONFIG_PATH")
        && !custom.trim().is_empty()
    {
        return Ok(PathBuf::from(custom));
    }

    dirs::config_dir()
        .map(|dir| dir.join("wpygen").join(CONFIG_FILE_NAME))
        .ok_or(Error::ConfigHomeNotFound)
}

/// 설정 파일을 로드한다. 파일이 없으면 기본 `AppConfig::default()`를 반환한다.
pub fn load() -> Result<AppConfig, Error> {
    let path = config_path()?;
    load_from_path(&path)
}

/// 특정 경로에서 설정을 로드한다.
pub fn load_from_path(path: &Path) -> Result<AppConfig, Error> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    toml::from_str(&content)
        .map_err(|e| Error::ConfigParseFailure(format!("{}: {e}", path.display())))
}

/// 현재 설정을 설정 파일에 저장한다.
pub fn save(config: &AppConfig) -> Result<PathBuf, Error> {
    let path = config_path()?;
    save_to_path(&path, config)?;
    Ok(path)
}

/// 특정 경로에 설정을 저장한다.
pub fn save_to_path(path: &Path, config: &AppConfig) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    let toml_str = config.to_toml_string()?;
    fs::write(path, toml_str).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// 기본 템플릿으로 설정 파일을 초기화한다.
pub fn init(force: bool) -> Result<PathBuf, Error> {
    let path = config_path()?;
    if path.exists() && !force {
        return Err(Error::ConfigFileAlreadyExists(path));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    fs::write(&path, AppConfig::default_template()).map_err(|e| Error::Io {
        path: path.clone(),
        source: e,
    })?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unique_temp_dir;

    #[test]
    fn loads_default_when_file_not_found() {
        let temp = unique_temp_dir("config-not-found");
        let path = temp.join("nonexistent.toml");
        let config = load_from_path(&path).unwrap();
        assert_eq!(config, AppConfig::default());
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn saves_and_loads_roundtrip() {
        let temp = unique_temp_dir("config-roundtrip");
        let path = temp.join("sub/config.toml");

        let mut config = AppConfig::default();
        config.set_value("ai.provider", "openai").unwrap();
        config.set_value("ai.model", "gpt-4o").unwrap();

        save_to_path(&path, &config).unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.ai.provider.as_deref(), Some("openai"));
        assert_eq!(loaded.ai.model.as_deref(), Some("gpt-4o"));

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn init_creates_template_and_rejects_overwrite_without_force() {
        let _guard = crate::testing::ENV_MUTEX.lock().unwrap();
        let temp = unique_temp_dir("config-init");
        let path = temp.join("wpygen/config.toml");

        // 환경 변수로 경로 임시 주입
        // SAFETY: 테스트 내부 격리
        unsafe {
            std::env::set_var("WPYGEN_CONFIG_PATH", &path);
        }

        let initialized_path = init(false).unwrap();
        assert_eq!(initialized_path, path);
        assert!(path.exists());

        // force 없이 재호출 시 에러
        let err = init(false).unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(matches!(err, Error::ConfigFileAlreadyExists(_)));

        // force 지정 시 성공
        assert!(init(true).is_ok());

        unsafe {
            std::env::remove_var("WPYGEN_CONFIG_PATH");
        }
        let _ = fs::remove_dir_all(temp);
    }
}
