# RFC-006: Model Operations

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** RFC-003, RFC-004, RFC-005  

---

## Summary

Define the API operations layer for models: listing, searching, getting, running, and streaming. Models are read-only marketplace resources — the CLI cannot create or delete them, only discover and execute.

## Motivation

Models are the core primitive of the aiXplain platform. The CLI must make it easy to find the right model for a task (by function, supplier, language), inspect its capabilities, and execute it — all from the terminal.

## API Endpoints

| Operation | Method | Path | Request Body |
|-----------|--------|------|-------------|
| Search/List | `POST` | `v2/models/paginate` | `PaginateRequest` with model-specific filters |
| Get by ID | `GET` | `v2/models/{id}` | — |
| Run (V2, async) | `POST` | `{MODELS_RUN_URL}/{id}` | Model input payload |
| Run (V1, sync fallback) | `POST` | `{MODELS_RUN_URL}` with `/v2/` replaced by `/v1/` `/{id}` | Payload with `data` key instead of `text` |
| Stream | `POST` | `{MODELS_RUN_URL}/{id}` | Model input payload with `options.stream = true` |

**V1 Fallback:** When a model has `connection_type == ["synchronous"]` only and an async run is attempted, the SDK falls back to V1 URL (`models.aixplain.com/api/v1/execute/{id}`) with `data` key instead of `text`. The CLI should handle this transparently.

## Operations

### Search/List Models

```rust
pub struct ModelSearchParams {
    pub query: Option<String>,
    pub functions: Option<Vec<Function>>,
    pub suppliers: Option<Vec<String>>,
    pub source_languages: Option<Vec<String>>,
    pub target_languages: Option<Vec<String>>,
    pub is_fine_tunable: Option<bool>,
    pub status: Option<Vec<AssetStatus>>,
    pub hosts: Option<Vec<String>>,
    pub developers: Option<Vec<String>>,
    pub sort: Option<Vec<SortField>>,
    pub page: i64,
    pub page_size: i64,
}

pub async fn search_models(
    client: &AixClient,
    params: &ModelSearchParams,
) -> Result<Page<Model>, AixError> {
    // Models use sort: [{"field":"...","dir":1}] — always required, defaults to [{}]
    let sort = params.sort.clone()
        .unwrap_or_else(|| vec![SortField::default()]);

    let mut body = PaginateRequest {
        q: params.query.clone(),
        page_number: params.page,
        page_size: params.page_size,
        sort: Some(sort),
        sort_by: None,
        sort_order: None,
        ..Default::default()
    };

    if let Some(ref functions) = params.functions {
        body.filters.insert(
            "functions".to_string(),
            serde_json::to_value(
                functions.iter().map(|f| json!({"id": f})).collect::<Vec<_>>()
            )?,
        );
    }

    if let Some(ref suppliers) = params.suppliers {
        body.filters.insert("suppliers".to_string(), serde_json::to_value(suppliers)?);
    }

    client.post("v2/models/paginate", &body).await
}
```

### Get Model

```rust
pub async fn get_model(
    client: &AixClient,
    id: &str,
) -> Result<Model, AixError> {
    client.get(&format!("v2/models/{id}")).await
}
```

### Run Model

The run flow adapts based on the model's `connection_type`:

```rust
pub async fn run_model(
    client: &AixClient,
    id: &str,
    input: ModelInput,
    config: &RunConfig,
) -> Result<ModelResult, AixError> {
    let model = get_model(client, id).await?;
    let payload = build_run_payload(&model, &input)?;

    // SDK logic: sync-only when connection_type contains "synchronous"
    // but does NOT contain "asynchronous". None → async path.
    let is_sync_only = model.connection_type
        .as_ref()
        .map(|ct| ct.contains(&"synchronous".to_string())
                 && !ct.contains(&"asynchronous".to_string()))
        .unwrap_or(false);

    if is_sync_only {
        run_sync(client, id, &payload).await
    } else {
        run_async(client, id, &payload, config).await
    }
}

async fn run_sync(
    client: &AixClient,
    id: &str,
    payload: &serde_json::Value,
) -> Result<ModelResult, AixError> {
    let url = client.models_url(id);
    client.post_url(&url, payload).await
}

async fn run_async(
    client: &AixClient,
    id: &str,
    payload: &serde_json::Value,
    config: &RunConfig,
) -> Result<ModelResult, AixError> {
    let url = client.models_url(id);
    let response: OperationResult = client.post_url(&url, payload).await?;

    let poll_url = response.url
        .ok_or_else(|| AixError::OperationFailed {
            message: "No polling URL returned".to_string(),
            supplier_error: None,
        })?;

    poll_until_complete(
        client,
        &poll_url,
        &PollConfig {
            timeout: Duration::from_secs(config.timeout_secs),
            ..Default::default()
        },
        config.on_progress.as_ref(),
    ).await
}
```

### Model Input Building

Models have typed parameters. The CLI maps user-provided key=value pairs to the correct payload shape:

```rust
pub struct ModelInput {
    pub text: Option<String>,
    pub file_url: Option<String>,
    pub params: HashMap<String, serde_json::Value>,
}

fn build_run_payload(
    model: &Model,
    input: &ModelInput,
) -> Result<serde_json::Value, AixError> {
    let mut payload = serde_json::Map::new();

    if let Some(ref text) = input.text {
        payload.insert("text".to_string(), json!(text));
    }

    if let Some(ref file_url) = input.file_url {
        payload.insert("fileUrl".to_string(), json!(file_url));
    }

    if let Some(ref params) = model.params {
        for param in params {
            if let Some(value) = input.params.get(&param.name) {
                validate_param(param, value)?;
                payload.insert(param.name.clone(), value.clone());
            } else if param.required && !param.name.eq("text") {
                if !param.default_values.is_empty() {
                    payload.insert(param.name.clone(), param.default_values[0].clone());
                } else {
                    return Err(AixError::Validation(
                        format!("Required parameter '{}' not provided", param.name)
                    ));
                }
            }
        }
    }

    for (key, value) in &input.params {
        if !payload.contains_key(key) {
            payload.insert(key.clone(), value.clone());
        }
    }

    Ok(serde_json::Value::Object(payload))
}

fn validate_param(param: &Parameter, value: &serde_json::Value) -> Result<(), AixError> {
    if !param.available_options.is_empty() {
        if !param.available_options.contains(value) {
            return Err(AixError::Validation(format!(
                "Parameter '{}' must be one of: {:?}",
                param.name, param.available_options
            )));
        }
    }
    Ok(())
}
```

### Stream Model

```rust
pub async fn stream_model(
    client: &AixClient,
    id: &str,
    input: ModelInput,
) -> Result<EventStream, AixError> {
    let model = get_model(client, id).await?;

    if model.supports_streaming == Some(false) {
        return Err(AixError::Validation(
            format!("Model '{}' does not support streaming", model.base.name.as_deref().unwrap_or(id))
        ));
    }

    let mut payload = build_run_payload(&model, &input)?;

    payload.as_object_mut().unwrap().insert(
        "options".to_string(),
        json!({"stream": true}),
    );

    let url = client.models_url(id);
    client.stream(&url, &payload).await
}
```

## CLI Handlers

### `aix models list`

```rust
pub async fn handle_models_list(
    client: &AixClient,
    args: &ModelsListArgs,
    global: &GlobalArgs,
) -> Result<()> {
    let spinner = if !global.json && is_tty() {
        Some(create_spinner("Fetching models..."))
    } else {
        None
    };

    let params = ModelSearchParams {
        query: args.query.clone(),
        functions: args.function.as_ref().map(|f| {
            f.iter().filter_map(|s| Function::from_str(s).ok()).collect()
        }),
        suppliers: args.supplier.clone(),
        page: args.page,
        page_size: args.limit,
        sort: args.sort.as_ref().map(|s| vec![SortField {
            field: s.clone(),
            dir: if args.desc { -1 } else { 1 },
        }]),
        ..Default::default()
    };

    let page = search_models(client, &params).await?;

    if let Some(sp) = spinner { sp.finish_and_clear(); }

    output(&page, global.json);
    Ok(())
}
```

### `aix models run`

```rust
pub async fn handle_models_run(
    client: &AixClient,
    args: &ModelsRunArgs,
    global: &GlobalArgs,
) -> Result<()> {
    let text = resolve_input(&args.text, &args.file, args.stdin).await?;

    let input = ModelInput {
        text: Some(text),
        file_url: None,
        params: args.param.iter().cloned().collect(),
    };

    let spinner = if !global.json && is_tty() {
        Some(create_spinner("Running model..."))
    } else {
        None
    };

    let config = RunConfig {
        timeout_secs: args.timeout,
        on_progress: spinner.as_ref().map(|sp| {
            Box::new(move |resp: &PollResponse| {
                sp.set_message(format!("Status: {}...", resp.status));
            }) as Box<dyn Fn(&PollResponse)>
        }),
    };

    let result = run_model(client, &args.id, input, &config).await?;

    if let Some(sp) = spinner { sp.finish_and_clear(); }

    if global.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_model_result(&result);
    }

    Ok(())
}
```

### `aix models stream`

```rust
pub async fn handle_models_stream(
    client: &AixClient,
    args: &ModelsStreamArgs,
    global: &GlobalArgs,
) -> Result<()> {
    let text = resolve_input(&args.text, &None, args.stdin).await?;

    let input = ModelInput {
        text: Some(text),
        file_url: None,
        params: args.param.iter().cloned().collect(),
    };

    let mut stream = stream_model(client, &args.id, input).await?;

    let mut stdout = std::io::stdout().lock();

    while let Some(event) = stream.next_event().await {
        match event {
            Ok(chunk) => {
                write!(stdout, "{}", chunk.data)?;
                stdout.flush()?;
            }
            Err(e) => {
                eprintln!("\nStream error: {e}");
                return Err(e.into());
            }
        }
    }

    writeln!(stdout)?;
    Ok(())
}
```

## Input Resolution

All run commands support multiple input sources:

```rust
async fn resolve_input(
    text: &Option<String>,
    file: &Option<String>,
    stdin: bool,
) -> Result<String, AixError> {
    if let Some(text) = text {
        return Ok(text.clone());
    }

    if let Some(path) = file {
        return tokio::fs::read_to_string(path).await
            .map_err(|e| AixError::Io(e));
    }

    if stdin || !atty::is(atty::Stream::Stdin) {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }

    Err(AixError::Validation(
        "No input provided. Use --text, --file, --stdin, or pipe input".to_string()
    ))
}
```

## File Input for Non-Text Models

Models that accept files (speech-recognition, image-classification, etc.) need the file uploaded first:

```rust
pub async fn run_model_with_file(
    client: &AixClient,
    id: &str,
    file_path: &str,
    params: HashMap<String, serde_json::Value>,
    config: &RunConfig,
) -> Result<ModelResult, AixError> {
    let s3_url = upload_file(client, file_path, true).await?;

    let input = ModelInput {
        text: None,
        file_url: Some(s3_url),
        params,
    };

    run_model(client, id, input, config).await
}
```

## Caching Strategy

Model metadata (but not run results) can be cached locally:

- `aix models list` results cached for `cache.ttl_seconds`
- `aix models get <id>` cached individually
- Cache key: `models:{query_hash}:{page}` or `model:{id}`
- Cache invalidation: TTL-based only (models rarely change)
- Cache location: `{cache_dir}/aixplain/models/`

Cache is optional and disabled by default. Enable with `aix config set cache.enabled true`.

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_search_models_with_function_filter() {
        let mock = setup_mock_server().await;
        mock_paginate(&mock, "v2/models/paginate", include_str!("fixtures/models_page.json")).await;

        let client = test_client(&mock.uri());
        let params = ModelSearchParams {
            functions: Some(vec![Function::TextGeneration]),
            ..Default::default()
        };

        let page = search_models(&client, &params).await.unwrap();
        assert!(!page.results.is_empty());
    }

    #[tokio::test]
    async fn test_run_model_sync() { /* ... */ }

    #[tokio::test]
    async fn test_run_model_async_with_polling() { /* ... */ }

    #[tokio::test]
    async fn test_stream_model_sse_parsing() { /* ... */ }

    #[tokio::test]
    async fn test_run_model_validates_params() { /* ... */ }

    #[tokio::test]
    async fn test_input_resolution_priority() { /* ... */ }
}
```

## Acceptance Criteria

- [ ] `aix models list` returns paginated results
- [ ] Filtering by function, supplier, and query works
- [ ] `aix models get <id>` shows full model details
- [ ] `aix models run <id> --text "..."` executes sync and async models correctly
- [ ] `aix models stream <id> --text "..."` streams output character-by-character
- [ ] Parameter validation rejects invalid values
- [ ] File input uploads to S3 first, then runs
- [ ] Stdin input works: `echo "hello" | aix models run <id> --stdin`
- [ ] `--json` output is valid JSON for all commands
- [ ] Models that don't support streaming return a clear error
