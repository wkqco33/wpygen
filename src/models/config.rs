use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::models::AiProvider;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefaultsConfig {
    pub output: Option<String>,
}

impl AppConfig {
    /// 기본 설정 파일 템플릿(주석 포함).
    pub fn default_template() -> &'static str {
        r#"# wpygen 설정 파일
# 플랫폼별 기본 경로에 저장되어 프로젝트 생성 시 기본값으로 적용됩니다.

[ai]
# AI 프로바이더: ollama, openai, azure-openai (기본값: ollama)
provider = "ollama"

# AI 모델명 (기본값: ollama는 llama3.2, openai는 gpt-4o-mini)
model = "llama3.2"

# AI API 엔드포인트 URL (기본값: ollama는 http://localhost:11434, openai는 https://api.openai.com/v1)
# endpoint = "http://localhost:11434"

# AI API 인증 키 (OpenAI 및 Azure OpenAI 사용 시 필요, 환경 변수로도 설정 가능)
# api_key = ""

[defaults]
# 프로젝트 생성 기본 상위 디렉터리 (기본값: ".")
# output = "."
"#
    }

    /// 점 표기법(key path)으로 설정값을 문자열 형태로 조회한다.
    pub fn get_value(&self, key: &str) -> Result<String, Error> {
        match key {
            "ai.provider" => Ok(self.ai.provider.clone().unwrap_or_default()),
            "ai.model" => Ok(self.ai.model.clone().unwrap_or_default()),
            "ai.endpoint" => Ok(self.ai.endpoint.clone().unwrap_or_default()),
            "ai.api_key" => Ok(self.ai.api_key.clone().unwrap_or_default()),
            "defaults.output" => Ok(self.defaults.output.clone().unwrap_or_default()),
            _ => Err(Error::InvalidConfigKey(key.to_string())),
        }
    }

    /// 점 표기법(key path)으로 설정값을 수정한다.
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<(), Error> {
        let trimmed = value.trim();
        let val_opt = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };

        match key {
            "ai.provider" => {
                if let Some(ref p) = val_opt {
                    AiProvider::from_str(p)?;
                }
                self.ai.provider = val_opt;
            }
            "ai.model" => self.ai.model = val_opt,
            "ai.endpoint" => self.ai.endpoint = val_opt,
            "ai.api_key" => self.ai.api_key = val_opt,
            "defaults.output" => self.defaults.output = val_opt,
            _ => return Err(Error::InvalidConfigKey(key.to_string())),
        }

        Ok(())
    }

    pub fn to_toml_string(&self) -> Result<String, Error> {
        toml::to_string_pretty(self)
            .map_err(|e| Error::ConfigParseFailure(format!("TOML 직렬화 실패: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_template() {
        let template = AppConfig::default_template();
        let config: AppConfig = toml::from_str(template).unwrap();
        assert_eq!(config.ai.provider.as_deref(), Some("ollama"));
        assert_eq!(config.ai.model.as_deref(), Some("llama3.2"));
    }

    #[test]
    fn gets_and_sets_values() {
        let mut config = AppConfig::default();
        assert_eq!(config.get_value("ai.provider").unwrap(), "");

        config.set_value("ai.provider", "openai").unwrap();
        assert_eq!(config.get_value("ai.provider").unwrap(), "openai");

        config.set_value("ai.model", "gpt-4o").unwrap();
        assert_eq!(config.get_value("ai.model").unwrap(), "gpt-4o");

        config
            .set_value("defaults.output", "/tmp/projects")
            .unwrap();
        assert_eq!(
            config.get_value("defaults.output").unwrap(),
            "/tmp/projects"
        );
    }

    #[test]
    fn rejects_invalid_provider_in_set() {
        let mut config = AppConfig::default();
        let err = config
            .set_value("ai.provider", "unsupported_ai")
            .unwrap_err();
        assert!(matches!(err, Error::InvalidAiProvider(_)));
    }

    #[test]
    fn rejects_unknown_key() {
        let mut config = AppConfig::default();
        let err = config.set_value("unknown.key", "value").unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(matches!(err, Error::InvalidConfigKey(_)));

        assert!(config.get_value("unknown.key").is_err());
    }
}
