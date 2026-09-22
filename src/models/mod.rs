pub mod ai;
pub mod config;
mod generated_file;
mod project_spec;

pub use ai::{AiFileEntry, AiProjectPlan, AiProvider};
pub use config::AppConfig;
pub use generated_file::GeneratedFile;
pub use project_spec::{ProjectSpec, TemplateKind};
