use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GeneratedFile {
    pub relative_path: PathBuf,
    pub contents: String,
}
