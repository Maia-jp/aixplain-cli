# RFC-010: TUI Agent Creation Wizard

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-08  
**Dependencies:** RFC-004, RFC-007, RFC-009  

---

## Summary

Add an agent creation and editing flow to the TUI, activated from the Agents tab. A step-by-step wizard guides the user through configuring an agent's name, instructions, LLM, tools, sub-agents, and execution parameters — then saves it via the same `POST v2/agents` / `PUT v2/agents/{id}` endpoints used by the CLI.

## Motivation

The CLI's `aix agents create` command works well for scripting, but composing an agent with multiple tools and sub-agents is cumbersome with long flag chains. The TUI can present a guided, interactive wizard where the user types a name, writes instructions, browses and picks tools from the platform, and configures parameters — all in one flow without leaving the terminal.

## Design Principles

1. **Wizard, not form.** Step-by-step progression (name → instructions → LLM → tools → params → confirm) is less overwhelming than a single form with many fields.
2. **Browse-to-attach.** Tool and sub-agent selection should use the existing list infrastructure (search, scroll, paginate) rather than requiring the user to know IDs.
3. **Match the SDK v2 exactly.** The payload sent to the API must be identical in structure to what the Python SDK sends — same keys, same defaults, same serialization.
4. **Graceful escape.** `Esc` at any step goes back to the previous step. `Esc` at the first step cancels the wizard entirely.

## Wizard Steps

### Step 1: Name

```
┌─ Create Agent ─ Step 1/6 ───────────────────────────┐
│                                                      │
│  Agent Name:                                         │
│  ┌──────────────────────────────────────────────┐   │
│  │ My Research Agent▏                           │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  Enter → next  ·  Esc → cancel                       │
└─────────────────────────────────────────────────────┘
```

- Text input, required (non-empty)
- `Enter` advances to step 2
- `Esc` cancels and returns to agent list

### Step 2: Instructions

```
┌─ Create Agent ─ Step 2/6 ───────────────────────────┐
│                                                      │
│  Instructions (system prompt):                       │
│  ┌──────────────────────────────────────────────┐   │
│  │ You are a research assistant that finds and  │   │
│  │ summarizes academic papers on a given topic. │   │
│  │ Use your tools to search and retrieve data.  │   │
│  │ ▏                                            │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  Supports {{variable}} placeholders for runtime      │
│  substitution.                                       │
│                                                      │
│  Enter → next  ·  Esc → back                         │
└─────────────────────────────────────────────────────┘
```

- Multi-line text area (shift+enter for newlines or just treat all input as single block)
- `{{variable}}` placeholders are preserved in user-facing text and converted to `{variable}` at save time (per SDK v2 regex `\{\{(\w+)\}\}` → `{\1}`)
- Optional — can be empty
- `Enter` on empty line or after content advances

### Step 3: LLM Selection

```
┌─ Create Agent ─ Step 3/6 ───────────────────────────┐
│                                                      │
│  Select LLM (the agent's brain):                     │
│                                                      │
│  ▸ Default (GPT-4o — 669a636...)    ← recommended    │
│    Custom ID: [enter model ID]                       │
│                                                      │
│  Enter → next  ·  Esc → back                         │
└─────────────────────────────────────────────────────┘
```

- Two options: use the default LLM (`669a63646eb56306647e1091`) or enter a custom model ID
- Default is pre-selected
- If custom: text input for the model ID, validated non-empty

### Step 4: Tool Attachment

```
┌─ Create Agent ─ Step 4/6 ───────────────────────────┐
│                                                      │
│  Attach tools (optional):                            │
│                                                      │
│  Selected (2):                                       │
│    ✓ Search Connection 81c (696e1045...)              │
│    ✓ SQLite Tool (690b506f...)                        │
│                                                      │
│  ──────────────────────────────────────────────────  │
│  Available tools:                 / search           │
│  ┌──────────────────────────────────────────────┐   │
│  │   SwiftSDKTest 3                              │   │
│  │ ▸ Search Connection 81c          ✓ selected   │   │
│  │   SQL Lite (7c498686)                         │   │
│  │   SQLite Tool (1762349166)                    │   │
│  │   SQLite Tool (1762349070)                    │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  Space → toggle  ·  / search  ·  Enter → next       │
│  Esc → back                                          │
└─────────────────────────────────────────────────────┘
```

- Split view: selected tools on top, browsable list below
- Uses the existing `search_tools` API with pagination
- `Space` or `Enter` toggles selection on the highlighted tool
- `/` enters search mode within the tool picker (reuses search infrastructure)
- `j/k` navigates the tool list
- Selected tools are stored as `Vec<String>` of asset IDs
- Tools and Models are interchangeable — both use `assetId` in the payload
- Skippable (just press `Enter` with nothing selected)

### Step 5: Sub-Agent Attachment

```
┌─ Create Agent ─ Step 5/6 ───────────────────────────┐
│                                                      │
│  Attach sub-agents for orchestration (optional):     │
│                                                      │
│  Selected (0): none                                  │
│                                                      │
│  ──────────────────────────────────────────────────  │
│  Available agents:                / search           │
│  ┌──────────────────────────────────────────────┐   │
│  │   Research Agent                              │   │
│  │ ▸ Code Reviewer                               │   │
│  │   Data Analyst                                │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  Space → toggle  ·  / search  ·  Enter → next       │
│  Esc → back                                          │
└─────────────────────────────────────────────────────┘
```

- Same picker pattern as tools, but browses agents via `search_agents`
- Serialized as `[{"id": "<agent_id>", "inspectors": []}]` per SDK v2
- Skippable

### Step 6: Parameters & Confirm

```
┌─ Create Agent ─ Step 6/6 ───────────────────────────┐
│                                                      │
│  Configuration:                                      │
│                                                      │
│    Max Iterations:  [5]                              │
│    Max Tokens:      [2048]                           │
│    Output Format:   text ▾                           │
│    Save as Draft:   No ▾                             │
│                                                      │
│  ──────────────────────────────────────────────────  │
│  Summary:                                            │
│    Name:         My Research Agent                    │
│    Instructions: You are a research assistant...     │
│    LLM:          Default (669a636...)                 │
│    Tools:        2 attached                          │
│    Sub-agents:   0                                   │
│                                                      │
│  Enter → Create Agent  ·  Esc → back                 │
└─────────────────────────────────────────────────────┘
```

- Editable fields: max_iterations (int), max_tokens (int), output_format (text/markdown/json cycle), as_draft (yes/no toggle)
- Read-only summary of previous steps
- `Enter` triggers the API call
- Shows spinner while saving, then success message or error

## Data Model

```rust
#[derive(Debug, Clone, Default)]
pub struct AgentWizard {
    pub visible: bool,
    pub step: WizardStep,
    pub mode: WizardMode,

    // Step 1
    pub name: String,

    // Step 2
    pub instructions: String,

    // Step 3
    pub llm_custom: bool,
    pub llm_id: String,

    // Step 4
    pub selected_tools: Vec<ToolSelection>,
    pub tool_picker: PickerState,

    // Step 5
    pub selected_subagents: Vec<AgentSelection>,
    pub agent_picker: PickerState,

    // Step 6
    pub max_iterations: i32,
    pub max_tokens: i32,
    pub output_format: OutputFormat,
    pub as_draft: bool,

    // Save state
    pub saving: bool,
    pub save_error: Option<String>,
    pub saved_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WizardStep {
    #[default]
    Name,
    Instructions,
    Llm,
    Tools,
    SubAgents,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WizardMode {
    #[default]
    Create,
    Edit,
}

#[derive(Debug, Clone)]
pub struct ToolSelection {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AgentSelection {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Markdown,
    Json,
}

#[derive(Debug, Clone, Default)]
pub struct PickerState {
    pub items: Vec<PickerItem>,
    pub selected: usize,
    pub search_query: String,
    pub searching: bool,
    pub loading: bool,
    pub page: i64,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct PickerItem {
    pub id: String,
    pub name: String,
    pub checked: bool,
}
```

## Message Extensions

New messages added to the TUI `Message` enum:

```rust
pub enum Message {
    // ... existing messages ...

    // Wizard lifecycle
    OpenWizard,
    CloseWizard,

    // Wizard navigation
    WizardNext,
    WizardBack,

    // Wizard text input (active in Name, Instructions, LLM steps)
    WizardInput(char),
    WizardBackspace,
    WizardNewline,

    // Wizard picker (active in Tools, SubAgents steps)
    WizardPickerNext,
    WizardPickerPrev,
    WizardPickerToggle,
    WizardPickerSearch,
    WizardPickerSearchInput(char),
    WizardPickerSearchBackspace,
    WizardPickerSearchSubmit,

    // Wizard confirm step
    WizardCycleField,
    WizardToggleField,

    // Wizard async
    WizardSaveComplete(String),
    WizardSaveFailed(String),
    WizardPickerLoaded(Vec<PickerItem>, i64),
}
```

## Event Mapping

When `app.wizard.visible` is true, all key events route to wizard-specific handlers:

| Step | Key | Action |
|------|-----|--------|
| Any | `Esc` | Back to previous step (or close wizard on step 1) |
| Any | `Enter` | Advance to next step (or save on confirm) |
| Name/Instructions/LLM | chars | Append to text field |
| Name/Instructions/LLM | `Backspace` | Delete last char |
| Tools/SubAgents | `j/k` | Navigate picker |
| Tools/SubAgents | `Space` | Toggle selection |
| Tools/SubAgents | `/` | Enter picker search |
| Confirm | `Tab` | Cycle between editable fields |
| Confirm | `Space` | Toggle yes/no or cycle enum |

## API Payload Construction

The wizard produces a payload matching the SDK v2's `build_save_payload()`:

```rust
fn build_wizard_payload(wizard: &AgentWizard) -> serde_json::Value {
    let instructions = convert_template_vars(&wizard.instructions);

    let tools: Vec<serde_json::Value> = wizard
        .selected_tools
        .iter()
        .map(|t| json!({"assetId": &t.id}))
        .collect();

    let subagents: Vec<serde_json::Value> = wizard
        .selected_subagents
        .iter()
        .map(|a| json!({"id": &a.id, "inspectors": []}))
        .collect();

    let llm_id = if wizard.llm_custom && !wizard.llm_id.is_empty() {
        &wizard.llm_id
    } else {
        "669a63646eb56306647e1091"
    };

    let status = if wizard.as_draft { "draft" } else { "onboarded" };

    let output_format = match wizard.output_format {
        OutputFormat::Text => "text",
        OutputFormat::Markdown => "markdown",
        OutputFormat::Json => "json",
    };

    json!({
        "name": wizard.name,
        "description": instructions,
        "instructions": instructions,
        "model": {"id": llm_id},
        "tools": tools,
        "status": status,
        "maxIterations": wizard.max_iterations,
        "maxTokens": wizard.max_tokens,
        "outputFormat": output_format,
        "agents": subagents,
        "tasks": [],
    })
}

/// Convert {{variable}} → {variable} per SDK v2 convention
fn convert_template_vars(text: &str) -> String {
    let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
    re.replace_all(text, "{$1}").to_string()
}
```

## Edit Mode

When pressing `e` on an existing agent in the Agents tab:

1. Fetch the agent's full details via `GET v2/agents/{id}`
2. Pre-populate the wizard from the response:
   - `name` → `wizard.name`
   - `instructions` → `wizard.instructions` (convert `{var}` back to `{{var}}`)
   - `model.id` → `wizard.llm_id` (set `llm_custom = true` if not default)
   - `tools` → `wizard.selected_tools` (extract id + name from each tool ref)
   - `agents` → `wizard.selected_subagents` (extract id + name)
   - `maxIterations` → `wizard.max_iterations`
   - `maxTokens` → `wizard.max_tokens`
3. Set `wizard.mode = WizardMode::Edit`
4. On save: `PUT v2/agents/{id}` instead of `POST v2/agents`

## Async Flows

All picker data loading and save operations run in background tasks:

```rust
// Load tools for picker (spawned when entering step 4)
tokio::spawn(async move {
    match search_tools(&client, query.as_deref(), page, 20).await {
        Ok(page) => {
            let items: Vec<PickerItem> = page.results.iter().map(|t| {
                PickerItem {
                    id: t.model.id.clone().unwrap_or_default(),
                    name: t.model.name.clone().unwrap_or_default(),
                    checked: false,
                }
            }).collect();
            tx.send(AsyncResult::WizardPickerLoaded(items, page.total)).await.ok();
        }
        Err(e) => { /* handle */ }
    }
});

// Save agent (spawned when Enter pressed on confirm step)
tokio::spawn(async move {
    let payload = build_wizard_payload(&wizard_snapshot);
    match client.post("v2/agents", &payload).await {
        Ok(agent) => tx.send(AsyncResult::WizardSaveComplete(agent.id)).await.ok(),
        Err(e) => tx.send(AsyncResult::WizardSaveFailed(e.to_string())).await.ok(),
    }
});
```

## Keybinding Updates

### Agents Tab (when wizard is closed)

| Key | Action |
|-----|--------|
| `n` | Open wizard in Create mode |
| `e` | Open wizard in Edit mode (pre-populated from selected agent) |
| `d` | Delete selected agent (with confirmation modal) |

### Status Bar

When on the Agents tab, the status bar should show:

```
j/k navigate  l detail  r run  n new  e edit  d delete  ]/[ tabs  / search  ? help  q quit
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn wizard_step_progression() {
    let mut w = AgentWizard::default();
    w.name = "Test".into();
    assert_eq!(w.step, WizardStep::Name);
    w.step = WizardStep::Instructions;
    assert_eq!(w.step, WizardStep::Instructions);
}

#[test]
fn wizard_back_from_first_step_closes() {
    // WizardBack on WizardStep::Name → wizard.visible = false
}

#[test]
fn wizard_payload_matches_sdk() {
    let mut w = AgentWizard::default();
    w.name = "Test Agent".into();
    w.instructions = "Help with {{topic}}".into();
    w.selected_tools = vec![ToolSelection { id: "t1".into(), name: "Search".into() }];
    w.max_iterations = 10;
    w.max_tokens = 4096;

    let payload = build_wizard_payload(&w);
    assert_eq!(payload["name"], "Test Agent");
    assert_eq!(payload["instructions"], "Help with {topic}");
    assert_eq!(payload["model"]["id"], "669a63646eb56306647e1091");
    assert_eq!(payload["tools"][0]["assetId"], "t1");
    assert_eq!(payload["maxIterations"], 10);
    assert_eq!(payload["status"], "onboarded");
}

#[test]
fn wizard_edit_mode_uses_put() {
    // mode == Edit → PUT v2/agents/{id}
}

#[test]
fn template_var_conversion() {
    assert_eq!(convert_template_vars("Hello {{name}}!"), "Hello {name}!");
    assert_eq!(convert_template_vars("No vars"), "No vars");
    assert_eq!(convert_template_vars("{{a}} and {{b}}"), "{a} and {b}");
}

#[test]
fn picker_toggle_selection() {
    let mut picker = PickerState::default();
    picker.items = vec![
        PickerItem { id: "1".into(), name: "A".into(), checked: false },
        PickerItem { id: "2".into(), name: "B".into(), checked: false },
    ];
    picker.items[0].checked = true;
    assert!(picker.items[0].checked);
    assert!(!picker.items[1].checked);
}
```

### E2E Tests

```bash
# Create agent via TUI wizard (manual test):
# 1. cargo run
# 2. Navigate to Agents tab (])
# 3. Press 'n' to open wizard
# 4. Type name → Enter
# 5. Type instructions → Enter
# 6. Select default LLM → Enter
# 7. Browse and select tools → Enter
# 8. Skip sub-agents → Enter
# 9. Review and press Enter to save
# 10. Verify agent appears in list

# Verify via CLI:
aix agents list --json | jq '.results[0]'
```

## Acceptance Criteria

- [ ] `n` on Agents tab opens the creation wizard
- [ ] `e` on Agents tab opens the edit wizard pre-populated
- [ ] Wizard progresses through 6 steps with Enter
- [ ] Wizard goes back with Esc (closes on step 1)
- [ ] Name step validates non-empty
- [ ] Instructions support multi-line input
- [ ] `{{variable}}` converted to `{variable}` at save time
- [ ] LLM defaults to `669a63646eb56306647e1091`
- [ ] Tool picker loads tools from API with search and pagination
- [ ] Sub-agent picker loads agents from API with search
- [ ] Space toggles tool/agent selection in pickers
- [ ] Confirm step shows summary of all choices
- [ ] Max iterations, max tokens, output format are editable
- [ ] Save as draft option works
- [ ] Create sends `POST v2/agents` with correct payload
- [ ] Edit sends `PUT v2/agents/{id}` with correct payload
- [ ] Payload matches SDK v2 `build_save_payload()` structure exactly
- [ ] Spinner shows during save
- [ ] Success message with agent ID shown after save
- [ ] Error displayed if save fails
- [ ] New agent appears in list after creation
- [ ] All wizard state tests pass
