use std::path::{Component, Path};

use crate::error::Error;
use crate::models::{AiFileEntry, AiProjectPlan};

/// AI가 제안한 파일 경로가 안전한 상대 경로인지 검증한다.
/// 절대 경로, 루트 접근, 상위 디렉터리 참조(`..`)를 엄격히 차단한다.
pub fn validate_relative_path(path: &Path) -> Result<(), Error> {
    if !path.is_relative() || path.as_os_str().is_empty() {
        return Err(Error::PathTraversalViolation(path.to_path_buf()));
    }

    for component in path.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(Error::PathTraversalViolation(path.to_path_buf()));
            }
            Component::Normal(_) | Component::CurDir => {}
        }
    }

    Ok(())
}

/// LLM 응답 문자열(마크다운 코드 블록 가능성 있음)을 파싱하고 경로 안전성을 검증한다.
pub fn parse_and_validate_plan(
    raw_response: &str,
    project_name: &str,
    package_name: &str,
) -> Result<AiProjectPlan, Error> {
    let clean_json = extract_json(raw_response);

    let mut plan: AiProjectPlan = serde_json::from_str(clean_json)
        .map_err(|e| Error::AiInvalidResponse(format!("JSON 역직렬화 실패: {e}")))?;

    if plan.files.is_empty() {
        return Err(Error::AiInvalidResponse(
            "생성할 파일 목록(files)이 비어있습니다.".to_string(),
        ));
    }

    for file in &plan.files {
        validate_relative_path(&file.path)?;
    }

    // 프로젝트명과 패키지명은 사용자가 CLI에서 지정한 명칭으로 확정한다.
    plan.project_name = project_name.to_string();
    plan.package_name = package_name.to_string();

    // pyproject.toml이 없으면 최소 스켈레톤을 자동 추가한다.
    if !plan
        .files
        .iter()
        .any(|f| f.path == Path::new("pyproject.toml"))
    {
        plan.files.insert(
            0,
            AiFileEntry {
                path: "pyproject.toml".into(),
                content: format!(
                    "[project]\nname = \"{project_name}\"\nversion = \"0.1.0\"\nrequires-python = \">=3.12\"\ndependencies = []\n"
                ),
            },
        );
    }

    // .python-version이 없으면 3.12 기본 추가
    if !plan
        .files
        .iter()
        .any(|f| f.path == Path::new(".python-version"))
    {
        plan.files.push(AiFileEntry {
            path: ".python-version".into(),
            content: "3.12\n".into(),
        });
    }

    Ok(plan)
}

/// ```json ... ``` 같은 마크다운 코드 블록을 벗겨내어 순수 JSON 문자열만 추출한다.
fn extract_json(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(stripped) = trimmed.strip_prefix("```json")
        && let Some(end) = stripped.rfind("```")
    {
        stripped[..end].trim()
    } else if let Some(stripped) = trimmed.strip_prefix("```")
        && let Some(end) = stripped.rfind("```")
    {
        stripped[..end].trim()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_valid_relative_paths() {
        assert!(validate_relative_path(Path::new("pyproject.toml")).is_ok());
        assert!(validate_relative_path(Path::new("src/my_app/main.py")).is_ok());
        assert!(validate_relative_path(Path::new("tests/test_main.py")).is_ok());
        assert!(validate_relative_path(Path::new("./nested/file.txt")).is_ok());
    }

    #[test]
    fn rejects_path_traversal_and_absolute_paths() {
        assert!(validate_relative_path(Path::new("../etc/passwd")).is_err());
        assert!(validate_relative_path(Path::new("src/../../secret")).is_err());
        assert!(validate_relative_path(Path::new("/root/bad.py")).is_err());
        assert!(validate_relative_path(Path::new("")).is_err());
    }

    #[test]
    fn parses_valid_json_response() {
        let json = r#"{
            "project_name": "ai-demo",
            "package_name": "ai_demo",
            "description": "demo",
            "files": [
                {
                    "path": "pyproject.toml",
                    "content": "[project]\nname = \"ai-demo\""
                },
                {
                    "path": "src/ai_demo/main.py",
                    "content": "print('ok')"
                }
            ]
        }"#;

        let plan = parse_and_validate_plan(json, "ai-demo", "ai_demo").unwrap();
        assert_eq!(plan.project_name, "ai-demo");
        assert_eq!(plan.package_name, "ai_demo");
        assert_eq!(plan.files.len(), 3); // pyproject.toml, main.py, .python-version(auto-added)
    }

    #[test]
    fn extracts_json_from_markdown_fences() {
        let fenced =
            "```json\n{\n  \"files\": [{\"path\": \"main.py\", \"content\": \"\"}]\n}\n```";
        let plan = parse_and_validate_plan(fenced, "app", "app").unwrap();
        assert_eq!(plan.project_name, "app");
        assert!(plan.files.iter().any(|f| f.path == Path::new("main.py")));
    }

    #[test]
    fn rejects_empty_file_list() {
        let json = r#"{ "files": [] }"#;
        let err = parse_and_validate_plan(json, "app", "app").unwrap_err();
        assert!(matches!(err, Error::AiInvalidResponse(_)));
    }

    #[test]
    fn rejects_malicious_paths_in_plan() {
        let json = r#"{
            "files": [
                {
                    "path": "../evil.py",
                    "content": "evil"
                }
            ]
        }"#;
        let err = parse_and_validate_plan(json, "app", "app").unwrap_err();
        assert!(matches!(err, Error::PathTraversalViolation(_)));
    }
}
