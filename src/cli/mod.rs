pub mod auth;
pub mod completions;
pub mod dns;
pub mod domains;
pub mod ns;
pub mod preset;
pub mod redirect;
pub mod verify;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "namecheap")]
#[command(author, version, about = "CLI tool for managing Namecheap DNS records", long_about = None)]
pub struct Cli {
    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Profile to use
    #[arg(long, short = 'p', global = true)]
    pub profile: Option<String>,

    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Clone)]
pub struct GlobalOpts {
    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Dry run mode - don't make any changes
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Quiet mode - minimal output
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Verbose output
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Skip confirmation prompts
    #[arg(long, short = 'y', global = true)]
    pub yes: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authentication commands
    Auth(auth::AuthCommand),

    /// Domain management commands
    Domains(domains::DomainsCommand),

    /// DNS record management commands
    Dns(dns::DnsCommand),

    /// Preset management commands
    Preset(preset::PresetCommand),

    /// Verify DNS propagation
    Verify(verify::VerifyCommand),

    /// Nameserver management commands
    Ns(ns::NsCommand),

    /// URL redirect management commands
    Redirect(redirect::RedirectCommand),

    /// Generate shell completions
    Completions(completions::CompletionsCommand),
}
