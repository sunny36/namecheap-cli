# namecheap-cli

A cross-platform CLI tool for managing Namecheap DNS records.

## Installation

### From source

```bash
cargo install --path .
```

### From crates.io

```bash
cargo install namecheap-cli
```

## Configuration

### Interactive setup

```bash
namecheap auth login
```

### Environment variables

```bash
export NAMECHEAP_API_USER="your-api-user"
export NAMECHEAP_API_KEY="your-api-key"
export NAMECHEAP_USERNAME="your-username"  # Optional, defaults to API user
export NAMECHEAP_CLIENT_IP="your-ip"       # Optional, auto-detected
export NAMECHEAP_SANDBOX="true"            # Optional, use sandbox API
```

### Config file

Configuration is stored in `~/.config/namecheap-cli/config.toml`:

```toml
default_profile = "default"

[profiles.default]
api_user = "your-api-user"
api_key = "${NAMECHEAP_API_KEY}"  # Environment variable expansion
username = "your-username"
sandbox = false

[profiles.sandbox]
api_user = "your-api-user"
api_key = "${NAMECHEAP_SANDBOX_API_KEY}"
sandbox = true
```

## Usage

### Authentication

```bash
# Configure credentials interactively
namecheap auth login

# Check authentication status
namecheap auth status

# Show current user info and balance
namecheap auth whoami
```

### Domain Management

```bash
# List all domains
namecheap domains list

# Get domain info
namecheap domains info example.com

# Check domain availability
namecheap domains check example.com example.org
```

### DNS Records

```bash
# List DNS records
namecheap dns list example.com

# Filter by type
namecheap dns list example.com -t A

# Add a record
namecheap dns add example.com A @ 1.2.3.4

# Add MX record with priority
namecheap dns add example.com MX @ mail.example.com --priority 10

# Set (replace) a record
namecheap dns set example.com A @ 5.6.7.8

# Remove a record
namecheap dns rm example.com A @

# Remove specific value
namecheap dns rm example.com A @ 1.2.3.4

# Export records
namecheap dns export example.com --format json
namecheap dns export example.com --format zone

# Show diff between current and desired
namecheap dns diff example.com records.json

# Sync records from file
namecheap dns sync example.com records.json
namecheap dns sync example.com records.json --delete  # Remove records not in file
```

### Presets

```bash
# List available presets
namecheap preset list

# Show preset details
namecheap preset show github-pages

# Apply a preset
namecheap preset apply github-pages example.com -V username=myuser

# Remove preset records
namecheap preset remove github-pages example.com
```

Available presets:
- `github-pages` - GitHub Pages hosting
- `google-workspace` - Google Workspace email
- `fastmail` - Fastmail email
- `protonmail` - ProtonMail email
- `microsoft-365` - Microsoft 365 email
- `cloudflare` - Cloudflare DNS proxy
- `netlify` - Netlify hosting
- `vercel` - Vercel hosting
- `sendgrid` - SendGrid email sending

### Nameservers

```bash
# List nameservers
namecheap ns list example.com

# Set custom nameservers
namecheap ns set example.com ns1.cloudflare.com ns2.cloudflare.com

# Reset to Namecheap defaults
namecheap ns reset example.com
```

### URL Redirects

```bash
# List redirects
namecheap redirect list example.com

# Add redirect
namecheap redirect add example.com @ https://www.example.com

# Add permanent (301) redirect
namecheap redirect add example.com old https://new.example.com --permanent

# Add frame/masked redirect
namecheap redirect add example.com masked https://example.com --frame

# Remove redirect
namecheap redirect rm example.com @
```

### DNS Verification

```bash
# Verify DNS records are propagated
namecheap verify example.com

# Verify specific record type
namecheap verify example.com -t A

# Wait for propagation
namecheap verify example.com --wait --timeout 300
```

### Shell Completions

```bash
# Bash
namecheap completions bash > ~/.local/share/bash-completion/completions/namecheap

# Zsh
namecheap completions zsh > ~/.zfunc/_namecheap

# Fish
namecheap completions fish > ~/.config/fish/completions/namecheap.fish

# PowerShell
namecheap completions powershell > namecheap.ps1
```

## Global Options

| Option | Description |
|--------|-------------|
| `--config <path>` | Path to config file |
| `-p, --profile <name>` | Profile to use |
| `--json` | Output as JSON |
| `--dry-run` | Don't make any changes |
| `-q, --quiet` | Minimal output |
| `-v, --verbose` | Verbose output |
| `-y, --yes` | Skip confirmation prompts |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Authentication error |
| 3 | Domain not found |
| 4 | Record not found |
| 5 | Validation error |
| 6 | Network error |
| 7 | Verification failed |

## API Access

To use this tool, you need API access enabled on your Namecheap account:

1. Go to Profile > Tools > Namecheap API Access
2. Enable API Access
3. Add your IP address to the whitelist
4. Copy your API Key

For testing, you can use the sandbox API by setting `sandbox = true` in your profile.

## License

MIT
