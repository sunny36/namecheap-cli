use clap::Parser;
use namecheap_cli::cli::{self, Cli, Commands};
use namecheap_cli::config;
use namecheap_cli::error::{Result, EXIT_SUCCESS};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = run(cli).await;

    match result {
        Ok(()) => std::process::exit(EXIT_SUCCESS),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(e.exit_code());
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Commands that don't need config
    if let Commands::Completions(cmd) = &cli.command {
        return cli::completions::run(cmd.clone());
    }

    // Try to load config, but allow fallback for preset commands
    let config_result = config::load_config(cli.config.as_deref(), cli.profile.as_deref());

    match cli.command {
        Commands::Auth(cmd) => {
            // Auth login doesn't need existing config
            match cmd.command {
                cli::auth::AuthSubcommand::Login { .. } => {
                    let config = config_result.unwrap_or_else(|_| config::Config {
                        profile: config::Profile {
                            api_user: String::new(),
                            api_key: String::new(),
                            username: None,
                            client_ip: None,
                            sandbox: false,
                        },
                        profile_name: "default".to_string(),
                        presets: std::collections::HashMap::new(),
                    });
                    cli::auth::run(
                        cli::auth::AuthCommand {
                            command: cmd.command,
                        },
                        &config,
                        &cli.global,
                    )
                    .await
                }
                _ => {
                    let config = config_result?;
                    cli::auth::run(cmd, &config, &cli.global).await
                }
            }
        }
        Commands::Preset(cmd) => {
            // Preset list/show don't need authenticated config
            let config = config_result.unwrap_or_else(|_| config::Config {
                profile: config::Profile {
                    api_user: String::new(),
                    api_key: String::new(),
                    username: None,
                    client_ip: None,
                    sandbox: false,
                },
                profile_name: "default".to_string(),
                presets: std::collections::HashMap::new(),
            });
            cli::preset::run(cmd, &config, &cli.global).await
        }
        Commands::Domains(cmd) => {
            let config = config_result?;
            cli::domains::run(cmd, &config, &cli.global).await
        }
        Commands::Dns(cmd) => {
            let config = config_result?;
            cli::dns::run(cmd, &config, &cli.global).await
        }
        Commands::Verify(cmd) => {
            let config = config_result?;
            cli::verify::run(cmd, &config, &cli.global).await
        }
        Commands::Ns(cmd) => {
            let config = config_result?;
            cli::ns::run(cmd, &config, &cli.global).await
        }
        Commands::Redirect(cmd) => {
            let config = config_result?;
            cli::redirect::run(cmd, &config, &cli.global).await
        }
        Commands::Completions(_) => unreachable!(),
    }
}
