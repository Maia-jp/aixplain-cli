# aix — aiXplain CLI

A fast, terminal-native interface for the [aiXplain](https://aixplain.com) AI platform. Browse models, manage agents, run tools, and explore integrations — all from your terminal.

Built in Rust. Single binary. Zero runtime dependencies.

```
$ aix models list --limit 5

┌──────────────────────┬─────────────────────────┬─────────────────────────┬──────────┬────────┐
│ ID                   ┆ Name                    ┆ Function                ┆ Supplier ┆ Status │
╞══════════════════════╪═════════════════════════╪═════════════════════════╪══════════╪════════╡
│ 6509990b513e2451702… ┆ CER                     ┆ Text Generation Metric  ┆ aixplain ┆ Online │
│ 625fe92af3a0773bd51… ┆ Language Identification ┆ Language Identification ┆ aixplain ┆ Online │
│ 66aa869f6eb56342c26… ┆ Cloud Translation       ┆ Translation             ┆ google   ┆ Online │
└──────────────────────┴─────────────────────────┴─────────────────────────┴──────────┴────────┘
```

## Installation

### macOS / Linux (one command)

```bash
curl -fsSL https://raw.githubusercontent.com/aixplain/aixplain-cli/main/scripts/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/aixplain/aixplain-cli/main/scripts/install.ps1 | iex
```

### Homebrew (macOS)

```bash
brew install aixplain/tap/aix
```

### With Cargo (any platform with Rust installed)

```bash
cargo install aixplain-cli
```

### From source

```bash
git clone https://github.com/aixplain/aixplain-cli.git
cd aixplain-cli
cargo install --path .
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/aixplain/aixplain-cli/releases) for:

| Platform | Architecture | File |
|----------|-------------|------|
| Linux | x86_64 | `aix-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `aix-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | Intel | `aix-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `aix-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `aix-vX.Y.Z-x86_64-pc-windows-msvc.zip` |

## Quick start

**1. Set your API key**

Create a `.env` file in the project directory (or export the variable):

```bash
cp .env.example .env
# Edit .env with your key:
AIXPLAIN_API_KEY=your-key-here
BACKEND_URL=https://platform-api.aixplain.com
MODELS_RUN_URL=https://models.aixplain.com/api/v2/execute
```

Or export directly:

```bash
export AIXPLAIN_API_KEY=your-key-here
```

**2. Browse with the TUI**

```bash
aix
```

This launches the interactive terminal browser. Use `j/k` to navigate, `Tab` to switch between Models/Agents/Tools/Integrations, `r` to run, `/` to search.

**3. Use the CLI**

```bash
aix models list --query "translation"
aix agents run <AGENT_ID> --query "What is 2+2?"
aix tools list --limit 10
```

## Usage

### Interactive TUI

```bash
aix              # Launch TUI (default)
aix browse       # Same as above
```

**Keybindings:**

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate up/down |
| `]` / `[` | Next/previous tab |
| `l` / `Enter` | Open detail panel |
| `h` / `Esc` | Close detail panel |
| `r` | Run selected model/agent |
| `/` | Search |
| `c` | Copy ID |
| `g` / `G` | Jump to first/last |
| `Ctrl+D` / `Ctrl+U` | Page down/up |
| `?` | Help |
| `q` | Quit |

### Models

```bash
# List models
aix models list
aix models list --query "gpt" --function text-generation
aix models list --limit 50 --json

# Get model details
aix models get <MODEL_ID>

# Run a model
aix models run <MODEL_ID> --text "Translate hello to French"

# Pipe input
echo "What is Rust?" | aix models run <MODEL_ID> --stdin
```

### Agents

```bash
# List agents
aix agents list
aix agents list --query "research"

# CRUD
aix agents create --name "My Agent" --instructions "You are helpful."
aix agents get <AGENT_ID>
aix agents update <AGENT_ID> --name "Updated Name"
aix agents delete <AGENT_ID>

# Run an agent
aix agents run <AGENT_ID> --query "Summarize the latest AI news"

# Interactive chat
aix agents chat <AGENT_ID>
```

### Tools

```bash
aix tools list
aix tools get <TOOL_ID>
aix tools delete <TOOL_ID>
```

### Integrations

```bash
aix integrations list
aix integrations get <INTEGRATION_ID>
```

### API Keys

```bash
aix api-keys usage
aix api-keys list
```

### JSON output

Every command supports `--json` for scripting and piping:

```bash
aix models list --json | jq '.results[].name'
aix agents get <ID> --json | jq '.instructions'
```

## Configuration

The CLI reads configuration from a `.env` file in the current directory:

| Variable | Default | Description |
|----------|---------|-------------|
| `AIXPLAIN_API_KEY` | — | Your aiXplain API key (required) |
| `TEAM_API_KEY` | — | Legacy alias for API key |
| `BACKEND_URL` | `https://platform-api.aixplain.com` | API base URL |
| `MODELS_RUN_URL` | `https://models.aixplain.com/api/v2/execute` | Model execution URL |

You can also pass `--api-key <KEY>` to any command to override.

## Project structure

```
src/
├── main.rs              # Entry point
├── cli/                 # CLI command definitions and dispatch
│   ├── mod.rs           # Clap command tree + dispatch logic
│   └── output.rs        # Table/detail output formatting
├── tui/                 # Interactive terminal browser
│   ├── mod.rs           # Event loop, terminal setup
│   ├── app.rs           # App state, messages, update logic (TEA)
│   ├── event.rs         # Key event → message mapping
│   └── ui.rs            # Rendering (tabs, lists, detail, run panel)
├── api/                 # API operation layer
│   ├── models.rs        # Model search, get, run
│   ├── agents.rs        # Agent CRUD + run
│   ├── tools.rs         # Tool search, get, delete
│   ├── integrations.rs  # Integration search, get
│   └── api_keys.rs      # API key management
├── client/              # HTTP transport
│   ├── mod.rs           # AixClient (requests, auth)
│   ├── error.rs         # Error types
│   └── polling.rs       # Async operation polling
├── config/              # Configuration + auth
│   └── mod.rs           # .env loading, key resolution
└── models/              # Data models (serde)
    ├── model.rs         # Model struct
    ├── agent.rs         # Agent + run request/result
    ├── tool.rs          # Tool struct
    ├── integration.rs   # Integration + actions
    ├── common.rs        # Page, PaginateRequest, SortField
    ├── enums.rs         # AssetStatus, Function, etc.
    ├── api_key.rs       # APIKey + usage
    └── resource.rs      # File upload types
```

## Architecture

The TUI follows the **Elm Architecture** (TEA) pattern:

```
Event (keyboard) → Message → Update (state) → Render (view)
```

All API calls are non-blocking — data loads in background tasks via `tokio::spawn` and results stream in through channels. The UI never freezes.

## Development

```bash
# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Build release
cargo build --release
```

## Design references

- CLI/TUI patterns inspired by [arimxyer/models](https://github.com/arimxyer/models)
- API surface based on [aiXplain SDK v2](https://github.com/aixplain/aiXplain/tree/main/aixplain/v2)
- See [docs/](docs/) for the RFC design documents

## License

Apache-2.0
