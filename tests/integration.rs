use std::process::Command;

fn namecheap_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_namecheap"))
}

#[test]
fn test_help() {
    let output = namecheap_cmd()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("namecheap"));
    assert!(stdout.contains("DNS"));
}

#[test]
fn test_version() {
    let output = namecheap_cmd()
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("namecheap"));
}

#[test]
fn test_completions_bash() {
    let output = namecheap_cmd()
        .args(["completions", "bash"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("complete"));
}

#[test]
fn test_completions_zsh() {
    let output = namecheap_cmd()
        .args(["completions", "zsh"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("compdef"));
}

#[test]
fn test_completions_fish() {
    let output = namecheap_cmd()
        .args(["completions", "fish"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("complete"));
}

#[test]
fn test_preset_list() {
    let output = namecheap_cmd()
        .args(["preset", "list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("github-pages"));
    assert!(stdout.contains("google-workspace"));
}

#[test]
fn test_preset_list_json() {
    let output = namecheap_cmd()
        .args(["--json", "preset", "list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(parsed["presets"].is_array());
}

#[test]
fn test_preset_show() {
    let output = namecheap_cmd()
        .args(["preset", "show", "github-pages"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("github-pages"));
    assert!(stdout.contains("185.199"));
}

#[test]
fn test_preset_show_json() {
    let output = namecheap_cmd()
        .args(["--json", "preset", "show", "vercel"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert_eq!(parsed["name"], "vercel");
    assert!(parsed["records"].is_array());
}

#[test]
fn test_preset_not_found() {
    let output = namecheap_cmd()
        .args(["preset", "show", "nonexistent-preset"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("PresetNotFound"));
}

#[test]
fn test_auth_status_no_config() {
    // This should fail gracefully when no credentials are configured
    let output = namecheap_cmd()
        .args(["auth", "status"])
        .env("HOME", "/nonexistent")
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .output()
        .expect("Failed to execute command");

    // Should fail but not crash
    assert!(!output.status.success());
}

#[test]
fn test_dns_subcommands_help() {
    let subcommands = ["list", "add", "set", "rm", "export", "sync", "diff"];

    for subcmd in subcommands {
        let output = namecheap_cmd()
            .args(["dns", subcmd, "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success(), "dns {} --help failed", subcmd);
    }
}

#[test]
fn test_domains_subcommands_help() {
    let subcommands = ["list", "info", "check"];

    for subcmd in subcommands {
        let output = namecheap_cmd()
            .args(["domains", subcmd, "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success(), "domains {} --help failed", subcmd);
    }
}

#[test]
fn test_ns_subcommands_help() {
    let subcommands = ["list", "set", "reset"];

    for subcmd in subcommands {
        let output = namecheap_cmd()
            .args(["ns", subcmd, "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success(), "ns {} --help failed", subcmd);
    }
}

#[test]
fn test_redirect_subcommands_help() {
    let subcommands = ["list", "add", "rm"];

    for subcmd in subcommands {
        let output = namecheap_cmd()
            .args(["redirect", subcmd, "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success(), "redirect {} --help failed", subcmd);
    }
}
