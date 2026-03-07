# RFC-002: Configuration & Authentication

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** RFC-001  

---

## Summary

Define the configuration system, authentication flow, and environment management for the aiXplain CLI. Users must be able to authenticate, switch between environments, and persist preferences with minimal friction.

## Motivation

The aiXplain platform supports multiple authentication modes (team API keys, individual API keys) and environments (production, development, staging). The CLI must handle this transparently while following the principle of least surprise — environment variables override config files, command flags override everything.

## Configuration File

### Location

Following XDG Base Directory Specification:

| Platform | Path |
|----------|------|
| Linux | `~/.config/aixplain/config.toml` |
| macOS | `~/Library/Application Support/aixplain/config.toml` |
| Windows | `%APPDATA%\aixplain\config.toml` |

Resolved via the `dirs` crate's `config_dir()`.

### Schema

```toml
# ~/.config/aixplain/config.toml
config_version = 1

[auth]
# Active profile name
active_profile = "default"

[profiles.default]
api_key = "your-team-api-key-here"
key_type = "team"                     # "team" | "individual"
backend_url = "https://platform-api.aixplain.com"
models_run_url = "https://models.aixplain.com/api/v2/execute"

[profiles.dev]
api_key = "your-dev-key"
key_type = "team"
backend_url = "https://dev-platform-api.aixplain.com"
models_run_url = "https://dev-models.aixplain.com/api/v2/execute"

[display]
# Default output format: "table" | "json" | "compact"
output_format = "table"
# Default tab when launching TUI
default_tab = "models"
# Enable color output (auto-detected from TTY)
color = "auto"

[cache]
# Cache directory override (default: platform cache dir)
dir = ""
# TTL for cached API responses in seconds
ttl_seconds = 3600

[tui]
# Tick rate in milliseconds
tick_rate_ms = 250
# Enable mouse support
mouse = true
```

### Config Versioning

The `config_version` field allows forward-compatible migrations. If the CLI encounters a config version higher than it supports, it warns but continues with defaults. If lower, it migrates silently.

## Authentication

### Resolution Chain

Authentication credentials are resolved in priority order (highest first):

```
1. --api-key <key>              (CLI flag)
2. AIXPLAIN_API_KEY env var     (environment)
3. TEAM_API_KEY env var         (legacy compat with Python SDK)
4. profiles.<active>.api_key    (config file)
```

### Key Types

| Type | Header | Environment Variable |
|------|--------|---------------------|
| Team API Key | `x-api-key: {key}` | `TEAM_API_KEY` or `AIXPLAIN_API_KEY` |
| Individual API Key | `x-aixplain-key: {key}` | `AIXPLAIN_API_KEY` with `--key-type individual` |

The key type determines which HTTP header is set on requests.

### File Upload Authentication

File uploads to S3 use a different header: `Authorization: token {api_key}`. This is handled transparently by the file upload module.

## Profile Management

### CLI Commands

```bash
# Login interactively (prompts for key, validates against API)
aix auth login

# Login with explicit key
aix auth login --api-key <key>

# Login to specific profile
aix auth login --profile dev --api-key <key>

# Show current auth status
aix auth status

# Switch active profile
aix auth use <profile-name>

# List profiles
aix auth list

# Remove a profile
aix auth remove <profile-name>

# Show resolved configuration (redacted keys)
aix config show

# Set a config value
aix config set display.output_format json
aix config set cache.ttl_seconds 7200

# Open config file in $EDITOR
aix config edit

# Reset to defaults
aix config reset
```

### Login Flow

```
┌─────────────────────────────────────────────────┐
│  $ aix auth login                               │
│                                                  │
│  ? Enter your aiXplain API key: ●●●●●●●●●●●●●   │
│  ? Key type: (team/individual) [team]            │
│  ? Profile name: [default]                       │
│  ? Backend URL: [https://platform-api.aixplain…] │
│                                                  │
│  ✓ Validating credentials...                     │
│  ✓ Authenticated as Team (team_id: 12345)        │
│  ✓ Profile "default" saved                       │
└─────────────────────────────────────────────────┘
```

Validation makes a lightweight API call (e.g., `GET sdk/api-keys/usage-limits`) to confirm the key works.

### Key Security

- API keys are stored in the config file with restricted permissions (`chmod 600`)
- Keys are redacted in all log output and `aix config show` (`581b...ac52`)
- The `--api-key` flag value is never written to shell history (recommend using env var instead)
- Config file creation sets `0600` permissions on Unix

## Environment Variables

All config values can be overridden via environment variables:

| Variable | Config Equivalent | Notes |
|----------|-------------------|-------|
| `AIXPLAIN_API_KEY` | `profiles.<active>.api_key` | Primary key variable |
| `TEAM_API_KEY` | `profiles.<active>.api_key` | Legacy compat |
| `BACKEND_URL` | `profiles.<active>.backend_url` | API base URL |
| `MODELS_RUN_URL` | `profiles.<active>.models_run_url` | Model execution URL |
| `AIXPLAIN_PROFILE` | `auth.active_profile` | Override active profile |
| `AIXPLAIN_OUTPUT` | `display.output_format` | Output format |
| `NO_COLOR` | `display.color = "never"` | Standard no-color convention |

### `.env` File Support

The CLI loads `.env` files from the current directory and `~/.config/aixplain/.env`, using the standard priority: explicit env vars > CWD `.env` > config dir `.env`.

This is implemented via the `dotenvy` crate.

## Data Structures

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub config_version: u32,
    pub auth: AuthConfig,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub tui: TuiConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub active_profile: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub api_key: String,
    #[serde(default = "default_key_type")]
    pub key_type: KeyType,
    #[serde(default = "default_backend_url")]
    pub backend_url: String,
    #[serde(default = "default_models_run_url")]
    pub models_run_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
    Team,
    Individual,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,
    #[serde(default = "default_tab")]
    pub default_tab: String,
    #[serde(default = "default_color")]
    pub color: ColorMode,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Table,
    Json,
    Compact,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheConfig {
    pub dir: Option<String>,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_tick_rate")]
    pub tick_rate_ms: u64,
    #[serde(default = "default_mouse")]
    pub mouse: bool,
}
```

## Resolved Configuration

At startup, the CLI resolves the final configuration by layering sources:

```rust
pub struct ResolvedConfig {
    pub api_key: String,
    pub key_type: KeyType,
    pub backend_url: Url,
    pub models_run_url: Url,
    pub display: DisplayConfig,
    pub cache: CacheConfig,
    pub tui: TuiConfig,
    pub profile_name: String,
}

impl ResolvedConfig {
    pub fn auth_header(&self) -> (&str, &str) {
        match self.key_type {
            KeyType::Team => ("x-api-key", &self.api_key),
            KeyType::Individual => ("x-aixplain-key", &self.api_key),
        }
    }

    pub fn upload_auth_header(&self) -> (&str, String) {
        ("Authorization", format!("token {}", self.api_key))
    }
}
```

## Error Cases

| Scenario | Behavior |
|----------|----------|
| No API key found anywhere | Error: "No API key configured. Run `aix auth login` or set AIXPLAIN_API_KEY" |
| Invalid API key (401) | Error: "Invalid API key. Run `aix auth login` to update" |
| Config file parse error | Warn, use defaults, suggest `aix config reset` |
| Config file doesn't exist | Create default config on first `aix auth login` |
| Profile not found | Error: "Profile 'xyz' not found. Available: default, dev" |
| Permissions too open | Warn: "Config file has insecure permissions, fixing..." |

## Testing Strategy

- Unit tests for config parsing with various TOML fixtures (missing fields, unknown fields, version migration)
- Unit tests for auth resolution chain (flag > env > config)
- Integration test for `aix auth login` with mock API
- Integration test for `aix config show` output format

## Acceptance Criteria

- [ ] `aix auth login` stores credentials and validates them
- [ ] `aix auth status` shows current profile and redacted key
- [ ] `aix auth use <profile>` switches profiles
- [ ] Environment variables override config file values
- [ ] `--api-key` flag overrides everything
- [ ] Config file is created with `0600` permissions
- [ ] Missing config file is handled gracefully
- [ ] `.env` files are loaded from CWD and config directory
- [ ] `NO_COLOR` environment variable is respected
