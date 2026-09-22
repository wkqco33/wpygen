use serde_json::{Value, json};
use std::time::Duration;

use crate::error::Error;
use crate::models::AiProvider;

const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// LLM 프롬프트 완성을 위한 추상 클라이언트 트레이트.
/// 단위 테스트 시 목(Mock) 주입을 위해 사용한다.
pub trait LlmClient {
    fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Error>;
}

/// HTTP 기반의 실제 LLM 호출 클라이언트.
pub struct HttpClient {
    pub provider: AiProvider,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout: Duration,
}

impl HttpClient {
    pub fn new(
        provider: AiProvider,
        endpoint: Option<String>,
        model: Option<String>,
        api_key: Option<String>,
    ) -> Result<Self, Error> {
        let endpoint = endpoint
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| provider.default_endpoint().to_string());

        let model = model
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| provider.default_model().to_string());

        if endpoint.is_empty() {
            return Err(Error::AiInvalidEndpoint(format!(
                "{:?} 프로바이더는 --endpoint 설정이 필요합니다.",
                provider.as_str()
            )));
        }

        if model.is_empty() {
            return Err(Error::AiInvalidEndpoint(format!(
                "{:?} 프로바이더는 --model 설정이 필요합니다.",
                provider.as_str()
            )));
        }

        // OpenAI 및 Azure는 API 키 필수 검증 (플래그 또는 환경 변수)
        let resolved_key = match provider {
            AiProvider::Ollama => api_key.or_else(|| std::env::var("OLLAMA_API_KEY").ok()),
            AiProvider::OpenAi => api_key
                .or_else(|| std::env::var("WPYGEN_AI_KEY").ok())
                .or_else(|| std::env::var("OPENAI_API_KEY").ok()),
            AiProvider::AzureOpenAi => api_key
                .or_else(|| std::env::var("AZURE_OPENAI_API_KEY").ok())
                .or_else(|| std::env::var("WPYGEN_AI_KEY").ok()),
        };

        if matches!(provider, AiProvider::OpenAi | AiProvider::AzureOpenAi)
            && resolved_key.as_deref().unwrap_or("").trim().is_empty()
        {
            return Err(Error::AiMissingApiKey(provider.as_str().to_string()));
        }

        Ok(Self {
            provider,
            endpoint,
            model,
            api_key: resolved_key,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        })
    }
}

impl LlmClient for HttpClient {
    fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Error> {
        match self.provider {
            AiProvider::Ollama => self.call_ollama(system_prompt, user_prompt),
            AiProvider::OpenAi => self.call_openai(system_prompt, user_prompt),
            AiProvider::AzureOpenAi => self.call_azure(system_prompt, user_prompt),
        }
    }
}

impl HttpClient {
    fn call_ollama(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Error> {
        let base = self.endpoint.trim_end_matches('/');
        let url = if base.ends_with("/api/chat") {
            base.to_string()
        } else {
            format!("{base}/api/chat")
        };

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "stream": false,
            "format": "json"
        });

        let mut req = ureq::post(&url).timeout(self.timeout);
        if let Some(key) = &self.api_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }

        let resp = req.send_json(body).map_err(|e| match e {
            ureq::Error::Status(status, resp) => {
                let text = resp.into_string().unwrap_or_default();
                Error::AiHttpFailure {
                    status,
                    message: text,
                }
            }
            ureq::Error::Transport(t) => Error::AiHttpFailure {
                status: 0,
                message: t.to_string(),
            },
        })?;

        let val: Value = resp
            .into_json()
            .map_err(|e| Error::AiInvalidResponse(format!("Ollama 응답 파싱 실패: {e}")))?;

        val.pointer("/message/content")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                Error::AiInvalidResponse("Ollama 응답에서 content를 찾을 수 없습니다.".to_string())
            })
    }

    fn call_openai(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Error> {
        let base = self.endpoint.trim_end_matches('/');
        let url = if base.ends_with("/chat/completions") {
            base.to_string()
        } else {
            format!("{base}/chat/completions")
        };

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": { "type": "json_object" }
        });

        let key = self.api_key.as_deref().unwrap_or_default();
        let resp = ureq::post(&url)
            .timeout(self.timeout)
            .set("Authorization", &format!("Bearer {key}"))
            .send_json(body)
            .map_err(|e| match e {
                ureq::Error::Status(status, resp) => {
                    let text = resp.into_string().unwrap_or_default();
                    Error::AiHttpFailure {
                        status,
                        message: text,
                    }
                }
                ureq::Error::Transport(t) => Error::AiHttpFailure {
                    status: 0,
                    message: t.to_string(),
                },
            })?;

        let val: Value = resp
            .into_json()
            .map_err(|e| Error::AiInvalidResponse(format!("OpenAI 응답 파싱 실패: {e}")))?;

        val.pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                Error::AiInvalidResponse("OpenAI 응답에서 content를 찾을 수 없습니다.".to_string())
            })
    }

    fn call_azure(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Error> {
        let base = self.endpoint.trim_end_matches('/');
        let url = if base.contains("/openai/deployments/") {
            base.to_string()
        } else {
            format!(
                "{base}/openai/deployments/{}/chat/completions?api-version=2024-02-15-preview",
                self.model
            )
        };

        let body = json!({
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": { "type": "json_object" }
        });

        let key = self.api_key.as_deref().unwrap_or_default();
        let resp = ureq::post(&url)
            .timeout(self.timeout)
            .set("api-key", key)
            .send_json(body)
            .map_err(|e| match e {
                ureq::Error::Status(status, resp) => {
                    let text = resp.into_string().unwrap_or_default();
                    Error::AiHttpFailure {
                        status,
                        message: text,
                    }
                }
                ureq::Error::Transport(t) => Error::AiHttpFailure {
                    status: 0,
                    message: t.to_string(),
                },
            })?;

        let val: Value = resp
            .into_json()
            .map_err(|e| Error::AiInvalidResponse(format!("Azure OpenAI 응답 파싱 실패: {e}")))?;

        val.pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                Error::AiInvalidResponse(
                    "Azure OpenAI 응답에서 content를 찾을 수 없습니다.".to_string(),
                )
            })
    }
}

/// 단위 테스트를 위한 Mock LLM 클라이언트.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockClient {
    pub response: Result<String, String>,
}

#[cfg(test)]
impl MockClient {
    pub fn success(json_content: impl Into<String>) -> Self {
        Self {
            response: Ok(json_content.into()),
        }
    }

    pub fn failure(err: impl Into<String>) -> Self {
        Self {
            response: Err(err.into()),
        }
    }
}

#[cfg(test)]
impl LlmClient for MockClient {
    fn complete(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, Error> {
        match &self.response {
            Ok(res) => Ok(res.clone()),
            Err(err) => Err(Error::AiInvalidResponse(err.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_client_initialization_defaults() {
        let client = HttpClient::new(AiProvider::Ollama, None, None, None).unwrap();
        assert_eq!(client.endpoint, "http://localhost:11434");
        assert_eq!(client.model, "llama3.2");
        assert!(client.api_key.is_none());
    }

    #[test]
    fn openai_client_requires_api_key_when_empty() {
        let res = HttpClient::new(AiProvider::OpenAi, None, None, Some("".to_string()));
        // Note: 환경 변수가 없을 때 에러
        if std::env::var("OPENAI_API_KEY").is_err() && std::env::var("WPYGEN_AI_KEY").is_err() {
            assert!(matches!(res, Err(Error::AiMissingApiKey(_))));
        }
    }

    #[test]
    fn azure_client_requires_endpoint() {
        let res = HttpClient::new(
            AiProvider::AzureOpenAi,
            Some("".to_string()),
            Some("my-deploy".into()),
            Some("key".into()),
        );
        assert!(matches!(res, Err(Error::AiInvalidEndpoint(_))));
    }

    #[test]
    fn mock_client_returns_configured_response() {
        let mock = MockClient::success(r#"{"files": []}"#);
        assert_eq!(mock.complete("sys", "usr").unwrap(), r#"{"files": []}"#);

        let mock_err = MockClient::failure("network timeout");
        assert!(mock_err.complete("sys", "usr").is_err());
    }
}
