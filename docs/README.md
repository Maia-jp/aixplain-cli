# aiXplain CLI — RFC Index

> Production-grade terminal interface for the aiXplain AI platform.  
> Browse models, manage agents, connect integrations, and run AI — all from your terminal.

## Overview

The aiXplain CLI (`aix`) is a Rust-native command-line tool and TUI inspired by [arimxyer/models](https://github.com/arimxyer/models), built to expose the full surface of the [aiXplain SDK v2](https://github.com/aixplain/aiXplain/tree/main/aixplain/v2) API.

## Design Principles

- **Single binary, zero dependencies.** One `aix` binary. No runtime. No containers.
- **Offline-first.** Cache aggressively, show stale data, refresh in the background.
- **Composable.** Every command supports `--json` for piping, scripting, and automation.
- **Progressive disclosure.** Simple CLI for common tasks, rich TUI for exploration.
- **Fail gracefully.** Never panic. Degrade with clear messages and actionable suggestions.

## RFCs

Implementation order follows the dependency chain. Each RFC is self-contained and produces a shippable increment.

### Foundation Layer

| RFC | Title | Status | Dependencies |
|-----|-------|--------|-------------|
| [RFC-001](RFC-001-project-architecture.md) | Project Architecture & Foundation | Draft | — |
| [RFC-002](RFC-002-configuration-and-authentication.md) | Configuration & Authentication | Draft | 001 |
| [RFC-003](RFC-003-http-client-and-error-handling.md) | HTTP Client & Error Handling | Draft | 001, 002 |
| [RFC-004](RFC-004-core-data-models.md) | Core Data Models & Serialization | Draft | 001 |

### Interface Layer

| RFC | Title | Status | Dependencies |
|-----|-------|--------|-------------|
| [RFC-005](RFC-005-cli-command-interface.md) | CLI Command Interface | Draft | 001–004 |

### Operations Layer

| RFC | Title | Status | Dependencies |
|-----|-------|--------|-------------|
| [RFC-006](RFC-006-model-operations.md) | Model Operations | Draft | 003–005 |
| [RFC-007](RFC-007-agent-operations.md) | Agent Operations | Draft | 003–005 |
| [RFC-008](RFC-008-tools-and-integrations.md) | Tools, Integrations & File Upload | Draft | 003–005 |

### Experience Layer

| RFC | Title | Status | Dependencies |
|-----|-------|--------|-------------|
| [RFC-009](RFC-009-tui-interactive-browser.md) | TUI Interactive Browser | Draft | 001–008 |
| [RFC-010](RFC-010-tui-agent-creation.md) | TUI Agent Creation Wizard | Draft | 004, 007, 009 |

## Dependency Graph

```
RFC-001 (Foundation)
  │
  ├── RFC-002 (Config & Auth)
  │     │
  │     └── RFC-003 (HTTP Client)
  │           │
  │           ├── RFC-006 (Models)
  │           ├── RFC-007 (Agents)
  │           └── RFC-008 (Tools & Integrations)
  │
  └── RFC-004 (Data Models)
        │
        └── RFC-005 (CLI Interface)
              │
              └── RFC-009 (TUI Browser)
```

## Implementation Strategy

### Phase 1: Walk (Weeks 1–2)
**Goal:** `aix models list` works end-to-end.

- RFC-001: Scaffold project, CI, build
- RFC-004: Core data models (Model, Page, enums)
- RFC-002: Config file + env var auth
- RFC-003: HTTP client with retry
- RFC-005: Clap skeleton + `models list` command
- RFC-006: Model search + get operations

### Phase 2: Run (Weeks 3–4)
**Goal:** Full CLI with agents, tools, and execution.

- RFC-006: Model run + stream
- RFC-007: Agent CRUD + run + chat
- RFC-008: Tool + integration operations, file upload
- RFC-005: Remaining CLI commands

### Phase 3: Fly (Weeks 5–6)
**Goal:** TUI browser, polish, release.

- RFC-009: TUI with all four tabs
- Cross-platform testing
- Release pipeline, Homebrew tap
- Documentation and README

## Architecture Summary

```
┌────────────────────────────────────────────────────┐
│                   User Interface                    │
│  ┌──────────────────┐  ┌────────────────────────┐  │
│  │   CLI (clap)      │  │   TUI (ratatui)        │  │
│  │   commands, flags │  │   tabs, panels, keys   │  │
│  └────────┬─────────┘  └──────────┬─────────────┘  │
├───────────┴────────────────────────┴────────────────┤
│                   API Operations                    │
│  agents.rs  │  models.rs  │  tools.rs  │  files.rs  │
├─────────────────────────────────────────────────────┤
│                   HTTP Client                       │
│  requests  │  retries  │  polling  │  streaming     │
├─────────────────────────────────────────────────────┤
│                   Data Models                       │
│  Agent  │  Model  │  Tool  │  Integration  │  Enums │
├─────────────────────────────────────────────────────┤
│                   Configuration                     │
│  profiles  │  env vars  │  auth  │  .env files      │
└─────────────────────────────────────────────────────┘
```

## SDK v2 Reference Constants

Canonical values from the Python SDK that the CLI must match:

| Constant | Value | Source |
|----------|-------|--------|
| `BACKEND_URL` | `https://platform-api.aixplain.com` | `core.py` |
| `MODELS_RUN_URL` | `https://models.aixplain.com/api/v2/execute` | `core.py` |
| Default LLM | `669a63646eb56306647e1091` | `agent.py` |
| Script Integration ID | `686432941223092cb4294d3f` | `tool.py` |
| Retry total | `5` | `client.py` |
| Retry backoff factor | `0.1` | `client.py` |
| Retry status codes | `[500, 502, 503, 504]` | `client.py` |
| Retry allowed methods | `GET, POST` | `client.py` |
| Poll initial interval | `0.5s` | `resource.py` |
| Poll backoff factor | `1.1` | `resource.py` |
| Poll max interval | `60s` | `resource.py` |
| Poll timeout | `300s` | `resource.py` |
| Agent max iterations | `5` | `agent.py` |
| Agent max tokens | `2048` | `agent.py` |

### Key Edge Cases

1. **File uploads use different auth**: `Authorization: token {key}` header + form data (not JSON)
2. **Tool creation bypasses CRUD**: Goes through `integration.connect()`, not `POST v2/tools`
3. **Tool update is not supported**: SDK raises `NotImplementedError`
4. **API keys list is non-paginated**: Plain `GET sdk/api-keys` with no body
5. **Model sort is always required**: Backend expects `sort: [{}]` at minimum
6. **Non-model sort uses different keys**: Agents/Tools/Integrations use `sortBy`/`sortOrder` strings, not `sort` array
7. **Sync-only models use V1 URL**: `api/v1/execute/{id}` with `data` key instead of `text`
8. **Sync routing check**: Sync-only = `"synchronous" in connection_type AND "asynchronous" NOT in connection_type`
9. **Agent run response routing**: If `data` is a URL string → polling URL; if object → direct result
10. **Template variables**: Agent `instructions`/`description` convert `{{var}}` → `{var}` before save
11. **Permanent uploads send full path**: `originalName` is basename for temp, full path for permanent

## Tech Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust | Speed, safety, single binary |
| CLI | clap 4 (derive) | Industry standard, completions |
| TUI | ratatui + crossterm | Active ecosystem, cross-platform |
| HTTP | reqwest + tokio | Async, streaming, TLS |
| Serialization | serde + serde_json | Zero-cost, derive macros |
| Errors | thiserror + anyhow | Typed errors + context |
| Config | toml + dirs | Human-readable, XDG-compliant |
| Testing | wiremock + assert_cmd + insta | HTTP mocks, CLI testing, snapshots |
