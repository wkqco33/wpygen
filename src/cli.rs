use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::models::TemplateKind;

const DEFAULT_INDEX_URL: &str = "https://pypi.wkqcosoft.cloud";

#[derive(Debug, Parser)]
#[command(
    name = "wpygen",
    version,
    about = "Python 프로젝트 템플릿(CLI/GUI/SERVER) 생성기"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 새 Python 프로젝트 템플릿을 생성한다.
    New(NewArgs),
}

#[derive(Debug, Args)]
pub struct NewArgs {
    /// 생성할 프로젝트 디렉터리 이름
    pub name: String,

    /// 생성할 템플릿 종류
    #[arg(long, short = 't', value_enum)]
    pub template: TemplateKind,

    /// gRPC 세팅을 함께 생성한다.
    #[arg(long)]
    pub grpc: bool,

    /// SQLite 세팅을 함께 생성한다.
    #[arg(long)]
    pub sqlite: bool,

    /// 프로젝트를 생성할 상위 디렉터리
    #[arg(long, short = 'o', default_value = ".")]
    pub output: PathBuf,

    /// Python 패키지명 (기본값: 프로젝트명 기반 정규화)
    #[arg(long)]
    pub package_name: Option<String>,

    /// 대상 디렉터리가 비어있지 않아도 파일을 덮어쓴다.
    #[arg(long)]
    pub force: bool,

    /// 사설 패키지 인덱스 URL
    #[arg(long, default_value = DEFAULT_INDEX_URL)]
    pub index_url: String,
}
