use clap::ValueEnum;

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum TemplateKind {
    Cli,
    Gui,
    Server,
}

impl TemplateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Gui => "gui",
            Self::Server => "server",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectSpec {
    pub project_name: String,
    pub package_name: String,
    pub template: TemplateKind,
    pub grpc: bool,
    pub sqlite: bool,
}
