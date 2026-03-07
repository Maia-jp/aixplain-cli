# RFC-007: Agent Operations

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** RFC-003, RFC-004, RFC-005  

---

## Summary

Define the full CRUD lifecycle, execution, session management, and interactive chat for agents. Agents are the most complex resource type — they have subagents, tools, tasks, inspectors, and support stateful multi-turn conversations.

## Motivation

Agents are the primary way users orchestrate AI workflows on aiXplain. The CLI must support the complete lifecycle: create agents with tools and instructions, run them with queries, maintain conversation sessions, and debug their behavior — all without leaving the terminal.

## API Endpoints

| Operation | Method | Path | Request Body |
|-----------|--------|------|-------------|
| Create | `POST` | `v2/agents` | Agent payload |
| Update | `PUT` | `v2/agents/{id}` | Agent payload |
| Get | `GET` | `v2/agents/{id}` | — |
| Delete | `DELETE` | `v2/agents/{id}` | — |
| Search/List | `POST` | `v2/agents/paginate` | `PaginateRequest` |
| Run | `POST` | `v2/agents/{id}/run` | `AgentRunRequest` |
| Poll | `GET` | `{poll_url}` | — |

## Operations

### List/Search Agents

```rust
pub struct AgentSearchParams {
    pub query: Option<String>,
    pub ownership: Option<Ownership>,
    pub sort_by: Option<String>,     // e.g. "NAME", "CREATED_AT"
    pub sort_order: Option<String>,  // "ASC" or "DESC"
    pub page: i64,
    pub page_size: i64,
}

pub async fn search_agents(
    client: &AixClient,
    params: &AgentSearchParams,
) -> Result<Page<Agent>, AixError> {
    // Agents use sortBy/sortOrder (not the sort array that Models use)
    let body = PaginateRequest {
        q: params.query.clone(),
        page_number: params.page,
        page_size: params.page_size,
        ownership: params.ownership.clone(),
        sort: None,
        sort_by: params.sort_by.clone(),
        sort_order: params.sort_order.clone(),
        ..Default::default()
    };

    client.post("v2/agents/paginate", &body).await
}
```

### Get Agent

```rust
pub async fn get_agent(
    client: &AixClient,
    id: &str,
) -> Result<Agent, AixError> {
    client.get(&format!("v2/agents/{id}")).await
}
```

### Create Agent

```rust
pub struct CreateAgentParams {
    pub name: String,
    pub instructions: Option<String>,
    pub llm_id: Option<String>,
    pub tool_ids: Vec<String>,
    pub max_iterations: Option<i32>,
    pub max_tokens: Option<i32>,
    pub output_format: Option<String>,
    pub tasks: Option<Vec<AgentTask>>,
    pub subagent_ids: Vec<String>,
    pub as_draft: bool,
}

pub async fn create_agent(
    client: &AixClient,
    params: &CreateAgentParams,
) -> Result<Agent, AixError> {
    let tools: Vec<serde_json::Value> = params.tool_ids.iter()
        .map(|id| json!({"assetId": id}))
        .collect();

    let llm_id = params.llm_id.as_deref()
        .unwrap_or("669a63646eb56306647e1091");

    let status = if params.as_draft {
        AssetStatus::Draft
    } else {
        AssetStatus::Onboarded
    };

    let body = json!({
        "name": params.name,
        "description": params.instructions,
        "instructions": params.instructions,
        "model": {"id": llm_id},
        "tools": tools,
        "status": status,
        "maxIterations": params.max_iterations.unwrap_or(5),
        "maxTokens": params.max_tokens.unwrap_or(2048),
        "outputFormat": params.output_format.as_deref().unwrap_or("text"),
        "agents": params.subagent_ids.iter()
            .map(|id| json!({"id": id}))
            .collect::<Vec<_>>(),
        "tasks": params.tasks,
    });

    client.post("v2/agents", &body).await
}
```

### Update Agent

```rust
pub struct UpdateAgentParams {
    pub name: Option<String>,
    pub instructions: Option<String>,
    pub llm_id: Option<String>,
    pub tool_ids: Option<Vec<String>>,
    pub max_iterations: Option<i32>,
    pub max_tokens: Option<i32>,
}

pub async fn update_agent(
    client: &AixClient,
    id: &str,
    params: &UpdateAgentParams,
) -> Result<Agent, AixError> {
    let current = get_agent(client, id).await?;

    let mut body = serde_json::to_value(&current)?;
    let obj = body.as_object_mut().unwrap();

    if let Some(ref name) = params.name {
        obj.insert("name".to_string(), json!(name));
    }
    if let Some(ref instructions) = params.instructions {
        obj.insert("instructions".to_string(), json!(instructions));
        obj.insert("description".to_string(), json!(instructions));
    }
    if let Some(ref llm_id) = params.llm_id {
        obj.insert("model".to_string(), json!({"id": llm_id}));
    }
    if let Some(ref tool_ids) = params.tool_ids {
        let tools: Vec<serde_json::Value> = tool_ids.iter()
            .map(|id| json!({"assetId": id}))
            .collect();
        obj.insert("tools".to_string(), json!(tools));
    }

    client.put(&format!("v2/agents/{id}"), &body).await
}
```

### Delete Agent

```rust
pub async fn delete_agent(
    client: &AixClient,
    id: &str,
) -> Result<(), AixError> {
    client.delete(&format!("v2/agents/{id}")).await
}
```

### Run Agent

```rust
pub struct AgentRunParams {
    pub query: String,
    pub session_id: Option<String>,
    pub history: Option<Vec<ChatMessage>>,
    pub variables: Option<HashMap<String, serde_json::Value>>,
    pub output_format: Option<String>,
    pub max_tokens: Option<i32>,
    pub max_iterations: Option<i32>,
    pub timeout_secs: u64,
}

pub async fn run_agent(
    client: &AixClient,
    id: &str,
    params: &AgentRunParams,
    on_progress: Option<&dyn Fn(&PollResponse)>,
) -> Result<AgentRunResult, AixError> {
    let mut query_obj = serde_json::Map::new();
    query_obj.insert("input".to_string(), json!(&params.query));

    if let Some(ref vars) = params.variables {
        for (k, v) in vars {
            query_obj.insert(k.clone(), v.clone());
        }
    }

    let body = AgentRunRequest {
        id: id.to_string(),
        query: AgentQuery {
            input: params.query.clone(),
            variables: if let Some(ref vars) = params.variables {
                vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            } else {
                serde_json::Map::new()
            },
        },
        execution_params: AgentExecutionParams {
            output_format: params.output_format.clone()
                .unwrap_or_else(|| "text".to_string()),
            max_tokens: params.max_tokens.unwrap_or(2048),
            max_iterations: params.max_iterations.unwrap_or(5),
            max_time: params.timeout_secs as i32,
            expected_output: None,
        },
        run_response_generation: true,
        session_id: params.session_id.clone(),
        history: params.history.clone(),
        allow_history_and_session_id: true,
    };

    let response: serde_json::Value = client
        .post(&format!("v2/agents/{id}/run"), &body)
        .await?;

    let poll_url = response["data"]
        .as_str()
        .ok_or_else(|| AixError::OperationFailed {
            message: "No polling URL in response".to_string(),
            supplier_error: None,
        })?;

    poll_until_complete(
        client,
        poll_url,
        &PollConfig {
            timeout: Duration::from_secs(params.timeout_secs),
            initial_interval: Duration::from_millis(500),
            ..Default::default()
        },
        on_progress,
    ).await
}
```

## Interactive Chat

The chat command maintains a REPL-style conversation with an agent:

```rust
pub async fn agent_chat(
    client: &AixClient,
    id: &str,
    initial_session: Option<String>,
) -> Result<()> {
    let agent = get_agent(client, id).await?;
    let agent_name = agent.base.name.as_deref().unwrap_or("Agent");

    print_chat_header(agent_name);

    let mut session_id = initial_session;
    let mut history: Vec<ChatMessage> = Vec::new();

    loop {
        let input = prompt_user_input()?;

        match input.as_str() {
            "/quit" | "/exit" | "/q" => {
                println!("\n  Goodbye!");
                break;
            }
            "/clear" => {
                session_id = None;
                history.clear();
                println!("  Session cleared.\n");
                continue;
            }
            "/session" => {
                println!("  Session: {}\n",
                    session_id.as_deref().unwrap_or("(none)"));
                continue;
            }
            "/help" => {
                print_chat_help();
                continue;
            }
            _ => {}
        }

        let spinner = create_spinner("Thinking...");

        let params = AgentRunParams {
            query: input.clone(),
            session_id: session_id.clone(),
            history: Some(history.clone()),
            variables: None,
            output_format: Some("text".to_string()),
            max_tokens: None,
            max_iterations: None,
            timeout_secs: 300,
        };

        match run_agent(client, id, &params, Some(&|resp| {
            spinner.set_message(format!("{}...", resp.status));
        })).await {
            Ok(result) => {
                spinner.finish_and_clear();

                let output = extract_agent_output(&result);
                println!("\n  {agent_name} › {output}\n");

                if let Some(sid) = &result.session_id {
                    session_id = Some(sid.clone());
                }

                history.push(ChatMessage {
                    role: "user".to_string(),
                    content: input,
                });
                history.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: output,
                });
            }
            Err(e) => {
                spinner.finish_and_clear();
                eprintln!("  Error: {}\n", e.user_message());
            }
        }
    }

    Ok(())
}

fn prompt_user_input() -> Result<String> {
    use std::io::Write;
    print!("  you › ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn extract_agent_output(result: &AgentRunResult) -> String {
    if let Some(ref data) = result.base.data {
        if let Some(output) = data.get("output") {
            if let Some(s) = output.as_str() {
                return s.to_string();
            }
            return serde_json::to_string_pretty(output).unwrap_or_default();
        }
    }
    result.base.data
        .as_ref()
        .map(|d| serde_json::to_string_pretty(d).unwrap_or_default())
        .unwrap_or_else(|| "(no output)".to_string())
}
```

### Chat Session File

For long sessions, conversation history can be saved and resumed:

```rust
#[derive(Serialize, Deserialize)]
struct ChatSession {
    agent_id: String,
    session_id: Option<String>,
    history: Vec<ChatMessage>,
    created_at: String,
    updated_at: String,
}

fn session_path(agent_id: &str, session_id: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap()
        .join("aixplain")
        .join("sessions")
        .join(format!("{agent_id}_{session_id}.json"))
}
```

## CLI Handlers

### `aix agents create`

```rust
pub async fn handle_agents_create(
    client: &AixClient,
    args: &AgentsCreateArgs,
    global: &GlobalArgs,
) -> Result<()> {
    let instructions = if let Some(ref file) = args.instructions_file {
        Some(tokio::fs::read_to_string(file).await?)
    } else {
        args.instructions.clone()
    };

    let params = CreateAgentParams {
        name: args.name.clone(),
        instructions,
        llm_id: args.llm.clone(),
        tool_ids: args.tool.clone(),
        max_iterations: Some(args.max_iterations),
        max_tokens: Some(args.max_tokens),
        output_format: None,
        tasks: None,
        subagent_ids: Vec::new(),
        as_draft: false,
    };

    let spinner = create_spinner_if_tty("Creating agent...", global);

    let agent = create_agent(client, &params).await?;

    finish_spinner(spinner);

    if global.json {
        println!("{}", serde_json::to_string_pretty(&agent)?);
    } else {
        println!("  ✓ Agent created: {} ({})",
            agent.base.name.as_deref().unwrap_or(""),
            agent.base.id.as_deref().unwrap_or("")
        );
    }

    Ok(())
}
```

### `aix agents run`

```rust
pub async fn handle_agents_run(
    client: &AixClient,
    args: &AgentsRunArgs,
    global: &GlobalArgs,
) -> Result<()> {
    let query = resolve_input(&args.query, &None, args.stdin).await?;

    let spinner = create_spinner_if_tty("Running agent...", global);

    let params = AgentRunParams {
        query,
        session_id: args.session.clone(),
        history: None,
        variables: None,
        output_format: Some(args.output_format.clone()),
        max_tokens: None,
        max_iterations: None,
        timeout_secs: args.timeout,
    };

    let result = run_agent(client, &args.id, &params, Some(&|resp| {
        if let Some(ref sp) = spinner {
            sp.set_message(format!("{}...", resp.status));
        }
    })).await?;

    finish_spinner(spinner);

    if global.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_agent_result(&result);
    }

    Ok(())
}
```

### `aix agents delete`

```rust
pub async fn handle_agents_delete(
    client: &AixClient,
    args: &AgentsDeleteArgs,
    global: &GlobalArgs,
) -> Result<()> {
    if !args.force && is_tty() {
        let agent = get_agent(client, &args.id).await?;
        let name = agent.base.name.as_deref().unwrap_or(&args.id);

        if !confirm_destructive(&format!("Delete agent \"{name}\""))? {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    delete_agent(client, &args.id).await?;

    if !global.quiet {
        println!("  ✓ Agent deleted");
    }

    Ok(())
}
```

## Agent Run Result Display

```
$ aix agents run abc123 --query "Summarize the latest AI news"

  ⠋ Running agent...
  ⠸ Processing query...
  ⠼ Executing tools...

  ──────────────────────────────────────────────
  Here are the key AI developments this week:

  1. OpenAI released GPT-5 with improved...
  2. Google DeepMind announced...
  ──────────────────────────────────────────────

  Session: sid_abc123def456
  Credits: 0.15  │  Time: 12.3s  │  Steps: 3
```

With `--json`:

```json
{
  "status": "SUCCESS",
  "completed": true,
  "data": {
    "output": "Here are the key AI developments...",
    "steps": [...],
    "sessionId": "sid_abc123def456"
  },
  "sessionId": "sid_abc123def456",
  "usedCredits": 0.15,
  "runTime": 12.3
}
```

## Error Handling

| Scenario | Error | Message |
|----------|-------|---------|
| Agent not found | `AixError::NotFound` | "Agent 'xyz' not found" |
| Agent not onboarded | `AixError::Validation` | "Agent is in DRAFT status. Save it first." |
| Missing tools | `AixError::OperationFailed` | "Agent references tool 'abc' which is not found" |
| Timeout | `AixError::Timeout` | "Agent run timed out after 300s" |
| LLM failure | `AixError::OperationFailed` | Shows supplier error from poll response |

## Testing Strategy

```rust
#[tokio::test]
async fn test_create_agent() {
    let mock = setup_mock_server().await;
    mock_post(&mock, "v2/agents", 201, include_str!("fixtures/agent_created.json")).await;

    let client = test_client(&mock.uri());
    let params = CreateAgentParams {
        name: "Test Agent".to_string(),
        instructions: Some("You are helpful.".to_string()),
        ..Default::default()
    };

    let agent = create_agent(&client, &params).await.unwrap();
    assert_eq!(agent.base.name.as_deref(), Some("Test Agent"));
}

#[tokio::test]
async fn test_run_agent_with_polling() { /* ... */ }

#[tokio::test]
async fn test_update_agent_partial() { /* ... */ }

#[tokio::test]
async fn test_delete_agent() { /* ... */ }

#[tokio::test]
async fn test_chat_session_continuity() { /* ... */ }
```

## Acceptance Criteria

- [ ] `aix agents list` returns paginated agent list
- [ ] `aix agents get <id>` shows full agent details with tools and config
- [ ] `aix agents create --name "..." --instructions "..."` creates an agent
- [ ] `aix agents update <id> --name "new name"` does partial update
- [ ] `aix agents delete <id>` prompts for confirmation and deletes
- [ ] `aix agents run <id> --query "..."` runs and polls until complete
- [ ] `aix agents chat <id>` maintains interactive session with history
- [ ] Session IDs carry across turns in chat mode
- [ ] `/quit`, `/clear`, `/help` commands work in chat
- [ ] Progress feedback during long-running agent executions
- [ ] `--json` output is valid JSON for all commands
