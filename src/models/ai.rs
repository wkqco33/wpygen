use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AiProvider {
    Ollama,
    OpenAi,
    AzureOpenAi,
}

impl AiProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OpenAi => "openai",
            Self::AzureOpenAi => "azure-openai",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_ascii_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai" => Ok(Self::OpenAi),
            "azure-openai" | "azure_openai" | "azure" => Ok(Self::AzureOpenAi),
            other => Err(Error::InvalidAiProvider(other.to_string())),
        }
    }

    pub fn default_endpoint(self) -> &'static str {
        match self {
            Self::Ollama => "http://localhost:11434",
            Self::OpenAi => "https://api.openai.com/v1",
            Self::AzureOpenAi => "",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::Ollama => "llama3.2",
            Self::OpenAi => "gpt-4o-mini",
            Self::AzureOpenAi => "",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiFileEntry {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiProjectPlan {
    #[serde(default)]
    pub project_name: String,
    #[serde(default)]
    pub package_name: String,
    #[serde(default)]
    pub description: String,
    pub files: Vec<AiFileEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_provider_strings() {
        assert_eq!(AiProvider::from_str("ollama").unwrap(), AiProvider::Ollama);
        assert_eq!(AiProvider::from_str("OLLAMA").unwrap(), AiProvider::Ollama);
        assert_eq!(AiProvider::from_str("openai").unwrap(), AiProvider::OpenAi);
        assert_eq!(
            AiProvider::from_str("azure-openai").unwrap(),
            AiProvider::AzureOpenAi
        );
        assert_eq!(
            AiProvider::from_str("azure").unwrap(),
            AiProvider::AzureOpenAi
        );
        assert!(matches!(
            AiProvider::from_str("unknown"),
            Err(Error::InvalidAiProvider(_))
        ));
    }

    #[test]
    fn provider_defaults_are_correct() {
        assert_eq!(
            AiProvider::Ollama.default_endpoint(),
            "http://localhost:11434"
        );
        assert_eq!(AiProvider::Ollama.default_model(), "llama3.2");
        assert_eq!(
            AiProvider::OpenAi.default_endpoint(),
            "https://api.openai.com/v1"
        );
        assert_eq!(AiProvider::OpenAi.default_model(), "gpt-4o-mini");
    }

    #[test]
    fn deserializes_ai_project_plan_json() {
        let json = r#"{
            "project_name": "my-demo",
            "package_name": "my_demo",
            "description": "A demo app",
            "files": [
                {
                    "path": "pyproject.toml",
                    "content": "[project]\nname = \"my-demo\""
                },
                {
                    "path": "src/my_demo/main.py",
                    "content": "print('hello')"
                }
            ]
        }"#;

        let plan: AiProjectPlan = serde_json::from_str(json).unwrap();
        assert_eq!(plan.project_name, "my-demo");
        assert_eq!(plan.package_name, "my_demo");
        assert_eq!(plan.files.len(), 2);
        assert_eq!(plan.files[0].path, PathBuf::from("pyproject.toml"));
        assert_eq!(plan.files[1].path, PathBuf::from("src/my_demo/main.py"));
    }
}
