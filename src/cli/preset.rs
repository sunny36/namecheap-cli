use crate::api::NamecheapClient;
use crate::config::Config;
use crate::dns::{apply_diff, calculate_diff, DnsRecord, RecordType};
use crate::error::{CliError, Result};
use crate::output::{is_json, json, table};
use crate::preset::{self, Preset};
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{Confirm, Input};
use std::collections::HashMap;

use super::GlobalOpts;

#[derive(Parser)]
pub struct PresetCommand {
    #[command(subcommand)]
    pub command: PresetSubcommand,
}

#[derive(Subcommand)]
pub enum PresetSubcommand {
    /// List available presets
    List,

    /// Show preset details
    Show {
        /// Preset name
        name: String,
    },

    /// Apply a preset to a domain
    Apply {
        /// Preset name
        preset: String,

        /// Domain name
        domain: String,

        /// Variable values (format: VAR=value)
        #[arg(short = 'V', long = "var")]
        vars: Vec<String>,

        /// Delete records not in the preset (for email presets, only affects MX/TXT)
        #[arg(long)]
        replace: bool,
    },

    /// Remove preset records from a domain
    Remove {
        /// Preset name
        preset: String,

        /// Domain name
        domain: String,

        /// Variable values (format: VAR=value)
        #[arg(short = 'V', long = "var")]
        vars: Vec<String>,
    },
}

pub async fn run(cmd: PresetCommand, config: &Config, global: &GlobalOpts) -> Result<()> {
    match cmd.command {
        PresetSubcommand::List => list(config, global),
        PresetSubcommand::Show { name } => show(&name, config, global),
        PresetSubcommand::Apply {
            preset,
            domain,
            vars,
            replace,
        } => apply(&preset, &domain, &vars, replace, config, global).await,
        PresetSubcommand::Remove {
            preset,
            domain,
            vars,
        } => remove(&preset, &domain, &vars, config, global).await,
    }
}

fn list(config: &Config, global: &GlobalOpts) -> Result<()> {
    let presets = preset::list_presets(config);

    if is_json(global) {
        json::print_presets_json(&presets);
    } else {
        table::print_presets(&presets);
    }

    Ok(())
}

fn show(name: &str, config: &Config, global: &GlobalOpts) -> Result<()> {
    let preset = preset::get_preset(name, config)?;

    if is_json(global) {
        json::print_preset_json(&preset);
    } else {
        table::print_preset_detail(&preset);
    }

    Ok(())
}

async fn apply(
    preset_name: &str,
    domain: &str,
    vars: &[String],
    replace: bool,
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    let preset = preset::get_preset(preset_name, config)?;

    // Parse and collect variables
    let mut variables: HashMap<String, String> = HashMap::new();
    for var in vars {
        if let Some((key, value)) = var.split_once('=') {
            variables.insert(key.to_string(), value.to_string());
        }
    }

    // Prompt for missing required variables
    for var in &preset.variables {
        if var.required && !variables.contains_key(&var.name) {
            if global.yes {
                if let Some(default) = &var.default {
                    variables.insert(var.name.clone(), default.clone());
                } else {
                    return Err(CliError::Validation(format!(
                        "Required variable '{}' not provided",
                        var.name
                    )));
                }
            } else {
                let prompt = format!(
                    "{}{}",
                    var.name,
                    var.description
                        .as_ref()
                        .map(|d| format!(" ({})", d))
                        .unwrap_or_default()
                );
                let mut input = Input::new().with_prompt(&prompt);
                if let Some(default) = &var.default {
                    input = input.default(default.clone());
                }
                let value: String = input
                    .interact_text()
                    .map_err(|e| CliError::Other(e.to_string()))?;
                variables.insert(var.name.clone(), value);
            }
        } else if !variables.contains_key(&var.name) {
            if let Some(default) = &var.default {
                variables.insert(var.name.clone(), default.clone());
            }
        }
    }

    // Render records with variables
    let desired_records = render_preset_records(&preset, &variables)?;

    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let current = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    // Calculate what records to add
    let mut diffs = calculate_diff(&current, &desired_records);

    // If not replacing, only add new records
    if !replace {
        diffs.retain(|d| matches!(d.action, crate::dns::DiffAction::Add));
    }

    if diffs.is_empty() {
        if is_json(global) {
            json::print_success_json("No changes needed");
        } else if !global.quiet {
            println!("No changes needed. Preset records already exist.");
        }
        return Ok(());
    }

    if is_json(global) && global.dry_run {
        json::print_dns_diff_json(&diffs);
        return Ok(());
    } else if !global.quiet {
        table::print_dns_diff(&diffs);
    }

    if global.dry_run {
        return Ok(());
    }

    if !global.yes {
        let confirm = Confirm::new()
            .with_prompt("Apply preset?")
            .default(false)
            .interact()
            .map_err(|e| CliError::Other(e.to_string()))?;

        if !confirm {
            return Ok(());
        }
    }

    let new_records = apply_diff(&current, &diffs);
    client
        .set_hosts(domain, &new_records)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if is_json(global) {
        json::print_success_json("Preset applied successfully");
    } else if !global.quiet {
        println!(
            "{} Preset '{}' applied to {}",
            style("✓").green(),
            preset_name,
            domain
        );
    }

    Ok(())
}

async fn remove(
    preset_name: &str,
    domain: &str,
    vars: &[String],
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    let preset = preset::get_preset(preset_name, config)?;

    // Parse and collect variables
    let mut variables: HashMap<String, String> = HashMap::new();
    for var in vars {
        if let Some((key, value)) = var.split_once('=') {
            variables.insert(key.to_string(), value.to_string());
        }
    }

    // Apply defaults for missing variables
    for var in &preset.variables {
        if !variables.contains_key(&var.name) {
            if let Some(default) = &var.default {
                variables.insert(var.name.clone(), default.clone());
            }
        }
    }

    // Render records with variables
    let preset_records = render_preset_records(&preset, &variables)?;

    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let current = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    // Find records to remove
    let remaining: Vec<DnsRecord> = current
        .iter()
        .filter(|c| !preset_records.iter().any(|p| c.matches(p)))
        .cloned()
        .collect();

    let removed_count = current.len() - remaining.len();

    if removed_count == 0 {
        if is_json(global) {
            json::print_success_json("No preset records found to remove");
        } else if !global.quiet {
            println!("No preset records found to remove.");
        }
        return Ok(());
    }

    if !global.quiet && !is_json(global) {
        println!("Will remove {} record(s):", removed_count);
        for record in &current {
            if preset_records.iter().any(|p| record.matches(p)) {
                println!(
                    "  {} {} {} {}",
                    style("-").red(),
                    record.record_type,
                    record.host,
                    record.value
                );
            }
        }
    }

    if global.dry_run {
        return Ok(());
    }

    if !global.yes {
        let confirm = Confirm::new()
            .with_prompt("Remove these records?")
            .default(false)
            .interact()
            .map_err(|e| CliError::Other(e.to_string()))?;

        if !confirm {
            return Ok(());
        }
    }

    client
        .set_hosts(domain, &remaining)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if is_json(global) {
        json::print_success_json(&format!("Removed {} record(s)", removed_count));
    } else if !global.quiet {
        println!("{} Removed {} record(s)", style("✓").green(), removed_count);
    }

    Ok(())
}

fn render_preset_records(
    preset: &Preset,
    variables: &HashMap<String, String>,
) -> Result<Vec<DnsRecord>> {
    let mut records = Vec::new();

    for pr in &preset.records {
        let host = substitute_variables(&pr.host, variables);
        let value = substitute_variables(&pr.value, variables);
        let record_type: RecordType = pr
            .record_type
            .parse()
            .map_err(|e: String| CliError::Validation(e))?;

        records.push(DnsRecord::new(
            record_type,
            &host,
            &value,
            pr.ttl.unwrap_or(1800),
            pr.priority,
        ));
    }

    Ok(records)
}

fn substitute_variables(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}
