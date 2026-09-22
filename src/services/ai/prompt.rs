/// AI 모델에게 프로젝트 템플릿 생성을 지시하는 시스템 프롬프트.
pub fn build_system_prompt(project_name: &str, package_name: &str) -> String {
    format!(
        r#"You are an expert Python project scaffolding assistant.
Your task is to generate a complete, production-grade Python project template based on the user's requirements.

Project Guidelines:
1. Target Python version: 3.12 or newer.
2. Package manager: uv (use modern standard pyproject.toml with [project] table).
3. Always include development dependencies: [dependency-groups] dev = ["pytest>=8.0.0", "ruff>=0.8.0"].
4. Directory layout: Use src-layout (e.g., src/{package_name}/...).
5. Always generate at least:
   - pyproject.toml
   - README.md
   - .gitignore
   - src/{package_name}/__init__.py
   - src/{package_name}/main.py (or relevant entrypoint)
   - tests/test_smoke.py
6. All Python code must be syntactically valid Python 3.12+ code.

Output Format:
You MUST output ONLY a valid JSON object matching the following structure without any explanations or introductory text:
{{
  "project_name": "{project_name}",
  "package_name": "{package_name}",
  "description": "Short description of the generated template",
  "files": [
    {{
      "path": "relative/path/to/file",
      "content": "full content of the file"
    }}
  ]
}}

Ensure all file paths are relative paths and NEVER contain '..' or start with a slash."#
    )
}

/// 사용자의 자연어 프롬프트와 프로젝트 메타데이터를 결합한 유저 프롬프트.
pub fn build_user_prompt(user_requirement: &str, project_name: &str, package_name: &str) -> String {
    format!(
        "Project Name: {project_name}\n\
         Package Name: {package_name}\n\
         User Requirements:\n\
         {user_requirement}\n\n\
         Please generate the full project template JSON according to these specifications."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_contains_package_and_project_names() {
        let prompt = build_system_prompt("my-api", "my_api");
        assert!(prompt.contains("src/my_api/"));
        assert!(prompt.contains("\"project_name\": \"my-api\""));
        assert!(prompt.contains("Python 3.12"));
        assert!(prompt.contains("pyproject.toml"));
    }

    #[test]
    fn user_prompt_contains_requirements() {
        let user_p = build_user_prompt("FastAPI + Celery queue", "worker-app", "worker_app");
        assert!(user_p.contains("FastAPI + Celery queue"));
        assert!(user_p.contains("worker-app"));
        assert!(user_p.contains("worker_app"));
    }
}
