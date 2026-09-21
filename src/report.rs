//! `new` 커맨드 결과 리포트. 같은 데이터에서 사람이 읽는 텍스트와 스크립트용
//! JSON을 만들어 출력 경로(stdout)를 한 곳에서 결정한다.

use std::path::{Path, PathBuf};

use crate::models::ProjectSpec;

/// `wpygen new` 실행 결과. `files`는 dry-run이면 생성 예정, 아니면 실제 생성한
/// 파일의 상대 경로 목록이다.
pub struct NewReport<'a> {
    pub spec: &'a ProjectSpec,
    pub target_dir: &'a Path,
    pub files: &'a [PathBuf],
    pub dry_run: bool,
}

impl NewReport<'_> {
    /// 스크립트용 JSON 한 덩어리. 키 순서는 계약이므로 테스트로 고정한다.
    pub fn to_json(&self) -> String {
        let files = if self.files.is_empty() {
            "[]".to_string()
        } else {
            let items = self
                .files
                .iter()
                .map(|path| format!("    \"{}\"", escape(&path.to_string_lossy())))
                .collect::<Vec<_>>()
                .join(",\n");
            format!("[\n{items}\n  ]")
        };

        format!(
            "{{\n  \"wpygen_version\": \"{version}\",\n  \"project_name\": \"{project_name}\",\n  \
             \"package_name\": \"{package_name}\",\n  \"template\": \"{template}\",\n  \
             \"grpc\": {grpc},\n  \"sqlite\": {sqlite},\n  \"target_dir\": \"{target_dir}\",\n  \
             \"dry_run\": {dry_run},\n  \"file_count\": {file_count},\n  \"files\": {files}\n}}",
            version = env!("CARGO_PKG_VERSION"),
            project_name = escape(&self.spec.project_name),
            package_name = escape(&self.spec.package_name),
            template = self.spec.template.as_str(),
            grpc = self.spec.grpc,
            sqlite = self.spec.sqlite,
            target_dir = escape(&self.target_dir.to_string_lossy()),
            dry_run = self.dry_run,
            file_count = self.files.len(),
            files = files,
        )
    }

    /// 기계 판독용 한 줄 출력. 생성될/생성된 파일의 경로를 계획 순서대로 한 줄씩
    /// 출력한다(`grep`/`xargs`용). 파일이 없으면 빈 문자열이다.
    pub fn to_plain(&self) -> String {
        self.files
            .iter()
            .map(|path| self.target_dir.join(path).display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 사람이 읽는 출력. `--dry-run`이면 파일 목록까지 보여준다.
    pub fn to_text(&self) -> String {
        if self.dry_run {
            let mut text = format!(
                "dry-run: {} 아래 생성될 파일 목록",
                self.target_dir.display()
            );
            for path in self.files {
                text.push_str(&format!("\n  {}", path.display()));
            }
            text.push_str("\n(dry-run 모드이므로 실제 파일은 생성되지 않았습니다)");
            return text;
        }

        format!(
            "생성 완료: {} (template={}, grpc={}, sqlite={}, files={})",
            self.target_dir.display(),
            self.spec.template.as_str(),
            on_off(self.spec.grpc),
            on_off(self.spec.sqlite),
            self.files.len()
        )
    }
}

fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

/// JSON 문자열 리터럴 본문으로 쓸 수 있게 이스케이프한다. 값은 검증된 이름과
/// 경로뿐이지만, 따옴표·역슬래시·제어문자가 들어와도 출력이 깨지지 않게 한다.
fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            ch if (ch as u32) < 0x20 => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TemplateKind;

    fn spec() -> ProjectSpec {
        ProjectSpec {
            project_name: "demo-app".to_string(),
            package_name: "demo_app".to_string(),
            template: TemplateKind::Cli,
            grpc: false,
            sqlite: false,
        }
    }

    #[test]
    fn json_report_is_complete_and_stable() {
        let files = vec![
            PathBuf::from("pyproject.toml"),
            PathBuf::from("src/demo_app/main.py"),
        ];
        let report = NewReport {
            spec: &spec(),
            target_dir: Path::new("/tmp/demo-app"),
            files: &files,
            dry_run: false,
        };

        let expected = format!(
            r#"{{
  "wpygen_version": "{}",
  "project_name": "demo-app",
  "package_name": "demo_app",
  "template": "cli",
  "grpc": false,
  "sqlite": false,
  "target_dir": "/tmp/demo-app",
  "dry_run": false,
  "file_count": 2,
  "files": [
    "pyproject.toml",
    "src/demo_app/main.py"
  ]
}}"#,
            env!("CARGO_PKG_VERSION")
        );

        assert_eq!(report.to_json(), expected);
    }

    #[test]
    fn json_report_escapes_values_and_allows_empty_file_list() {
        let mut project = spec();
        project.project_name = "quote\"and\\slash".to_string();
        let report = NewReport {
            spec: &project,
            target_dir: Path::new("/tmp"),
            files: &[],
            dry_run: true,
        };

        let json = report.to_json();
        assert!(
            json.contains(r#""project_name": "quote\"and\\slash""#),
            "{json}"
        );
        assert!(json.contains(r#""files": []"#), "{json}");
        assert!(json.contains(r#""file_count": 0"#), "{json}");
    }

    #[test]
    fn json_report_marks_dry_run_options() {
        let mut project = spec();
        project.grpc = true;
        project.sqlite = true;
        let report = NewReport {
            spec: &project,
            target_dir: Path::new("/tmp"),
            files: &[],
            dry_run: true,
        };

        let json = report.to_json();
        assert!(json.contains("\"dry_run\": true"), "{json}");
        assert!(json.contains("\"grpc\": true"), "{json}");
        assert!(json.contains("\"sqlite\": true"), "{json}");
    }

    #[test]
    fn text_report_lists_dry_run_files() {
        let files = vec![PathBuf::from("pyproject.toml")];
        let report = NewReport {
            spec: &spec(),
            target_dir: Path::new("/tmp/demo-app"),
            files: &files,
            dry_run: true,
        };

        assert_eq!(
            report.to_text(),
            "dry-run: /tmp/demo-app 아래 생성될 파일 목록\n  pyproject.toml\n\
             (dry-run 모드이므로 실제 파일은 생성되지 않았습니다)"
        );
    }

    #[test]
    fn text_report_summarizes_generation() {
        let mut project = spec();
        project.sqlite = true;
        let files = vec![
            PathBuf::from("pyproject.toml"),
            PathBuf::from("config.toml"),
        ];
        let report = NewReport {
            spec: &project,
            target_dir: Path::new("/tmp/demo-app"),
            files: &files,
            dry_run: false,
        };

        assert_eq!(
            report.to_text(),
            "생성 완료: /tmp/demo-app (template=cli, grpc=off, sqlite=on, files=2)"
        );
    }

    #[test]
    fn escape_encodes_control_characters() {
        assert_eq!(escape("a\tb\nc\rd"), "a\\tb\\nc\\rd");
        assert_eq!(escape("\u{1}"), "\\u0001");
        assert_eq!(escape("한글"), "한글");
    }

    #[test]
    fn plain_report_prints_one_path_per_line() {
        let files = vec![
            PathBuf::from("pyproject.toml"),
            PathBuf::from("src/demo_app/main.py"),
        ];
        let report = NewReport {
            spec: &spec(),
            target_dir: Path::new("/tmp/demo-app"),
            files: &files,
            dry_run: true,
        };

        // 경로 구분자는 플랫폼마다 다르므로 기대값도 같은 방식으로 조립한다.
        let expected = format!(
            "{}\n{}",
            Path::new("/tmp/demo-app").join("pyproject.toml").display(),
            Path::new("/tmp/demo-app")
                .join("src/demo_app/main.py")
                .display()
        );
        assert_eq!(report.to_plain(), expected);
    }

    #[test]
    fn plain_report_is_empty_without_files() {
        let report = NewReport {
            spec: &spec(),
            target_dir: Path::new("/tmp/demo-app"),
            files: &[],
            dry_run: false,
        };

        assert_eq!(report.to_plain(), "");
    }
}
