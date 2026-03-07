# RFC-004: Core Data Models & Serialization

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** RFC-001  

---

## Summary

Define all data models, enumerations, and serialization conventions that map to the aiXplain platform API's JSON schema. This is the shared vocabulary between the API layer, CLI output, and TUI rendering.

## Motivation

The aiXplain API uses camelCase JSON with nullable fields, nested objects, and polymorphic responses. Rust's type system and serde give us a way to make illegal states unrepresentable while faithfully mapping to the API wire format.

## Serialization Conventions

### Naming

All API-facing structs use `#[serde(rename_all = "camelCase")]` to map Rust's `snake_case` to the API's `camelCase`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: Option<String>,
    pub name: Option<String>,
    pub team_id: Option<i64>,  // Wire: "teamId"
}
```

### Optional Fields

Fields that may be absent from the API response use `Option<T>` with `#[serde(default)]`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub description: Option<String>,
```

### Field Renaming

Where Rust naming conflicts with the API (e.g., `type` is a keyword):

```rust
#[serde(rename = "type", default)]
pub resource_type: Option<String>,
```

### Flattening

For base types shared across resources:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[serde(flatten)]
    pub base: BaseResource,
    pub function: Option<Function>,
    // ...
}
```

## Base Types

### BaseResource

Shared fields for all platform resources:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseResource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}
```

### OperationResult

Base result from any async operation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub status: String,
    pub completed: bool,

    #[serde(default)]
    pub error_message: Option<String>,

    #[serde(default)]
    pub supplier_error: Option<String>,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default)]
    pub result: Option<serde_json::Value>,

    #[serde(default)]
    pub data: Option<serde_json::Value>,
}
```

### Page

Generic pagination envelope:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub results: Vec<T>,
    pub total: i64,
    pub page_total: i64,
}

impl<T> Page<T> {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    pub fn has_next(&self, current_page: i64) -> bool {
        current_page + 1 < self.page_total
    }
}
```

### PaginateRequest

The platform uses two different pagination patterns:

- **Models**: `sort: [{"field":"name","dir":1}]` (always required, defaults to `[{}]`)
- **Agents/Tools/Integrations**: `sortBy: "NAME"`, `sortOrder: "ASC"`

Both share `q`, `ownership`, `pageNumber`, `pageSize`.

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,

    #[serde(default)]
    pub page_number: i64,

    #[serde(default = "default_page_size")]
    pub page_size: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership: Option<Ownership>,

    /// Used by Models paginate: array of sort objects, always required (at least [{}])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Vec<SortField>>,

    /// Used by Agents/Tools/Integrations paginate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,

    /// Used by Agents/Tools/Integrations paginate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,

    /// Resource-specific filters (functions, suppliers, etc.) merged at top level
    #[serde(flatten)]
    pub filters: serde_json::Map<String, serde_json::Value>,
}

fn default_page_size() -> i64 { 20 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortField {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub field: String,
    #[serde(default = "default_sort_dir", skip_serializing_if = "is_default_dir")]
    pub dir: i32,
}

fn default_sort_dir() -> i32 { 1 }
fn is_default_dir(d: &i32) -> bool { *d == 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Ownership {
    Private,
    Public,
    Team,
}
```

## Resource Models

### Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[serde(flatten)]
    pub base: BaseResource,

    #[serde(default)]
    pub service_name: Option<String>,

    #[serde(default)]
    pub status: Option<AssetStatus>,

    #[serde(default)]
    pub host: Option<String>,

    #[serde(default)]
    pub developer: Option<String>,

    #[serde(default)]
    pub vendor: Option<VendorInfo>,

    #[serde(default)]
    pub function: Option<ModelFunction>,

    #[serde(default)]
    pub pricing: Option<Pricing>,

    #[serde(default)]
    pub version: Option<Version>,

    #[serde(default)]
    pub function_type: Option<String>,

    #[serde(rename = "type", default)]
    pub model_type: Option<String>,

    #[serde(default)]
    pub created_at: Option<String>,

    #[serde(default)]
    pub updated_at: Option<String>,

    #[serde(default)]
    pub supports_streaming: Option<bool>,

    #[serde(default)]
    pub supports_byoc: Option<bool>,

    #[serde(default)]
    pub connection_type: Option<Vec<String>>,

    #[serde(default)]
    pub attributes: Option<Vec<Attribute>>,

    #[serde(default)]
    pub params: Option<Vec<Parameter>>,
}
```

### Model Sub-types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorInfo {
    pub name: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFunction {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pricing {
    pub price_per_unit: Option<f64>,
    pub unit_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub name: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multiple_values: bool,
    #[serde(default)]
    pub is_fixed: bool,
    #[serde(default)]
    pub data_type: Option<String>,
    #[serde(default)]
    pub data_sub_type: Option<String>,
    #[serde(default)]
    pub values: Vec<serde_json::Value>,
    #[serde(default)]
    pub default_values: Vec<serde_json::Value>,
    #[serde(default)]
    pub available_options: Vec<serde_json::Value>,
}
```

### ModelResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelResult {
    #[serde(flatten)]
    pub base: OperationResult,

    #[serde(default)]
    pub details: Option<Vec<Detail>>,

    #[serde(default)]
    pub run_time: Option<f64>,

    #[serde(default)]
    pub used_credits: Option<f64>,

    #[serde(default)]
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    pub key: Option<String>,
    pub value: Option<serde_json::Value>,
}
```

### Agent

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    #[serde(flatten)]
    pub base: BaseResource,

    #[serde(default)]
    pub instructions: Option<String>,

    #[serde(default)]
    pub status: Option<AssetStatus>,

    #[serde(default)]
    pub team_id: Option<i64>,

    #[serde(default)]
    pub tools: Option<Vec<AgentToolRef>>,

    #[serde(default)]
    pub inspector_id: Option<String>,

    #[serde(default)]
    pub supervisor_id: Option<String>,

    #[serde(default)]
    pub planner_id: Option<String>,

    #[serde(default)]
    pub tasks: Option<Vec<AgentTask>>,

    #[serde(rename = "agents", default)]
    pub subagents: Option<Vec<serde_json::Value>>,

    #[serde(default)]
    pub output_format: Option<String>,

    #[serde(default)]
    pub expected_output: Option<serde_json::Value>,

    #[serde(default)]
    pub created_at: Option<String>,

    #[serde(default)]
    pub updated_at: Option<String>,

    #[serde(default)]
    pub max_iterations: Option<i32>,

    #[serde(default)]
    pub max_tokens: Option<i32>,

    #[serde(default)]
    pub resource_info: Option<serde_json::Value>,

    #[serde(default)]
    pub inspector_targets: Option<Vec<String>>,

    #[serde(default)]
    pub max_inspectors: Option<i32>,

    #[serde(default)]
    pub inspectors: Option<Vec<serde_json::Value>>,
}
```

**Template variable conversion:** When saving agents, `{{variable}}` in `instructions` and `description` must be converted to `{variable}` before sending to the API. The SDK uses regex `\{\{(\w+)\}\}` → `{\1}`. The CLI must apply the same conversion in the save payload builder.

**Save payload post-processing:**
- `model` → always `{"id": "<llm_id>"}` (separate from the agent struct)
- `tools` → each tool serialized via its full representation (id, name, supplier, parameters, actions, etc.)
- `agents` → subagents serialized as `[{"id": "<id>", "inspectors": []}]`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentToolRef {
    #[serde(default)]
    pub id: Option<String>,

    #[serde(rename = "assetId", default)]
    pub asset_id: Option<String>,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub supplier: Option<String>,

    #[serde(default)]
    pub function: Option<String>,

    #[serde(rename = "type", default)]
    pub tool_type: Option<String>,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub parameters: Option<Vec<serde_json::Value>>,

    #[serde(default)]
    pub actions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTask {
    pub name: String,

    #[serde(rename = "description", default)]
    pub instructions: Option<String>,

    #[serde(default)]
    pub expected_output: Option<String>,

    #[serde(default)]
    pub dependencies: Vec<String>,
}
```

### AgentRunResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunResult {
    #[serde(flatten)]
    pub base: OperationResult,

    #[serde(default)]
    pub session_id: Option<String>,

    #[serde(default)]
    pub request_id: Option<String>,

    #[serde(default)]
    pub used_credits: Option<f64>,

    #[serde(default)]
    pub run_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResponseData {
    #[serde(default)]
    pub input: Option<serde_json::Value>,

    #[serde(default)]
    pub output: Option<serde_json::Value>,

    #[serde(default)]
    pub steps: Option<Vec<serde_json::Value>>,

    #[serde(default)]
    pub session_id: Option<String>,

    #[serde(default)]
    pub execution_stats: Option<serde_json::Value>,
}
```

### AgentRunRequest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunRequest {
    pub id: String,

    pub query: AgentQuery,

    #[serde(default)]
    pub execution_params: AgentExecutionParams,

    #[serde(default = "default_true")]
    pub run_response_generation: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Vec<AgentTask>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspectors: Option<Vec<serde_json::Value>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<ChatMessage>>,

    #[serde(default = "default_true")]
    pub allow_history_and_session_id: bool,

    /// Custom system prompt override for this run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Evaluation criteria for the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria: Option<String>,

    /// Evolution instructions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolve: Option<String>,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentQuery {
    pub input: String,
    #[serde(flatten)]
    pub variables: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentExecutionParams {
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: i32,
    #[serde(default = "default_max_time")]
    pub max_time: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<serde_json::Value>,
}

fn default_output_format() -> String { "text".to_string() }
fn default_max_tokens() -> i32 { 2048 }
fn default_max_iterations() -> i32 { 5 }
fn default_max_time() -> i32 { 300 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
```

### Tool

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    #[serde(flatten)]
    pub model: Model,

    #[serde(default)]
    pub asset_id: Option<String>,

    #[serde(default)]
    pub allowed_actions: Option<Vec<String>>,

    #[serde(default)]
    pub subscriptions: Option<serde_json::Value>,
}
```

### Integration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Integration {
    #[serde(flatten)]
    pub model: Model,

    #[serde(default)]
    pub actions_available: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub display_name: Option<String>,

    #[serde(default)]
    pub slug: Option<String>,

    #[serde(default)]
    pub available_versions: Option<Vec<String>>,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub input_parameters: Option<serde_json::Value>,

    #[serde(default)]
    pub output_parameters: Option<serde_json::Value>,

    #[serde(default)]
    pub tags: Option<Vec<String>>,

    #[serde(default)]
    pub no_auth: Option<bool>,

    #[serde(default)]
    pub inputs: Option<Vec<ActionInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionInput {
    pub name: String,

    #[serde(default)]
    pub code: Option<String>,

    #[serde(default)]
    pub value: Vec<serde_json::Value>,

    #[serde(default)]
    pub available_options: Vec<serde_json::Value>,

    #[serde(default = "default_datatype")]
    pub datatype: String,

    #[serde(default)]
    pub allow_multi: bool,

    #[serde(default)]
    pub required: bool,

    #[serde(default)]
    pub description: String,
}

fn default_datatype() -> String { "string".to_string() }
```

### APIKey

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    #[serde(flatten)]
    pub base: BaseResource,

    #[serde(default)]
    pub budget: Option<f64>,

    #[serde(default)]
    pub expires_at: Option<String>,

    #[serde(default)]
    pub access_key: Option<String>,

    #[serde(default)]
    pub is_admin: bool,

    #[serde(default)]
    pub global_limits: Option<ApiKeyLimits>,

    #[serde(default)]
    pub assets_limits: Vec<ApiKeyLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyLimits {
    #[serde(rename = "tpm", default)]
    pub token_per_minute: i64,
    #[serde(rename = "tpd", default)]
    pub token_per_day: i64,
    #[serde(rename = "rpm", default)]
    pub request_per_minute: i64,
    #[serde(rename = "rpd", default)]
    pub request_per_day: i64,
    #[serde(rename = "assetId", default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub token_type: Option<TokenType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyUsage {
    #[serde(rename = "requestCount", default)]
    pub daily_request_count: Option<i64>,
    #[serde(rename = "requestCountLimit", default)]
    pub daily_request_limit: Option<i64>,
    #[serde(rename = "tokenCount", default)]
    pub daily_token_count: Option<i64>,
    #[serde(rename = "tokenCountLimit", default)]
    pub daily_token_limit: Option<i64>,
    #[serde(rename = "assetId", default)]
    pub model_id: Option<String>,
}
```

### Resource (File)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    #[serde(default)]
    pub file_path: Option<String>,

    #[serde(default)]
    pub s3_url: Option<String>,

    #[serde(default)]
    pub file_type: Option<FileType>,

    #[serde(default)]
    pub is_temp: bool,
}

/// Note: Sent as form data (multipart/form-data), NOT JSON.
/// The `Authorization: token {api_key}` header is used instead of `x-api-key`.
pub struct PresignedUrlRequest {
    pub content_type: String,
    pub original_name: String,
    pub tags: Option<String>,     // Comma-separated, sent as form field
    pub license: Option<String>,  // Sent as form field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignedUrlResponse {
    pub key: String,
    pub upload_url: String,
    #[serde(default)]
    pub download_url: Option<String>,
}
```

## Enumerations

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    Draft,
    Hidden,
    Scheduled,
    Onboarding,
    Onboarded,
    Pending,
    Failed,
    Training,
    Rejected,
    Enabling,
    Deleting,
    Disabled,
    Deleted,
    InProgress,
    Completed,
    Canceling,
    Canceled,
    #[serde(alias = "deprecated_draft")]
    DeprecatedDraft,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileType {
    Audio,
    Video,
    Image,
    Text,
    Application,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TokenType {
    Input,
    Output,
    Total,
    #[serde(other)]
    Unknown,
}
```

### Function Enum

This is a large enum representing all model capabilities. Generated from the SDK's `enums_include.py`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Function {
    #[serde(rename = "text-generation")]
    TextGeneration,
    #[serde(rename = "translation")]
    Translation,
    #[serde(rename = "speech-recognition")]
    SpeechRecognition,
    #[serde(rename = "text-to-speech")]
    TextToSpeech,
    #[serde(rename = "text-summarization")]
    TextSummarization,
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "classification")]
    Classification,
    #[serde(rename = "text-to-image")]
    TextToImage,
    #[serde(rename = "speech-enhancement")]
    SpeechEnhancement,
    // ... (complete list from enums_include.py)
    #[serde(other)]
    Other,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Human-readable display names
    }
}
```

## Design Decisions

### Why `Option<T>` everywhere?

The API returns different field subsets depending on the endpoint (e.g., search results have fewer fields than `get` by ID). Using `Option<T>` with `#[serde(default)]` makes deserialization resilient to missing fields without custom deserializers.

### Why `serde_json::Value` for polymorphic fields?

Fields like `expected_output` (can be string, dict, or schema) and `data` (varies by operation type) are typed as `serde_json::Value`. This avoids complex enum dispatch for fields that the CLI displays as-is (pretty-printed JSON) rather than interpreting structurally.

### Why `#[serde(flatten)]` for base types?

Flattening keeps the JSON wire format flat (no nesting) while giving Rust code a shared base. This matches the API's actual response shape where `id`, `name`, etc. appear at the top level alongside resource-specific fields.

### Why `#[serde(other)]` on enums?

The platform may add new enum variants. `#[serde(other)]` catches unknown values into a fallback variant instead of failing deserialization. This makes the CLI forward-compatible with API changes.

## Testing Strategy

Every model struct must have:

1. **Round-trip test**: Deserialize a fixture → serialize → compare
2. **Missing fields test**: Deserialize with minimal fields (only required ones)
3. **Unknown fields test**: Deserialize with extra unknown fields (should be ignored)
4. **Display test**: Verify human-readable formatting

```rust
#[test]
fn test_agent_round_trip() {
    let json = include_str!("../../tests/fixtures/agent.json");
    let agent: Agent = serde_json::from_str(json).unwrap();
    let serialized = serde_json::to_string(&agent).unwrap();
    let deserialized: Agent = serde_json::from_str(&serialized).unwrap();
    assert_eq!(agent.base.id, deserialized.base.id);
}

#[test]
fn test_agent_minimal() {
    let json = r#"{"id": "123", "name": "test"}"#;
    let agent: Agent = serde_json::from_str(json).unwrap();
    assert_eq!(agent.base.name.as_deref(), Some("test"));
    assert!(agent.tools.is_none());
}
```

## Acceptance Criteria

- [ ] All resource types deserialize from real API responses without errors
- [ ] All resource types serialize to valid API request payloads
- [ ] Unknown enum variants are caught by `#[serde(other)]`
- [ ] Missing optional fields default to `None`
- [ ] `Page<T>` works for all resource types
- [ ] Round-trip serialization preserves all fields
- [ ] `Display` implementations produce human-readable output
