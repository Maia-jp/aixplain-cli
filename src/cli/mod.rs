mod output;
mod styles;

use anyhow::Result;
use clap::{Parser, Subcommand};

fn long_version() -> &'static str {
    Box::leak(
        format!(
            "{}\ncommit: {}\nbuilt:  {}",
            env!("CARGO_PKG_VERSION"),
            env!("BUILD_GIT_HASH"),
            env!("BUILD_TIMESTAMP"),
        )
        .into_boxed_str(),
    )
}

#[derive(Parser)]
#[command(
    name = "aix",
    about = "aiXplain CLI — Browse and run AI models, agents, and tools",
    version,
    long_version = long_version(),
    propagate_version = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Override API key (takes precedence over .env and environment)
    #[arg(long, global = true)]
    pub api_key: Option<String>,

    /// Use named profile
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch interactive TUI browser
    Browse,

    /// Browse and run models
    Models {
        #[command(subcommand)]
        command: ModelsCommands,
    },

    /// Manage and run agents
    Agents {
        #[command(subcommand)]
        command: AgentsCommands,
    },

    /// Manage and run tools
    Tools {
        #[command(subcommand)]
        command: ToolsCommands,
    },

    /// Browse integrations
    Integrations {
        #[command(subcommand)]
        command: IntegrationsCommands,
    },

    /// Upload files
    Files {
        #[command(subcommand)]
        command: FilesCommands,
    },

    /// Manage API keys
    #[command(name = "api-keys")]
    ApiKeys {
        #[command(subcommand)]
        command: ApiKeysCommands,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Authentication and profile management
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
}

// ── Models ──────────────────────────────────────────

#[derive(Subcommand)]
pub enum ModelsCommands {
    /// List models with optional filtering
    List {
        #[arg(short, long, help = "Search query")]
        query: Option<String>,
        #[arg(short, long, help = "Filter by function (e.g. text-generation)")]
        function: Option<Vec<String>>,
        #[arg(long, help = "Filter by supplier")]
        supplier: Option<Vec<String>>,
        #[arg(short, long, default_value = "20")]
        limit: i64,
        #[arg(short, long, default_value = "0")]
        page: i64,
        #[arg(long, help = "Sort field")]
        sort: Option<String>,
        #[arg(long, help = "Sort descending")]
        desc: bool,
    },
    /// Show model details
    Get {
        /// Model ID
        id: String,
    },
    /// Run a model
    Run {
        /// Model ID
        id: String,
        #[arg(short, long)]
        text: Option<String>,
        #[arg(short, long, help = "Input file path")]
        file: Option<String>,
        #[arg(long, help = "Read from stdin")]
        stdin: bool,
        #[arg(short, long, value_parser = parse_key_value, help = "key=value params")]
        param: Vec<(String, String)>,
        #[arg(long, default_value = "300")]
        timeout: u64,
    },
    /// Stream model output
    Stream {
        /// Model ID
        id: String,
        #[arg(short, long)]
        text: Option<String>,
        #[arg(long)]
        stdin: bool,
        #[arg(short, long, value_parser = parse_key_value)]
        param: Vec<(String, String)>,
    },
}

// ── Agents ──────────────────────────────────────────

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
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        instructions: Option<String>,
        #[arg(long)]
        instructions_file: Option<String>,
        #[arg(long, help = "LLM model ID")]
        llm: Option<String>,
        #[arg(long, help = "Tool IDs to attach")]
        tool: Vec<String>,
        #[arg(long, default_value = "5")]
        max_iterations: i32,
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
    /// Run an agent
    Run {
        id: String,
        #[arg(short, long)]
        query: Option<String>,
        #[arg(long)]
        stdin: bool,
        #[arg(long, help = "Session ID for continuity")]
        session: Option<String>,
        #[arg(long, default_value = "text")]
        output_format: String,
        #[arg(long, default_value = "300")]
        timeout: u64,
    },
    /// Interactive chat session
    Chat {
        id: String,
        #[arg(long)]
        session: Option<String>,
    },
}

// ── Tools ───────────────────────────────────────────

#[derive(Subcommand)]
pub enum ToolsCommands {
    /// List tools
    List {
        #[arg(short, long)]
        query: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: i64,
        #[arg(short, long, default_value = "0")]
        page: i64,
    },
    /// Show tool details
    Get { id: String },
    /// Create a new tool
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        description: String,
        #[arg(long, help = "Inline code")]
        code: Option<String>,
        #[arg(long, help = "Code from file")]
        code_file: Option<String>,
        #[arg(long, help = "Integration ID override")]
        integration: Option<String>,
    },
    /// Delete a tool
    Delete {
        id: String,
        #[arg(long)]
        force: bool,
    },
    /// Run a tool action
    Run {
        id: String,
        #[arg(long)]
        action: Option<String>,
        #[arg(short, long, value_parser = parse_key_value)]
        data: Vec<(String, String)>,
        #[arg(long, default_value = "300")]
        timeout: u64,
    },
}

// ── Integrations ────────────────────────────────────

#[derive(Subcommand)]
pub enum IntegrationsCommands {
    /// List integrations
    List {
        #[arg(short, long)]
        query: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: i64,
        #[arg(short, long, default_value = "0")]
        page: i64,
    },
    /// Show integration details
    Get { id: String },
    /// List available actions
    Actions { id: String },
    /// Create a tool from integration
    Connect {
        id: String,
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        action: Vec<String>,
        #[arg(long, value_parser = parse_key_value)]
        config: Vec<(String, String)>,
    },
}

// ── Files ───────────────────────────────────────────

#[derive(Subcommand)]
pub enum FilesCommands {
    /// Upload a file
    Upload {
        path: String,
        #[arg(long, help = "Permanent upload (default: temporary)")]
        permanent: bool,
    },
}

// ── API Keys ────────────────────────────────────────

#[derive(Subcommand)]
pub enum ApiKeysCommands {
    /// List API keys
    List,
    /// Show API key details
    Get { id: String },
    /// Create a new API key
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        budget: Option<f64>,
        #[arg(long)]
        expires: Option<String>,
        #[arg(long)]
        rpm: Option<i64>,
        #[arg(long)]
        rpd: Option<i64>,
    },
    /// Update an API key
    Update {
        id: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(long)]
        budget: Option<f64>,
    },
    /// Delete an API key
    Delete {
        id: String,
        #[arg(long)]
        force: bool,
    },
    /// Show usage statistics
    Usage {
        /// Key ID (omit for current key)
        id: Option<String>,
    },
}

// ── Config ──────────────────────────────────────────

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Display resolved configuration
    Show,
    /// Set a config value
    Set { key: String, value: String },
    /// Open config in $EDITOR
    Edit,
    /// Reset to defaults
    Reset,
}

// ── Auth ────────────────────────────────────────────

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Authenticate with aiXplain
    Login {
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long, default_value = "team")]
        key_type: String,
    },
    /// Remove stored credentials
    Logout,
    /// Show current auth status
    Status,
    /// Switch active profile
    Use { profile: String },
    /// List configured profiles
    List,
}

// ── Dispatch ────────────────────────────────────────

fn build_client(api_key_override: Option<&str>) -> Result<crate::client::AixClient> {
    let config = crate::config::resolve(api_key_override)?;
    crate::client::AixClient::new(&config).map_err(Into::into)
}

async fn launch_tui(api_key: Option<&str>) -> Result<()> {
    let client = std::sync::Arc::new(build_client(api_key)?);
    crate::tui::run(client).await
}

pub async fn dispatch(cli: Cli) -> Result<()> {
    let command = match cli.command {
        Some(cmd) => cmd,
        None => return launch_tui(cli.api_key.as_deref()).await,
    };

    match command {
        Commands::Browse => return launch_tui(cli.api_key.as_deref()).await,

        Commands::Models { command } => {
            let client = build_client(cli.api_key.as_deref())?;
            match command {
                ModelsCommands::List {
                    query,
                    function,
                    supplier,
                    limit,
                    page,
                    sort,
                    desc,
                } => {
                    let sort_fields = sort.map(|s| {
                        vec![crate::models::common::SortField {
                            field: s,
                            dir: if desc { -1 } else { 1 },
                        }]
                    });
                    let params = crate::api::models::ModelSearchParams {
                        query,
                        functions: function,
                        suppliers: supplier,
                        page,
                        page_size: limit,
                        sort: sort_fields,
                    };
                    let page_result = crate::api::models::search_models(&client, &params).await?;

                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&page_result)?);
                    } else {
                        output::print_models_table(&page_result);
                    }
                }
                ModelsCommands::Get { id } => {
                    let model = crate::api::models::get_model(&client, &id).await?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&model)?);
                    } else {
                        output::print_model_detail(&model);
                    }
                }
                ModelsCommands::Run {
                    id,
                    text,
                    file: _,
                    stdin,
                    param,
                    timeout,
                } => {
                    let input_text = resolve_text_input(text, stdin)?;
                    let params: std::collections::HashMap<String, serde_json::Value> = param
                        .into_iter()
                        .map(|(k, v)| (k, serde_json::Value::String(v)))
                        .collect();

                    let input = crate::api::models::ModelInput {
                        text: Some(input_text),
                        file_url: None,
                        params,
                    };

                    let spinner = maybe_spinner("Running model...", cli.json);
                    let result =
                        crate::api::models::run_model(&client, &id, &input, timeout, None).await;
                    finish_spinner(spinner);
                    let result = result?;

                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        output::print_model_result(&result);
                    }
                }
                ModelsCommands::Stream { .. } => {
                    anyhow::bail!("Streaming not yet implemented");
                }
            }
        }

        // ── Agents ──────────────────────────────
        Commands::Agents { command } => {
            let client = build_client(cli.api_key.as_deref())?;
            match command {
                AgentsCommands::List { query, limit, page } => {
                    let result =
                        crate::api::agents::search_agents(&client, query.as_deref(), page, limit)
                            .await?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        output::print_agents_table(&result);
                    }
                }
                AgentsCommands::Get { id } => {
                    let agent = crate::api::agents::get_agent(&client, &id).await?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&agent)?);
                    } else {
                        output::print_agent_detail(&agent);
                    }
                }
                AgentsCommands::Create {
                    name,
                    instructions,
                    instructions_file,
                    llm,
                    tool,
                    max_iterations,
                    max_tokens,
                } => {
                    let inst = if let Some(ref file) = instructions_file {
                        Some(tokio::fs::read_to_string(file).await?)
                    } else {
                        instructions
                    };
                    let spinner = maybe_spinner("Creating agent...", cli.json);
                    let agent = crate::api::agents::create_agent(
                        &client,
                        &name,
                        inst.as_deref(),
                        llm.as_deref(),
                        &tool,
                        max_iterations,
                        max_tokens,
                    )
                    .await;
                    finish_spinner(spinner);
                    let agent = agent?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&agent)?);
                    } else {
                        println!(
                            "  ✓ Agent created: {} ({})",
                            agent.name.as_deref().unwrap_or(""),
                            agent.id.as_deref().unwrap_or("")
                        );
                    }
                }
                AgentsCommands::Update {
                    id,
                    name,
                    instructions,
                    llm,
                    tool,
                } => {
                    let tools = if tool.is_empty() {
                        None
                    } else {
                        Some(tool.as_slice())
                    };
                    let spinner = maybe_spinner("Updating agent...", cli.json);
                    let agent = crate::api::agents::update_agent(
                        &client,
                        &id,
                        name.as_deref(),
                        instructions.as_deref(),
                        llm.as_deref(),
                        tools,
                    )
                    .await;
                    finish_spinner(spinner);
                    let agent = agent?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&agent)?);
                    } else {
                        println!("  ✓ Agent updated: {}", agent.name.as_deref().unwrap_or(""));
                    }
                }
                AgentsCommands::Delete { id, force } => {
                    if !force && output::is_tty() {
                        eprint!("  Delete agent {id}? [y/N]: ");
                        let mut confirm = String::new();
                        std::io::stdin().read_line(&mut confirm)?;
                        if !confirm.trim().eq_ignore_ascii_case("y") {
                            println!("  Cancelled.");
                            return Ok(());
                        }
                    }
                    crate::api::agents::delete_agent(&client, &id).await?;
                    if !cli.quiet {
                        println!("  ✓ Agent deleted");
                    }
                }
                AgentsCommands::Run {
                    id,
                    query,
                    stdin,
                    session,
                    output_format,
                    timeout,
                } => {
                    let query_text = resolve_text_input(query, stdin)?;
                    let spinner = maybe_spinner("Running agent...", cli.json);

                    let result = crate::api::agents::run_agent(
                        &client,
                        &id,
                        &query_text,
                        session.as_deref(),
                        &output_format,
                        timeout,
                        if spinner.is_some() {
                            Some(&|status: &str| {
                                eprint!("\r  ⠋ {status}...  ");
                            })
                        } else {
                            None
                        },
                    )
                    .await;
                    finish_spinner(spinner);
                    let result = result?;

                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        output::print_agent_result(&result);
                    }
                }
                AgentsCommands::Chat { id, session } => {
                    let agent = crate::api::agents::get_agent(&client, &id).await?;
                    let agent_name = agent.name.as_deref().unwrap_or("Agent");
                    println!("\n  ╭─ Chat with \"{agent_name}\" ─────────────────╮");
                    println!("  │  Type /quit to exit, /clear to reset.     │");
                    println!("  ╰────────────────────────────────────────────╯\n");

                    let mut session_id = session;
                    loop {
                        eprint!("  you › ");
                        use std::io::Write;
                        std::io::stderr().flush()?;
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        let input = input.trim();
                        if input.is_empty() {
                            continue;
                        }
                        match input {
                            "/quit" | "/exit" | "/q" => {
                                println!("\n  Goodbye!\n");
                                break;
                            }
                            "/clear" => {
                                session_id = None;
                                println!("  Session cleared.\n");
                                continue;
                            }
                            "/session" => {
                                println!(
                                    "  Session: {}\n",
                                    session_id.as_deref().unwrap_or("(none)")
                                );
                                continue;
                            }
                            _ => {}
                        }

                        let spinner = maybe_spinner("Thinking...", false);
                        let result = crate::api::agents::run_agent(
                            &client,
                            &id,
                            input,
                            session_id.as_deref(),
                            "text",
                            300,
                            None,
                        )
                        .await;
                        finish_spinner(spinner);

                        match result {
                            Ok(r) => {
                                println!("\n  {agent_name} › {}\n", r.output_text());
                                if let Some(sid) = r.session_id {
                                    session_id = Some(sid);
                                }
                            }
                            Err(e) => {
                                eprintln!("  Error: {e}\n");
                            }
                        }
                    }
                }
            }
        }

        // ── Tools ───────────────────────────────
        Commands::Tools { command } => {
            let client = build_client(cli.api_key.as_deref())?;
            match command {
                ToolsCommands::List { query, limit, page } => {
                    let result =
                        crate::api::tools::search_tools(&client, query.as_deref(), page, limit)
                            .await?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        output::print_tools_table(&result);
                    }
                }
                ToolsCommands::Get { id } => {
                    let tool = crate::api::tools::get_tool(&client, &id).await?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&tool)?);
                    } else {
                        output::print_tool_detail(&tool);
                    }
                }
                ToolsCommands::Delete { id, force } => {
                    if !force && output::is_tty() {
                        eprint!("  Delete tool {id}? [y/N]: ");
                        let mut confirm = String::new();
                        std::io::stdin().read_line(&mut confirm)?;
                        if !confirm.trim().eq_ignore_ascii_case("y") {
                            println!("  Cancelled.");
                            return Ok(());
                        }
                    }
                    crate::api::tools::delete_tool(&client, &id).await?;
                    if !cli.quiet {
                        println!("  ✓ Tool deleted");
                    }
                }
                ToolsCommands::Create { .. } => {
                    anyhow::bail!(
                        "Tool creation not yet implemented (requires integration connect flow)"
                    );
                }
                ToolsCommands::Run { .. } => {
                    anyhow::bail!("Tool run not yet implemented");
                }
            }
        }

        // ── Integrations ────────────────────────
        Commands::Integrations { command } => {
            let client = build_client(cli.api_key.as_deref())?;
            match command {
                IntegrationsCommands::List { query, limit, page } => {
                    let result = crate::api::integrations::search_integrations(
                        &client,
                        query.as_deref(),
                        page,
                        limit,
                    )
                    .await?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        output::print_integrations_table(&result);
                    }
                }
                IntegrationsCommands::Get { id } => {
                    let integration =
                        crate::api::integrations::get_integration(&client, &id).await?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&integration)?);
                    } else {
                        output::print_integration_detail(&integration);
                    }
                }
                IntegrationsCommands::Actions { .. } => {
                    anyhow::bail!("Integration actions not yet implemented");
                }
                IntegrationsCommands::Connect { .. } => {
                    anyhow::bail!("Integration connect not yet implemented");
                }
            }
        }

        // ── API Keys ────────────────────────────
        Commands::ApiKeys { command } => {
            let client = build_client(cli.api_key.as_deref())?;
            match command {
                ApiKeysCommands::List => {
                    let keys = crate::api::api_keys::list_api_keys(&client).await?;
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&keys)?);
                    } else {
                        output::print_api_keys_table(&keys);
                    }
                }
                ApiKeysCommands::Get { id } => {
                    let key = crate::api::api_keys::get_api_key(&client, &id).await?;
                    println!("{}", serde_json::to_string_pretty(&key)?);
                }
                ApiKeysCommands::Usage { id } => {
                    let usage = crate::api::api_keys::get_usage(&client, id.as_deref()).await?;
                    println!("{}", serde_json::to_string_pretty(&usage)?);
                }
                ApiKeysCommands::Delete { id, force } => {
                    if !force && output::is_tty() {
                        eprint!("  Delete API key {id}? [y/N]: ");
                        let mut confirm = String::new();
                        std::io::stdin().read_line(&mut confirm)?;
                        if !confirm.trim().eq_ignore_ascii_case("y") {
                            println!("  Cancelled.");
                            return Ok(());
                        }
                    }
                    crate::api::api_keys::delete_api_key(&client, &id).await?;
                    if !cli.quiet {
                        println!("  ✓ API key deleted");
                    }
                }
                ApiKeysCommands::Create { .. } => {
                    anyhow::bail!("API key creation not yet implemented");
                }
                ApiKeysCommands::Update { .. } => {
                    anyhow::bail!("API key update not yet implemented");
                }
            }
        }

        Commands::Files { .. } => anyhow::bail!("Files not yet implemented"),
        Commands::Config { .. } => anyhow::bail!("Config not yet implemented"),
        Commands::Auth { .. } => anyhow::bail!("Auth not yet implemented"),
    }
    Ok(())
}

fn resolve_text_input(text: Option<String>, stdin: bool) -> Result<String> {
    if let Some(t) = text {
        return Ok(t);
    }
    if stdin {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    anyhow::bail!("Provide input with --text, --query, or --stdin")
}

fn maybe_spinner(msg: &str, json_mode: bool) -> Option<indicatif::ProgressBar> {
    if !json_mode && output::is_tty() {
        Some(output::create_spinner(msg))
    } else {
        None
    }
}

fn finish_spinner(spinner: Option<indicatif::ProgressBar>) {
    if let Some(sp) = spinner {
        sp.finish_and_clear();
    }
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid key=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
