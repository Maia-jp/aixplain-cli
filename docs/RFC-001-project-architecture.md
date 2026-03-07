# RFC-001: Project Architecture & Foundation

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** None  

---

## Summary

Establish the Rust project structure, build system, dependency selection, module layout, and CI/CD pipeline for the aiXplain CLI — a terminal-native interface for browsing and executing Models, Tools, Integrations, and Agents on the aiXplain platform.

## Motivation

The aiXplain Python SDK v2 provides programmatic access to the platform. However, power users, DevOps engineers, and CLI-first developers need a fast, ergonomic terminal interface. Rust provides the ideal combination of startup speed, single-binary distribution, cross-platform support, and memory safety.

Inspired by [arimxyer/models](https://github.com/arimxyer/models) — a Rust CLI/TUI for browsing AI models — our CLI adapts its architectural patterns (TEA-based TUI, clap CLI, dual-mode operation) to the aiXplain platform's resource model and API surface.

## Design Principles

1. **Single binary, zero runtime dependencies.** Ship one `aix` binary. No Python, no Node, no Docker.
2. **Offline-first UX.** Cache aggressively. Show stale data immediately, refresh in the background.
3. **Composable output.** Every command supports `--json` for piping into `jq`, scripts, and other tools.
4. **Progressive disclosure.** Simple commands for common tasks, rich TUI for exploration.
5. **Fail gracefully.** Never panic in production. Degrade with clear messages.

## Project Structure

```
aixplain-cli/
├── Cargo.toml                  # Workspace root (single crate for now)
├── Cargo.lock
├── build.rs                    # Build-time version embedding
├── .github/
│   └── workflows/
│       ├── ci.yml              # Lint + test on every PR
│       └── release.yml         # Cross-platform binary builds + GitHub Releases
├── docs/
│   └── RFC-*.md                # Design documents
├── src/
│   ├── main.rs                 # Entry point, clap dispatch
│   ├── lib.rs                  # Library root (re-exports for testing)
│   │
│   ├── config/
│   │   ├── mod.rs              # Config loading, defaults, env overlay
│   │   └── auth.rs             # API key resolution chain
│   │
│   ├── client/
│   │   ├── mod.rs              # AixplainClient, request/response plumbing
│   │   ├── error.rs            # Error types, API error parsing
│   │   ├── retry.rs            # Retry policy with exponential backoff
│   │   └── polling.rs          # Async operation polling
│   │
│   ├── models/
│   │   ├── mod.rs              # Re-exports
│   │   ├── agent.rs            # Agent data model
│   │   ├── model.rs            # Model data model
│   │   ├── tool.rs             # Tool data model
│   │   ├── integration.rs      # Integration data model
│   │   ├── resource.rs         # File/Resource data model
│   │   ├── api_key.rs          # APIKey data model
│   │   ├── common.rs           # Shared types (Page, Result, Pricing, etc.)
│   │   └── enums.rs            # All enumerations (Function, Supplier, Language, etc.)
│   │
│   ├── api/
│   │   ├── mod.rs              # Re-exports
│   │   ├── agents.rs           # Agent API operations
│   │   ├── models.rs           # Model API operations
│   │   ├── tools.rs            # Tool API operations
│   │   ├── integrations.rs     # Integration API operations
│   │   ├── api_keys.rs         # APIKey API operations
│   │   └── files.rs            # File upload operations
│   │
│   ├── cli/
│   │   ├── mod.rs              # Clap definitions, subcommand dispatch
│   │   ├── agents.rs           # Agent CLI commands
│   │   ├── models.rs           # Model CLI commands
│   │   ├── tools.rs            # Tool CLI commands
│   │   ├── integrations.rs     # Integration CLI commands
│   │   ├── config_cmd.rs       # Config CLI commands
│   │   ├── output.rs           # Output formatting (table, JSON, pretty)
│   │   └── styles.rs           # TTY-aware terminal styling
│   │
│   └── tui/
│       ├── mod.rs              # TUI entry, event loop, async wiring
│       ├── app.rs              # Application state, message enum, update
│       ├── event.rs            # Key event → message mapping
│       ├── ui.rs               # Rendering (all draw functions)
│       ├── tabs/
│       │   ├── models.rs       # Models tab state + rendering
│       │   ├── agents.rs       # Agents tab state + rendering
│       │   ├── tools.rs        # Tools tab state + rendering
│       │   └── integrations.rs # Integrations tab state + rendering
│       ├── widgets/
│       │   ├── detail.rs       # Detail panel rendering
│       │   ├── search.rs       # Search bar widget
│       │   ├── table.rs        # Sortable table widget
│       │   └── status.rs       # Status bar widget
│       └── markdown.rs         # Markdown → ratatui spans
│
├── tests/
│   ├── integration/
│   │   ├── cli_test.rs         # CLI integration tests (assert_cmd)
│   │   └── api_mock_test.rs    # API mock tests (wiremock)
│   └── fixtures/
│       ├── agent.json
│       ├── model.json
│       └── paginate_response.json
│
└── .env.example                # Template for environment variables
```

## Dependencies

### Core

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.x | CLI argument parsing (derive) |
| `tokio` | 1.x | Async runtime (rt-multi-thread, macros, sync) |
| `reqwest` | 0.12.x | HTTP client (rustls-tls, json, stream) |
| `serde` | 1.x | Serialization framework (derive) |
| `serde_json` | 1.x | JSON serialization |
| `anyhow` | 1.x | Error handling with context |
| `thiserror` | 2.x | Typed error definitions |

### TUI

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.29.x | Terminal UI framework |
| `crossterm` | 0.28.x | Cross-platform terminal control |

### CLI Output

| Crate | Version | Purpose |
|-------|---------|---------|
| `comfy-table` | 7.x | Pretty table rendering |
| `console` | 0.15.x | Terminal styling and TTY detection |
| `indicatif` | 0.17.x | Progress bars and spinners |
| `dialoguer` | 0.11.x | Interactive prompts and selectors |

### Configuration

| Crate | Version | Purpose |
|-------|---------|---------|
| `toml` | 0.8.x | TOML config file parsing |
| `dirs` | 6.x | Platform-specific config directories |

### Utilities

| Crate | Version | Purpose |
|-------|---------|---------|
| `chrono` | 0.4.x | Date/time handling |
| `url` | 2.x | URL parsing and construction |
| `arboard` | 3.x | Cross-platform clipboard |
| `open` | 5.x | Open URLs in default browser |
| `mime_guess` | 2.x | MIME type detection for file uploads |
| `semver` | 1.x | Version comparison for updates |

### Dev / Test

| Crate | Version | Purpose |
|-------|---------|---------|
| `assert_cmd` | 2.x | CLI integration testing |
| `predicates` | 3.x | Assertion helpers |
| `wiremock` | 0.6.x | HTTP mock server |
| `tempfile` | 3.x | Temporary files for testing |
| `insta` | 1.x | Snapshot testing |

## Cargo.toml

```toml
[package]
name = "aixplain-cli"
version = "0.1.0"
edition = "2024"
authors = ["aiXplain Engineering"]
description = "Terminal interface for the aiXplain AI platform"
license = "Apache-2.0"
repository = "https://github.com/aixplain/aixplain-cli"
keywords = ["ai", "cli", "tui", "aixplain", "llm"]
categories = ["command-line-utilities"]

[[bin]]
name = "aix"
path = "src/main.rs"

[dependencies]
# ... (as listed above)

[dev-dependencies]
# ... (as listed above)

[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
opt-level = 3
```

## Build System

### Version Embedding

`build.rs` captures git hash and build timestamp at compile time:

```rust
fn main() {
    println!("cargo:rustc-env=BUILD_GIT_HASH={}", git_hash());
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", chrono::Utc::now().to_rfc3339());
}
```

Accessed via `env!("BUILD_GIT_HASH")` in the binary.

### Release Profile

Optimized for single-binary distribution:
- `strip = true` — Remove debug symbols
- `lto = true` — Link-time optimization for size
- `codegen-units = 1` — Maximum optimization at cost of compile time
- `panic = "abort"` — Smaller binary, no unwinding

## CI/CD Pipeline

### PR Checks (`ci.yml`)

```yaml
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

### Release (`release.yml`)

Triggered on version tags (`v*`). Builds for:

| Target | OS | Arch |
|--------|----|------|
| `x86_64-unknown-linux-gnu` | Linux | x64 |
| `aarch64-unknown-linux-gnu` | Linux | ARM64 |
| `x86_64-apple-darwin` | macOS | x64 |
| `aarch64-apple-darwin` | macOS | ARM64 (Apple Silicon) |
| `x86_64-pc-windows-msvc` | Windows | x64 |

Uses `cross` for cross-compilation. Uploads artifacts to GitHub Releases.

### Distribution Channels

1. **GitHub Releases** — Direct binary downloads
2. **Homebrew** — `brew install aixplain/tap/aix`
3. **Cargo** — `cargo install aixplain-cli`
4. **Shell script** — `curl -fsSL https://get.aixplain.com/cli | sh`

## Panic & Signal Handling

Install a custom panic hook that restores the terminal before crashing:

```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    // Restore terminal from TUI raw mode
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(std::io::stdout(), LeaveAlternateScreen);
    original_hook(info);
}));
```

Handle `SIGINT`/`SIGTERM` via `tokio::signal` for graceful shutdown.

## Module Boundaries

The architecture enforces strict layering:

```
main.rs / lib.rs
    │
    ├── cli/         ← User-facing CLI layer (clap, output formatting)
    ├── tui/         ← User-facing TUI layer (ratatui, events)
    │
    ├── api/         ← API operation layer (business logic, orchestration)
    │
    ├── client/      ← HTTP transport layer (requests, retries, polling)
    │
    ├── models/      ← Data model layer (structs, enums, serde)
    │
    └── config/      ← Configuration layer (files, env vars, auth)
```

Rules:
- `cli/` and `tui/` may call `api/` but never `client/` directly
- `api/` calls `client/` and uses `models/`
- `client/` is transport-only — no business logic
- `models/` is pure data — no I/O, no side effects
- `config/` is read at startup and threaded through via dependency injection

## Testing Strategy

| Layer | Test Type | Tools |
|-------|-----------|-------|
| `models/` | Unit tests | `#[cfg(test)]`, `serde_json` round-trip |
| `config/` | Unit tests | `tempfile`, fixture files |
| `client/` | Integration tests | `wiremock` mock server |
| `api/` | Integration tests | `wiremock` + fixture responses |
| `cli/` | CLI tests | `assert_cmd`, snapshot testing with `insta` |
| `tui/` | Manual + unit | State mutation tests, no rendering tests |

## Implementation Order

This RFC is implemented first. The binary should compile and print `aix --version` before any other RFC begins.

## Resolved Questions

1. **Workspace vs single crate?** Single crate. Split only if compile times become painful.
2. **Binary name:** `aix` — short, memorable, no conflicts.
3. **Minimum Rust version:** Track latest stable. No MSRV policy.

## Acceptance Criteria

- [ ] `cargo build` produces the `aix` binary
- [ ] `aix --version` prints version, git hash, and build timestamp
- [ ] `aix --help` shows the command tree
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] CI pipeline runs on PR
- [ ] Release pipeline produces binaries for all 5 targets
