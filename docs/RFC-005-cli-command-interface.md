# RFC-005: CLI Command Interface

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** RFC-001, RFC-002, RFC-003, RFC-004  

---

## Summary

Define the CLI command tree, output formatting, interactive prompts, and shell integration for the aiXplain CLI. The CLI is the primary non-TUI interface and must be composable, scriptable, and delightful to use.

## Motivation

The CLI serves two audiences:
1. **Interactive users** who want quick access to aiXplain resources with styled, readable output
2. **Scripts and automation** that need structured JSON output and predictable exit codes

The design follows modern CLI conventions (GitHub CLI, Stripe CLI, Railway CLI) with `--json` flags, table output, and progressive disclosure.

## Command Tree

```
aix
├── auth
│   ├── login                   # Authenticate with aiXplain
│   ├── logout                  # Remove stored credentials
│   ├── status                  # Show current auth state
│   ├── use <profile>           # Switch active profile
│   └── list                    # List configured profiles
│
├── models
│   ├── list                    # List/search models
│   ├── get <id>                # Show model details
│   ├── run <id>                # Execute a model
│   └── stream <id>             # Stream model output (LLMs)
│
├── agents
│   ├── list                    # List/search agents
│   ├── get <id>                # Show agent details
│   ├── create                  # Create a new agent
│   ├── update <id>             # Update an agent
│   ├── delete <id>             # Delete an agent
│   ├── run <id>                # Run an agent
│   └── chat <id>               # Interactive chat session
│
├── tools
│   ├── list                    # List/search tools
│   ├── get <id>                # Show tool details
│   ├── create                  # Create a new tool
│   ├── delete <id>             # Delete a tool
│   └── run <id>                # Run a tool action
│
├── integrations
│   ├── list                    # List/search integrations
│   ├── get <id>                # Show integration details
│   ├── actions <id>            # List available actions
│   └── connect <id>            # Create a tool from integration
│
├── files
│   └── upload <path>           # Upload a file to aiXplain
│
├── api-keys
│   ├── list                    # List API keys
│   ├── create                  # Create new API key
│   ├── get <id>                # Show API key details
│   ├── update <id>             # Update API key
│   ├── delete <id>             # Delete API key
│   └── usage [id]              # Show usage statistics
│
├── config
│   ├── show                    # Display resolved configuration
│   ├── set <key> <value>       # Set a config value
│   ├── edit                    # Open config in $EDITOR
│   └── reset                   # Reset to defaults
│
├── browse                      # Launch TUI (default when no args)
│
└── completion <shell>          # Generate shell completions
```

## Clap Definitions

```rust
use clap::{Parser, Subcommand, Args, ValueEnum};

#[derive(Parser)]
#[command(
    name = "aix",
    about = "aiXplain CLI — Browse and run AI models, agents, and tools",
    version,
    long_version = long_version(),
    propagate_version = true,
    arg_required_else_help = false,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, global = true, help = "Override API key")]
    pub api_key: Option<String>,

    #[arg(long, global = true, help = "Use named profile")]
    pub profile: Option<String>,

    #[arg(long, global = true, help = "Output as JSON")]
    pub json: bool,

    #[arg(long, global = true, help = "Verbose output")]
    pub verbose: bool,

    #[arg(long, global = true, help = "Suppress non-essential output")]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authentication and profile management
    Auth(AuthArgs),

    /// Browse and run models
    Models(ModelsArgs),

    /// Manage and run agents
    Agents(AgentsArgs),

    /// Manage and run tools
    Tools(ToolsArgs),

    /// Browse integrations
    Integrations(IntegrationsArgs),

    /// Upload files
    Files(FilesArgs),

    /// Manage API keys
    #[command(name = "api-keys")]
    ApiKeys(ApiKeysArgs),

    /// Configuration management
    Config(ConfigArgs),

    /// Launch interactive TUI browser
    Browse,

    /// Generate shell completions
    Completion(CompletionArgs),
}
```

### Models Commands

```rust
#[derive(Args)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub command: ModelsCommands,
}

#[derive(Subcommand)]
pub enum ModelsCommands {
    /// List models with optional filtering
    List {
        #[arg(short, long, help = "Search query")]
        query: Option<String>,

        #[arg(short, long, help = "Filter by function (e.g., text-generation)")]
        function: Option<Vec<String>>,

        #[arg(long, help = "Filter by supplier")]
        supplier: Option<Vec<String>>,

        #[arg(short, long, default_value = "20", help = "Results per page")]
        limit: i64,

        #[arg(short, long, default_value = "0", help = "Page number")]
        page: i64,

        #[arg(long, help = "Sort field (name, date, cost)")]
        sort: Option<String>,

        #[arg(long, help = "Sort descending")]
        desc: bool,
    },

    /// Show detailed model information
    Get {
        /// Model ID
        id: String,
    },

    /// Run a model
    Run {
        /// Model ID
        id: String,

        /// Input text (or use --file for file input)
        #[arg(short, long)]
        text: Option<String>,

        /// Input file path
        #[arg(short, long)]
        file: Option<String>,

        /// Read input from stdin
        #[arg(long)]
        stdin: bool,

        /// Additional parameters as key=value pairs
        #[arg(short, long, value_parser = parse_key_value)]
        param: Vec<(String, String)>,

        /// Wait for completion (default: true)
        #[arg(long, default_value = "true")]
        wait: bool,

        /// Timeout in seconds
        #[arg(long, default_value = "300")]
        timeout: u64,
    },

    /// Stream model output (for LLMs)
    Stream {
        /// Model ID
        id: String,

        /// Input text
        #[arg(short, long)]
        text: Option<String>,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,

        /// Additional parameters
        #[arg(short, long, value_parser = parse_key_value)]
        param: Vec<(String, String)>,
    },
}
```

### Agents Commands

```rust
#[derive(Subcommand)]
pub enum AgentsCommands {
    /// List agents
    List {
        #[arg(short, long)]
        query: Option<String>,

        #[arg(short, long, default_value = "20")]
        limit: i64,

        #[arg(short, long, default_value = "0")]
        page: i64,
    },

    /// Show agent details
    Get { id: String },

    /// Create a new agent
    Create {
        /// Agent name
        #[arg(short, long)]
        name: String,

        /// Instructions/system prompt
        #[arg(short, long)]
        instructions: Option<String>,

        /// Instructions from file
        #[arg(long)]
        instructions_file: Option<String>,

        /// LLM model ID to use
        #[arg(long)]
        llm: Option<String>,

        /// Tool IDs to attach
        #[arg(long)]
        tool: Vec<String>,

        /// Max iterations
        #[arg(long, default_value = "5")]
        max_iterations: i32,

        /// Max tokens
        #[arg(long, default_value = "2048")]
        max_tokens: i32,
    },

    /// Update an agent
    Update {
        id: String,

        #[arg(short, long)]
        name: Option<String>,

        #[arg(short, long)]
        instructions: Option<String>,

        #[arg(long)]
        llm: Option<String>,

        #[arg(long)]
        tool: Vec<String>,
    },

    /// Delete an agent
    Delete {
        id: String,

        #[arg(long, help = "Skip confirmation")]
        force: bool,
    },

    /// Run an agent with a query
    Run {
        id: String,

        /// Query text
        #[arg(short, long)]
        query: Option<String>,

        /// Read query from stdin
        #[arg(long)]
        stdin: bool,

        /// Session ID for conversation continuity
        #[arg(long)]
        session: Option<String>,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        output_format: String,

        /// Timeout in seconds
        #[arg(long, default_value = "300")]
        timeout: u64,
    },

    /// Interactive chat session with agent
    Chat {
        id: String,

        /// Resume existing session
        #[arg(long)]
        session: Option<String>,
    },
}
```

## Output Formatting

### Table Output (Default for TTY)

```
$ aix models list --query "gpt" --function text-generation

  ID                        NAME                   FUNCTION          SUPPLIER    STATUS
  ─────────────────────────────────────────────────────────────────────────────────────
  6414bd3cd09663e9ef1b6      GPT-4o                Text Generation   OpenAI      ● Online
  6414bd3cd09663e9ef1b7      GPT-4o Mini           Text Generation   OpenAI      ● Online
  6414bd3cd09663e9ef1b8      GPT-4 Turbo           Text Generation   OpenAI      ● Online

  Showing 3 of 47 results (page 1/3)
```

### JSON Output (`--json`)

```json
{
  "results": [
    {
      "id": "6414bd3cd09663e9ef1b6",
      "name": "GPT-4o",
      "function": "text-generation",
      "supplier": "OpenAI",
      "status": "ONBOARDED"
    }
  ],
  "total": 47,
  "page": 0,
  "pageTotal": 3
}
```

### Detail View

```
$ aix models get 6414bd3cd09663e9ef1b6

  ╭──────────────────────────────────────────────╮
  │  GPT-4o                                      │
  │  OpenAI · Text Generation                    │
  ├──────────────────────────────────────────────┤
  │  ID          6414bd3cd09663e9ef1b6            │
  │  Status      ● Online                        │
  │  Streaming   ✓ Supported                     │
  │  Path        openai/gpt-4o/openai            │
  │  Created     2024-03-15                       │
  │  Updated     2024-06-20                       │
  ├──────────────────────────────────────────────┤
  │  Pricing                                     │
  │  $0.005 / 1K tokens (input)                  │
  │  $0.015 / 1K tokens (output)                 │
  ├──────────────────────────────────────────────┤
  │  Parameters                                  │
  │  text         string  (required)             │
  │  temperature  float   (default: 0.7)         │
  │  max_tokens   int     (default: 2048)        │
  ╰──────────────────────────────────────────────╯
```

### Run Output

```
$ aix models run 6414bd3cd09663e9ef1b6 --text "What is Rust?"

  ⠋ Running model...

  ──────────────────────────────────────────────
  Rust is a systems programming language focused
  on safety, speed, and concurrency...
  ──────────────────────────────────────────────

  Credits: 0.002  │  Time: 1.3s  │  Tokens: 45/128
```

### Stream Output

```
$ aix models stream 6414bd3cd09663e9ef1b6 --text "Write a haiku about Rust"

  Rust is a systems▌
```

Text streams directly to stdout, character by character. No framing when piped.

## TTY-Aware Output

```rust
use std::sync::OnceLock;

static IS_TTY: OnceLock<bool> = OnceLock::new();

pub fn is_tty() -> bool {
    *IS_TTY.get_or_init(|| atty::is(atty::Stream::Stdout))
}

pub fn output<T: Serialize + TableDisplay>(data: &T, json_mode: bool) {
    if json_mode || !is_tty() {
        println!("{}", serde_json::to_string_pretty(data).unwrap());
    } else {
        data.print_table();
    }
}
```

### Trait for Dual Rendering

```rust
pub trait TableDisplay {
    fn print_table(&self);
    fn print_detail(&self);
}

impl TableDisplay for Page<Model> {
    fn print_table(&self) {
        let mut table = Table::new();
        table.set_header(vec!["ID", "Name", "Function", "Supplier", "Status"]);
        for model in &self.results {
            table.add_row(vec![
                model.base.id.as_deref().unwrap_or("-"),
                model.base.name.as_deref().unwrap_or("-"),
                // ...
            ]);
        }
        println!("{table}");
    }
}
```

## Interactive Features

### Confirmation Prompts

Destructive operations prompt for confirmation:

```
$ aix agents delete abc123

  ⚠ Delete agent "My Agent" (abc123)?
  This action cannot be undone.

  ? Confirm deletion [y/N]: y
  ✓ Agent deleted
```

Skipped with `--force` or when not a TTY.

### Interactive Agent Chat

```
$ aix agents chat abc123

  ╭─ Chat with "My Research Agent" ─────────────╮
  │  Type your message and press Enter.          │
  │  Type /quit to exit, /clear to reset.        │
  ╰──────────────────────────────────────────────╯

  you › What papers were published about LLMs last week?

  ⠋ Thinking...

  agent › I found 3 relevant papers:
          1. "Scaling Laws for..." (arXiv:2024.1234)
          2. ...

  you › Tell me more about the first one
```

### Stdin Support

All run commands accept stdin for input:

```bash
echo "Translate this to French" | aix models run $MODEL_ID --stdin
cat document.txt | aix agents run $AGENT_ID --stdin
```

## Shell Completions

```rust
#[derive(Args)]
pub struct CompletionArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: CompletionShell,
}

#[derive(ValueEnum, Clone)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}
```

Generated via `clap_complete`:

```bash
# Install completions
aix completion zsh > ~/.zfunc/_aix
aix completion bash > /etc/bash_completion.d/aix
aix completion fish > ~/.config/fish/completions/aix.fish
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Usage error (bad arguments) |
| 3 | Authentication error |
| 4 | Resource not found |
| 5 | Operation timeout |
| 130 | Interrupted (Ctrl+C) |

## Progress Indicators

```rust
use indicatif::{ProgressBar, ProgressStyle};

fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
```

## Pipe-Friendly Design

When stdout is not a TTY:
- No spinners, progress bars, or color
- JSON output by default
- No confirmation prompts (fail-safe: abort destructive ops unless `--force`)
- Stream output writes raw text without framing
- Errors go to stderr

## Testing Strategy

### CLI Integration Tests

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version() {
    Command::cargo_bin("aix")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("aix"));
}

#[test]
fn test_help() {
    Command::cargo_bin("aix")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("models"))
        .stdout(predicate::str::contains("agents"));
}

#[test]
fn test_models_list_json() {
    // Uses wiremock for API mocking
    Command::cargo_bin("aix")
        .unwrap()
        .args(["models", "list", "--json"])
        .env("AIXPLAIN_API_KEY", "test-key")
        .assert()
        .success();
}

#[test]
fn test_no_auth_error() {
    Command::cargo_bin("aix")
        .unwrap()
        .args(["models", "list"])
        .env_remove("AIXPLAIN_API_KEY")
        .env_remove("TEAM_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("aix auth login"));
}
```

## Acceptance Criteria

- [ ] `aix` with no args launches TUI (RFC-009) or shows help
- [ ] `aix models list` shows table output in TTY, JSON when piped
- [ ] `aix models get <id>` shows formatted detail view
- [ ] `aix models run <id> --text "..."` executes and shows result
- [ ] `aix agents chat <id>` enables interactive conversation
- [ ] `--json` flag works on every list/get command
- [ ] Shell completions generate for bash, zsh, fish, powershell
- [ ] Destructive operations prompt for confirmation
- [ ] Stdin input works for all run commands
- [ ] Exit codes are consistent and documented
- [ ] No color/spinners when output is piped
