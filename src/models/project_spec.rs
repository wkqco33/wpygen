use crate::error::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "cli" => Ok(Self::Cli),
            "gui" => Ok(Self::Gui),
            "server" => Ok(Self::Server),
            other => Err(Error::InvalidTemplate(other.to_string())),
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
