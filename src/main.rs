#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

mod linter;
use std::path::PathBuf;

use clap::Parser;

/// A github action to lint ClusterWorkflowTemplates
#[derive(Debug, Parser)]
#[allow(clippy::large_enum_variant)]
enum Cli {
    /// Lint the templates
    Lint(LintArgs),
}

/// Arguments for linting the templates
#[derive(Debug, Parser)]
struct LintArgs {
    /// The base path to the checked-out repo.
    #[arg(long, env="BASE_PATH", default_value = ".")]
    base_path: PathBuf,

    /// Location of the workflows-lint config file - relative to the base_path
    #[arg(long, env="CONFIG_FILE", default_value = ".workflows-lint.yaml")]
    config_file: PathBuf,
}

fn main() {
    let args = Cli::parse();

    match args {
        Cli::Lint(args) => {
            linter::lint(args);
        }
    }
}
