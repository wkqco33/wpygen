use wrcli::OutputFormat;

use crate::error::Error;
use crate::services::config;

pub fn init(force: bool, quiet: bool) -> Result<(), Error> {
    let path = config::init(force)?;
    if !quiet {
        eprintln!("설정 파일이 생성되었습니다: {}", path.display());
    }
    Ok(())
}

pub fn show(format: OutputFormat) -> Result<(), Error> {
    let app_config = config::load()?;
    match format {
        OutputFormat::Json => {
            let json_str = serde_json::to_string_pretty(&app_config)
                .map_err(|e| Error::ConfigParseFailure(format!("JSON 변환 실패: {e}")))?;
            println!("{json_str}");
        }
        _ => {
            let toml_str = app_config.to_toml_string()?;
            print!("{toml_str}");
        }
    }
    Ok(())
}

pub fn path() -> Result<(), Error> {
    let path = config::config_path()?;
    println!("{}", path.display());
    Ok(())
}

pub fn get(key: &str) -> Result<(), Error> {
    let app_config = config::load()?;
    let value = app_config.get_value(key)?;
    println!("{value}");
    Ok(())
}

pub fn set(key: &str, value: &str, quiet: bool) -> Result<(), Error> {
    let mut app_config = config::load()?;
    app_config.set_value(key, value)?;
    config::save(&app_config)?;
    if !quiet {
        eprintln!("설정값이 저장되었습니다: {key} = {value}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unique_temp_dir;
    use std::fs;

    #[test]
    fn config_init_show_set_get_flow() {
        let temp = unique_temp_dir("cmds-config");
        let config_file = temp.join("wpygen/config.toml");

        unsafe {
            std::env::set_var("WPYGEN_CONFIG_PATH", &config_file);
        }

        // 1. init
        init(false, true).unwrap();
        assert!(config_file.exists());

        // 2. set
        set("ai.provider", "openai", true).unwrap();
        set("ai.model", "gpt-4o", true).unwrap();

        // 3. get
        let app_config = config::load().unwrap();
        assert_eq!(app_config.get_value("ai.provider").unwrap(), "openai");
        assert_eq!(app_config.get_value("ai.model").unwrap(), "gpt-4o");

        unsafe {
            std::env::remove_var("WPYGEN_CONFIG_PATH");
        }
        let _ = fs::remove_dir_all(temp);
    }
}
