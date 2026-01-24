use crate::error::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

use super::Cli;

#[derive(Parser, Clone)]
pub struct CompletionsCommand {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run(cmd: CompletionsCommand) -> Result<()> {
    let mut cli = Cli::command();
    let name = cli.get_name().to_string();
    generate(cmd.shell, &mut cli, name, &mut std::io::stdout());
    Ok(())
}
