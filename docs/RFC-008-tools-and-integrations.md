# RFC-008: Tools, Integrations & File Upload

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** RFC-003, RFC-004, RFC-005  

---

## Summary

Define operations for tools (create, run, delete), integrations (browse, connect, discover actions), and file uploads (presigned URL workflow to S3). These are the building blocks that agents use to interact with external systems.

## Motivation

Tools and integrations are how agents gain capabilities beyond pure LLM reasoning. A user might browse available integrations (GitHub, Slack, Google Sheets), connect one to create a tool, then attach that tool to an agent. The CLI must make this discovery-connection-attachment flow smooth and scriptable.

## Tools

### API Endpoints

| Operation | Method | Path |
|-----------|--------|------|
| Get | `GET` | `v2/tools/{id}` |
| Search/List | `POST` | `v2/tools/paginate` |
| Delete | `DELETE` | `v2/tools/{id}` |
| Run | `POST` | `{MODELS_RUN_URL}/{id}` |
| Create (via integration connect) | `POST` | `{MODELS_RUN_URL}/{integration_id}` |

### List/Search Tools

```rust
pub struct ToolSearchParams {
    pub query: Option<String>,
    pub ownership: Option<Ownership>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

pub async fn search_tools(
    client: &AixClient,
    params: &ToolSearchParams,
) -> Result<Page<Tool>, AixError> {
    // Tools use sortBy/sortOrder (same as Agents, not Models' sort array)
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

    client.post("v2/tools/paginate", &body).await
}
```

### Get Tool

```rust
pub async fn get_tool(
    client: &AixClient,
    id: &str,
) -> Result<Tool, AixError> {
    client.get(&format!("v2/tools/{id}")).await
}
```

### Create Tool (Code-Based)

Code-based tools are created by connecting to the script integration:

```rust
const SCRIPT_INTEGRATION_ID: &str = "686432941223092cb4294d3f";

pub struct CreateToolParams {
    pub name: String,
    pub description: String,
    pub code: String,
    pub integration_id: Option<String>,
}

pub async fn create_tool(
    client: &AixClient,
    params: &CreateToolParams,
) -> Result<Tool, AixError> {
    let integration_id = params.integration_id.as_deref()
        .unwrap_or(SCRIPT_INTEGRATION_ID);

    // Tool creation goes through integration.connect(), not v2/tools.
    // The payload is sent to {MODELS_RUN_URL}/{integration_id}.
    // Note: Tool._update() is not supported by the API (raises NotImplementedError in SDK).
    let mut data = serde_json::Map::new();
    data.insert("name".to_string(), json!(params.name));
    data.insert("description".to_string(), json!(params.description));
    data.insert("code".to_string(), json!(params.code));

    let payload = json!({
        "name": params.name,
        "description": params.description,
        "data": data
    });

    let url = client.models_url(integration_id);
    let response: serde_json::Value = client.post_url(&url, &payload).await?;

    let poll_url = response["url"].as_str()
        .ok_or_else(|| AixError::OperationFailed {
            message: "No polling URL returned for tool creation".to_string(),
            supplier_error: None,
        })?;

    let result: serde_json::Value = poll_until_complete(
        client, poll_url, &PollConfig::default(), None,
    ).await?;

    let tool_id = result["data"]["toolId"]
        .as_str()
        .ok_or_else(|| AixError::OperationFailed {
            message: "No tool ID in creation response".to_string(),
            supplier_error: None,
        })?;

    get_tool(client, tool_id).await
}
```

### Delete Tool

```rust
pub async fn delete_tool(
    client: &AixClient,
    id: &str,
) -> Result<(), AixError> {
    client.delete(&format!("v2/tools/{id}")).await
}
```

### Run Tool

```rust
pub struct ToolRunParams {
    pub action: Option<String>,
    pub data: HashMap<String, serde_json::Value>,
    pub timeout_secs: u64,
}

pub async fn run_tool(
    client: &AixClient,
    id: &str,
    params: &ToolRunParams,
) -> Result<ModelResult, AixError> {
    let tool = get_tool(client, id).await?;

    let action = resolve_tool_action(&tool, &params.action)?;

    let mut payload = serde_json::Map::new();
    payload.insert("action".to_string(), json!(action));
    payload.insert("data".to_string(), json!(params.data));

    let url = client.models_url(
        tool.asset_id.as_deref().unwrap_or(id)
    );

    let response: OperationResult = client.post_url(&url, &payload).await?;

    if let Some(poll_url) = &response.url {
        poll_until_complete(
            client, poll_url,
            &PollConfig {
                timeout: Duration::from_secs(params.timeout_secs),
                ..Default::default()
            },
            None,
        ).await
    } else {
        Ok(serde_json::from_value(response.data.unwrap_or_default())?)
    }
}

fn resolve_tool_action(tool: &Tool, requested: &Option<String>) -> Result<String, AixError> {
    let actions = tool.allowed_actions.as_deref().unwrap_or_default();

    if let Some(ref action) = requested {
        if actions.is_empty() || actions.contains(action) {
            return Ok(action.clone());
        }
        return Err(AixError::Validation(format!(
            "Action '{}' not found. Available: {:?}", action, actions
        )));
    }

    match actions.len() {
        0 => Err(AixError::Validation("Tool has no actions available".to_string())),
        1 => Ok(actions[0].clone()),
        _ => Err(AixError::Validation(format!(
            "Tool has multiple actions. Specify one with --action: {:?}", actions
        ))),
    }
}
```

### CLI: `aix tools create`

```rust
pub async fn handle_tools_create(
    client: &AixClient,
    args: &ToolsCreateArgs,
    global: &GlobalArgs,
) -> Result<()> {
    let code = if let Some(ref file) = args.code_file {
        tokio::fs::read_to_string(file).await?
    } else if let Some(ref code) = args.code {
        code.clone()
    } else {
        return Err(anyhow!("Provide code with --code or --code-file"));
    };

    let params = CreateToolParams {
        name: args.name.clone(),
        description: args.description.clone(),
        code,
        integration_id: args.integration.clone(),
    };

    let spinner = create_spinner_if_tty("Creating tool...", global);
    let tool = create_tool(client, &params).await?;
    finish_spinner(spinner);

    if global.json {
        println!("{}", serde_json::to_string_pretty(&tool)?);
    } else {
        println!("  ✓ Tool created: {} ({})",
            tool.model.base.name.as_deref().unwrap_or(""),
            tool.model.base.id.as_deref().unwrap_or("")
        );
    }

    Ok(())
}
```

### CLI: `aix tools run`

```
$ aix tools run TOOL_ID --action "SEARCH" --data query="Rust programming"

  ⠋ Running tool...

  ──────────────────────────────────
  Results:
  1. The Rust Programming Language
  2. Rust by Example
  ──────────────────────────────────
  Credits: 0.001  │  Time: 2.1s
```

## Integrations

### API Endpoints

| Operation | Method | Path |
|-----------|--------|------|
| Get | `GET` | `v2/integrations/{id}` |
| Search/List | `POST` | `v2/integrations/paginate` |
| List Actions | `POST` | `{MODELS_RUN_URL}/{id}` (action: LIST_ACTIONS) |
| List Inputs | `POST` | `{MODELS_RUN_URL}/{id}` (action: LIST_INPUTS) |
| Connect | `POST` | `{MODELS_RUN_URL}/{id}` (action: CONNECT) |

### List/Search Integrations

```rust
pub struct IntegrationSearchParams {
    pub query: Option<String>,
    pub ownership: Option<Ownership>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

pub async fn search_integrations(
    client: &AixClient,
    params: &IntegrationSearchParams,
) -> Result<Page<Integration>, AixError> {
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

    client.post("v2/integrations/paginate", &body).await
}
```

### Get Integration

```rust
pub async fn get_integration(
    client: &AixClient,
    id: &str,
) -> Result<Integration, AixError> {
    client.get(&format!("v2/integrations/{id}")).await
}
```

### List Actions

Discover what actions an integration supports:

```rust
pub async fn list_actions(
    client: &AixClient,
    integration_id: &str,
) -> Result<Vec<Action>, AixError> {
    let url = client.models_url(integration_id);

    let payload = json!({
        "action": "LIST_ACTIONS",
        "data": {}
    });

    let response: OperationResult = client.post_url(&url, &payload).await?;

    let poll_url = response.url
        .ok_or_else(|| AixError::OperationFailed {
            message: "No polling URL returned".to_string(),
            supplier_error: None,
        })?;

    let result: serde_json::Value = poll_until_complete(
        client, &poll_url, &PollConfig::default(), None,
    ).await?;

    let actions_data = result.get("data")
        .and_then(|d| d.get("items"))
        .cloned()
        .unwrap_or(json!([]));

    Ok(serde_json::from_value(actions_data)?)
}
```

### List Action Inputs

Get the required inputs for specific actions:

```rust
pub async fn list_action_inputs(
    client: &AixClient,
    integration_id: &str,
    action_names: &[String],
) -> Result<Vec<Action>, AixError> {
    let url = client.models_url(integration_id);

    let payload = json!({
        "action": "LIST_INPUTS",
        "data": {
            "actions": action_names
        }
    });

    let response: OperationResult = client.post_url(&url, &payload).await?;

    if let Some(poll_url) = &response.url {
        let result: serde_json::Value = poll_until_complete(
            client, poll_url, &PollConfig::default(), None,
        ).await?;

        let items = result.get("data")
            .and_then(|d| d.get("items"))
            .cloned()
            .unwrap_or(json!([]));

        Ok(serde_json::from_value(items)?)
    } else {
        Ok(vec![])
    }
}
```

### Connect Integration (Create Tool from Integration)

```rust
pub struct ConnectParams {
    pub name: String,
    pub description: Option<String>,
    pub actions: Vec<String>,
    pub config: HashMap<String, serde_json::Value>,
}

pub async fn connect_integration(
    client: &AixClient,
    integration_id: &str,
    params: &ConnectParams,
) -> Result<Tool, AixError> {
    let url = client.models_url(integration_id);

    let mut data = serde_json::Map::new();
    data.insert("name".to_string(), json!(params.name));
    if let Some(ref desc) = params.description {
        data.insert("description".to_string(), json!(desc));
    }
    if !params.actions.is_empty() {
        data.insert("actions".to_string(), json!(params.actions));
    }
    for (k, v) in &params.config {
        data.insert(k.clone(), v.clone());
    }

    let payload = json!({
        "action": "CONNECT",
        "data": data
    });

    let response: OperationResult = client.post_url(&url, &payload).await?;

    let poll_url = response.url
        .ok_or_else(|| AixError::OperationFailed {
            message: "No polling URL returned".to_string(),
            supplier_error: None,
        })?;

    let result: serde_json::Value = poll_until_complete(
        client, &poll_url, &PollConfig::default(), None,
    ).await?;

    let tool_id = result["data"]["toolId"].as_str()
        .or_else(|| result["data"]["assetId"].as_str())
        .ok_or_else(|| AixError::OperationFailed {
            message: "No tool ID in connect response".to_string(),
            supplier_error: None,
        })?;

    get_tool(client, tool_id).await
}
```

### CLI: `aix integrations actions`

```
$ aix integrations actions INTEGRATION_ID

  ╭──────────────────────────────────────────────────╮
  │  GitHub Integration — Available Actions           │
  ├──────────────────────────────────────────────────┤
  │                                                   │
  │  CREATE_ISSUE        Create a new issue           │
  │  LIST_ISSUES         List repository issues       │
  │  CREATE_PR           Create a pull request        │
  │  LIST_REPOS          List repositories            │
  │  SEARCH_CODE         Search code in repos         │
  │                                                   │
  ╰──────────────────────────────────────────────────╯

  5 actions available
```

### CLI: `aix integrations connect`

```
$ aix integrations connect INTEGRATION_ID \
    --name "My GitHub Tool" \
    --action CREATE_ISSUE \
    --action LIST_ISSUES \
    --config token=ghp_xxxx

  ⠋ Connecting integration...
  ✓ Tool created: My GitHub Tool (tool_abc123)
```

## File Upload

### API Endpoints

| Operation | Method | Path |
|-----------|--------|------|
| Get temp presigned URL | `POST` | `sdk/file/upload/temp-url` |
| Get perm presigned URL | `POST` | `sdk/file/upload-url` |
| Upload to S3 | `PUT` | `{presigned_url}` |

### Upload Flow

```rust
pub async fn upload_file(
    client: &AixClient,
    file_path: &str,
    temporary: bool,
) -> Result<String, AixError> {
    let path = std::path::Path::new(file_path);

    if !path.exists() {
        return Err(AixError::Upload(format!("File not found: {file_path}")));
    }

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AixError::Upload("Invalid file name".to_string()))?;

    let content_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    validate_file_size(path, &content_type)?;

    let endpoint = if temporary {
        "sdk/file/upload/temp-url"
    } else {
        "sdk/file/upload-url"
    };

    // Presigned URL requests use form data (NOT JSON) and
    // Authorization: token {api_key} header (NOT x-api-key)
    // SDK difference: temp uploads send basename, permanent uploads send full path
    let original_name = if temporary {
        file_name.to_string()
    } else {
        file_path.to_string()
    };

    let form = reqwest::multipart::Form::new()
        .text("contentType", content_type.clone())
        .text("originalName", original_name);

    let presigned: PresignedUrlResponse = client.post_form(
        endpoint,
        form,
        AuthMode::Upload,
    ).await?;

    let data = tokio::fs::read(path).await?;

    client.upload_to_s3(
        &presigned.upload_url,
        data,
        &content_type,
    ).await?;

    Ok(presigned.download_url
        .unwrap_or_else(|| format!("s3://{}", presigned.key)))
}

fn validate_file_size(path: &Path, content_type: &str) -> Result<(), AixError> {
    let size = std::fs::metadata(path)?.len();

    let max_size = if content_type.starts_with("audio/") {
        50 * 1024 * 1024   // 50 MB
    } else if content_type.starts_with("video/") {
        300 * 1024 * 1024  // 300 MB
    } else if content_type.starts_with("image/") {
        25 * 1024 * 1024   // 25 MB
    } else if content_type.starts_with("application/x-sqlite")
           || content_type.starts_with("application/vnd.ms-access") {
        300 * 1024 * 1024  // 300 MB (database)
    } else if content_type.starts_with("application/") {
        25 * 1024 * 1024   // 25 MB
    } else {
        50 * 1024 * 1024   // 50 MB (other)
    };

    if size > max_size {
        return Err(AixError::Upload(format!(
            "File too large: {} bytes (max: {} bytes for {})",
            size, max_size, content_type
        )));
    }

    Ok(())
}
```

### CLI: `aix files upload`

```
$ aix files upload ./data.csv

  ⠋ Uploading data.csv (1.2 MB)...
  ✓ Uploaded: https://aixplain-storage.s3.amazonaws.com/data.csv

$ aix files upload ./recording.mp3 --permanent

  ⠋ Uploading recording.mp3 (4.8 MB)...
  ✓ Uploaded: https://aixplain-storage.s3.amazonaws.com/recording.mp3
```

With `--json`:

```json
{
  "url": "https://aixplain-storage.s3.amazonaws.com/data.csv",
  "key": "uploads/team123/data.csv",
  "contentType": "text/csv",
  "size": 1258291
}
```

## API Keys Management

### Endpoints

| Operation | Method | Path |
|-----------|--------|------|
| List | `GET` | `sdk/api-keys` |
| Get | `GET` | `sdk/api-keys/{id}` |
| Create | `POST` | `sdk/api-keys` |
| Update | `PUT` | `sdk/api-keys/{id}` |
| Delete | `DELETE` | `sdk/api-keys/{id}` |
| Usage (key) | `GET` | `sdk/api-keys/{id}/usage-limits` |
| Usage (current) | `GET` | `sdk/api-keys/usage-limits` |

### CLI: `aix api-keys`

```
$ aix api-keys list

  ID                NAME              BUDGET    EXPIRES       ADMIN
  ────────────────────────────────────────────────────────────────
  key_abc123        Production        $100.00   2025-12-31    No
  key_def456        Development       $50.00    Never         Yes

$ aix api-keys usage

  ╭─ Usage for current key ─────────────────────╮
  │  Requests today:   142 / 1,000              │
  │  Tokens today:     45,230 / 100,000         │
  ╰──────────────────────────────────────────────╯

$ aix api-keys create \
    --name "CI Pipeline" \
    --budget 25.00 \
    --expires "2025-06-30" \
    --rpm 100 --rpd 10000

  ✓ API key created: key_xyz789
    Access key: aix_sk_abc...xyz (save this — it won't be shown again)
```

## Testing Strategy

```rust
#[tokio::test]
async fn test_create_tool_from_code() { /* ... */ }

#[tokio::test]
async fn test_run_tool_single_action() { /* ... */ }

#[tokio::test]
async fn test_run_tool_requires_action_selection() { /* ... */ }

#[tokio::test]
async fn test_list_integration_actions() { /* ... */ }

#[tokio::test]
async fn test_connect_integration() { /* ... */ }

#[tokio::test]
async fn test_file_upload_size_validation() { /* ... */ }

#[tokio::test]
async fn test_file_upload_presigned_url_flow() { /* ... */ }

#[tokio::test]
async fn test_api_key_crud() { /* ... */ }
```

## Acceptance Criteria

- [ ] `aix tools list` shows tools with pagination
- [ ] `aix tools create --name "..." --code-file script.py` creates a script tool
- [ ] `aix tools run <id> --action "..." --data key=value` executes a tool
- [ ] `aix tools delete <id>` deletes with confirmation
- [ ] `aix integrations list` shows available integrations
- [ ] `aix integrations actions <id>` shows available actions
- [ ] `aix integrations connect <id> --name "..." --action ...` creates a tool
- [ ] `aix files upload <path>` uploads and returns URL
- [ ] File size validation rejects oversized files with clear message
- [ ] `aix api-keys list/create/delete/usage` all work correctly
- [ ] All commands support `--json` output
