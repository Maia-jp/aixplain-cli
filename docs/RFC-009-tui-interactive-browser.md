# RFC-009: TUI Interactive Browser

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** RFC-001 through RFC-008  

---

## Summary

Define the terminal UI (TUI) — an interactive, keyboard-driven interface for browsing, inspecting, and running aiXplain resources. Launched via `aix` (no args) or `aix browse`, the TUI provides a rich exploration experience inspired by [arimxyer/models](https://github.com/arimxyer/models).

## Motivation

While the CLI is powerful for scripting and quick lookups, exploring the aiXplain catalog is better served by an interactive interface. The TUI lets users browse models by function, inspect agent configurations, discover integrations, and execute resources — all without leaving the terminal.

## Architecture: The Elm Architecture (TEA)

Following the pattern established by `arimxyer/models`, the TUI uses strict unidirectional data flow:

```
┌─────────┐     ┌──────────┐     ┌──────────┐
│  Event   │────▶│  Update  │────▶│  Render  │
│ (input)  │     │ (state)  │     │  (view)  │
└─────────┘     └──────────┘     └──────────┘
      ▲                                │
      └────────────────────────────────┘
                  (loop)
```

1. **Event** (`event.rs`): Keyboard/mouse input → `Message` enum
2. **Update** (`app.rs`): `Message` + `AppState` → new `AppState`
3. **Render** (`ui.rs`): `AppState` → terminal frame

No side effects in render. No state mutation outside update.

## Module Structure

```
src/tui/
├── mod.rs              # TUI entry point, event loop, async channel setup
├── app.rs              # AppState, Message enum, update logic
├── event.rs            # Key/mouse event → Message mapping
├── ui.rs               # Top-level rendering dispatch
├── tabs/
│   ├── mod.rs          # Tab enum, shared tab traits
│   ├── models.rs       # Models tab: state, rendering, filters
│   ├── agents.rs       # Agents tab: state, rendering, CRUD modals
│   ├── tools.rs        # Tools tab: state, rendering
│   └── integrations.rs # Integrations tab: state, rendering, action viewer
├── widgets/
│   ├── detail.rs       # Right-panel detail view
│   ├── search.rs       # Search input bar
│   ├── table.rs        # Reusable sortable table
│   ├── status.rs       # Bottom status bar
│   ├── modal.rs        # Modal dialog (confirm, input)
│   ├── help.rs         # Help overlay (keybindings)
│   └── run_panel.rs    # Run input/output panel
└── markdown.rs         # Markdown → ratatui Spans
```

## Application State

```rust
pub struct AppState {
    pub active_tab: Tab,
    pub models: ModelsTabState,
    pub agents: AgentsTabState,
    pub tools: ToolsTabState,
    pub integrations: IntegrationsTabState,

    pub search: SearchState,
    pub status: StatusMessage,
    pub show_help: bool,
    pub show_run_panel: bool,

    pub client: Arc<AixClient>,
    pub tx: mpsc::Sender<AsyncResult>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Models,
    Agents,
    Tools,
    Integrations,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Models, Tab::Agents, Tab::Tools, Tab::Integrations]
    }

    pub fn title(&self) -> &str {
        match self {
            Tab::Models => "Models",
            Tab::Agents => "Agents",
            Tab::Tools => "Tools",
            Tab::Integrations => "Integrations",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Tab::Models => Tab::Agents,
            Tab::Agents => Tab::Tools,
            Tab::Tools => Tab::Integrations,
            Tab::Integrations => Tab::Models,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Tab::Models => Tab::Integrations,
            Tab::Agents => Tab::Models,
            Tab::Tools => Tab::Agents,
            Tab::Integrations => Tab::Tools,
        }
    }
}
```

### Tab State (Models Example)

```rust
pub struct ModelsTabState {
    pub items: Vec<Model>,
    pub selected: usize,
    pub detail_model: Option<Model>,
    pub focus: ModelsFocus,
    pub page: i64,
    pub total: i64,
    pub loading: bool,

    pub filters: ModelFilters,
    pub sort: SortConfig,

    pub function_list: Vec<(Function, usize)>,
    pub selected_function: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ModelsFocus {
    Functions,
    List,
    Detail,
}

pub struct ModelFilters {
    pub query: Option<String>,
    pub function: Option<Function>,
    pub supplier: Option<String>,
    pub streaming_only: bool,
}

pub struct SortConfig {
    pub field: String,
    pub ascending: bool,
}
```

## Message Enum

All state changes flow through a single enum:

```rust
pub enum Message {
    // Navigation
    NextTab,
    PrevTab,
    SelectTab(Tab),

    // List navigation
    NextItem,
    PrevItem,
    PageDown,
    PageUp,
    SelectFirst,
    SelectLast,

    // Focus
    FocusLeft,
    FocusRight,

    // Search
    EnterSearch,
    ExitSearch,
    SearchInput(char),
    SearchBackspace,
    SearchSubmit,
    ClearSearch,

    // Filters (Models)
    CycleFunction,
    ToggleStreamingFilter,
    ClearFilters,

    // Sort
    CycleSort,
    ToggleSortDir,

    // Actions
    OpenDetail,
    CloseDetail,
    RunSelected,
    CopyId,
    CopyPath,
    OpenInBrowser,

    // Agent-specific
    CreateAgent,
    EditAgent,
    DeleteAgent,
    RunAgent,
    ChatWithAgent,

    // Modals
    ShowHelp,
    HideHelp,
    ConfirmAction,
    CancelAction,
    ModalInput(char),
    ModalBackspace,

    // Async results
    DataLoaded(Tab, Box<serde_json::Value>),
    DataLoadFailed(Tab, String),
    RunCompleted(Box<serde_json::Value>),
    RunFailed(String),
    RunProgress(String),

    // System
    Tick,
    Resize(u16, u16),
    Quit,
}
```

## Layout

### Main Layout

```
┌─ aiXplain ──────────────────────────────────────────┐
│  [Models]  Agents  Tools  Integrations    🔍 /:srch │
├──────────────────┬──────────────────────────────────┤
│                  │                                   │
│  Functions ▼     │  Name            Supplier  Cost   │
│  ─────────────   │  ──────────────────────────────── │
│  Text Gen (124)  │  GPT-4o          OpenAI   $0.005  │
│  Translation (45)│▶ Claude 3.5      Anthr.   $0.003  │
│  Speech Rec (32) │  Gemini Pro      Google   $0.001  │
│  TTS (28)        │  Llama 3.1       Meta     Free    │
│  Search (15)     │  Mixtral 8x22B   Mistral  $0.002  │
│                  │                                   │
│                  │                                   │
├──────────────────┴──────────────────────────────────┤
│  ↑↓ navigate  ←→ focus  ] [ tabs  / search  ? help │
│  ⏎ detail  r run  c copy  o open  q quit           │
└─────────────────────────────────────────────────────┘
```

### Detail Panel (Expanded)

When a model/agent is selected and focus moves right:

```
┌─ aiXplain ──────────────────────────────────────────┐
│  [Models]  Agents  Tools  Integrations              │
├──────────────────┬──────────────────────────────────┤
│                  │  ╭─ Claude 3.5 Sonnet ──────────╮│
│  GPT-4o          │  │                              ││
│▶ Claude 3.5      │  │  ID:     6414bd...ef1b7      ││
│  Gemini Pro      │  │  Status: ● Online            ││
│  Llama 3.1       │  │  Path:   anthropic/claude... ││
│  Mixtral         │  │  Stream: ✓ Supported         ││
│                  │  │                              ││
│                  │  │  Parameters:                  ││
│                  │  │  • text (string, required)    ││
│                  │  │  • temperature (float, 0.7)   ││
│                  │  │  • max_tokens (int, 4096)     ││
│                  │  │                              ││
│                  │  ╰──────────────────────────────╯│
├──────────────────┴──────────────────────────────────┤
│  ⏎ run  c copy ID  p copy path  o open docs        │
└─────────────────────────────────────────────────────┘
```

### Agents Tab

```
┌─ aiXplain ──────────────────────────────────────────┐
│  Models  [Agents]  Tools  Integrations              │
├──────────────────────────────────────────────────────┤
│                                                      │
│  Name              Status    Tools  LLM     Updated  │
│  ─────────────────────────────────────────────────── │
│  Research Agent    ● Online   3     GPT-4o  2h ago   │
│▶ Code Reviewer     ● Online   5     Claude  1d ago   │
│  Data Analyst      ○ Draft    2     GPT-4o  3d ago   │
│                                                      │
├──────────────────────────────────────────────────────┤
│  n new  e edit  d delete  r run  ⏎ chat  / search   │
└─────────────────────────────────────────────────────┘
```

### Run Panel (Overlay)

When `r` is pressed on a model/agent:

```
┌─ Run: Claude 3.5 Sonnet ───────────────────────────┐
│                                                      │
│  Input:                                              │
│  ┌──────────────────────────────────────────────┐   │
│  │ What is the capital of France?               │   │
│  │                                              │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  Parameters:                                         │
│  temperature: [0.7]  max_tokens: [2048]              │
│                                                      │
│  ─────────────────────────────────────────────────── │
│                                                      │
│  ⠋ Running...                                        │
│                                                      │
│  Output:                                             │
│  The capital of France is Paris.                     │
│                                                      │
│  Credits: 0.001  │  Time: 0.8s  │  Tokens: 12/32    │
│                                                      │
│  [Esc] close  [Ctrl+Enter] run  [c] copy output     │
└─────────────────────────────────────────────────────┘
```

## Event Loop

```rust
pub async fn run_tui(client: Arc<AixClient>, config: &TuiConfig) -> Result<()> {
    let mut terminal = setup_terminal()?;

    install_panic_hook();

    let (tx, mut rx) = mpsc::channel::<AsyncResult>(32);

    let mut app = AppState::new(client.clone(), tx.clone());

    spawn_initial_data_load(&app);

    let tick_rate = Duration::from_millis(config.tick_rate_ms);

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        while let Ok(result) = rx.try_recv() {
            app.handle_async_result(result);
        }

        if crossterm::event::poll(tick_rate)? {
            let event = crossterm::event::read()?;
            if let Some(msg) = event::map_event(event, &app) {
                if !app.update(msg) {
                    break;
                }
            }
        } else {
            app.update(Message::Tick);
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
    )?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Into::into)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), LeaveAlternateScreen);
        original(info);
    }));
}
```

## Key Bindings

### Global

| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `]` / `Tab` | Next tab |
| `[` / `Shift+Tab` | Previous tab |
| `/` | Enter search mode |
| `?` | Toggle help overlay |
| `Esc` | Close overlay / exit search / go back |

### List Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Next item |
| `k` / `↑` | Previous item |
| `h` / `←` | Focus left panel |
| `l` / `→` | Focus right panel / open detail |
| `g` | Go to first |
| `G` | Go to last |
| `Ctrl+D` | Page down |
| `Ctrl+U` | Page up |

### Actions

| Key | Action |
|-----|--------|
| `Enter` | Open detail / confirm |
| `r` | Run selected resource |
| `c` | Copy ID to clipboard |
| `p` | Copy path to clipboard |
| `o` | Open in browser (platform URL) |
| `s` | Cycle sort |
| `S` | Toggle sort direction |
| `f` | Cycle filter |
| `F` | Clear all filters |

### Agent-Specific

| Key | Action |
|-----|--------|
| `n` | Create new agent |
| `e` | Edit selected agent |
| `d` | Delete selected agent (with confirmation) |
| `Enter` | Start chat with agent |

### Search Mode

| Key | Action |
|-----|--------|
| Any char | Append to search |
| `Backspace` | Delete last char |
| `Enter` | Submit search |
| `Esc` | Cancel search |

## Async Data Loading

All API calls happen in background tasks. The TUI never blocks on I/O:

```rust
fn spawn_initial_data_load(app: &AppState) {
    let client = app.client.clone();
    let tx = app.tx.clone();

    tokio::spawn(async move {
        match search_models(&client, &ModelSearchParams::default()).await {
            Ok(page) => {
                tx.send(AsyncResult::ModelsLoaded(page)).await.ok();
            }
            Err(e) => {
                tx.send(AsyncResult::LoadFailed(Tab::Models, e.to_string())).await.ok();
            }
        }
    });

    let client = app.client.clone();
    let tx = app.tx.clone();
    tokio::spawn(async move {
        match search_agents(&client, &AgentSearchParams::default()).await {
            Ok(page) => {
                tx.send(AsyncResult::AgentsLoaded(page)).await.ok();
            }
            Err(e) => {
                tx.send(AsyncResult::LoadFailed(Tab::Agents, e.to_string())).await.ok();
            }
        }
    });
}

pub enum AsyncResult {
    ModelsLoaded(Page<Model>),
    AgentsLoaded(Page<Agent>),
    ToolsLoaded(Page<Tool>),
    IntegrationsLoaded(Page<Integration>),
    LoadFailed(Tab, String),
    RunCompleted(serde_json::Value),
    RunFailed(String),
    RunProgress(String),
}
```

## Search

Search is debounced and triggers API calls:

```rust
pub struct SearchState {
    pub active: bool,
    pub query: String,
    pub debounce_timer: Option<Instant>,
}

impl AppState {
    fn handle_search_submit(&mut self) {
        let query = self.search.query.clone();
        let tab = self.active_tab;
        let client = self.client.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            match tab {
                Tab::Models => {
                    let params = ModelSearchParams {
                        query: Some(query),
                        ..Default::default()
                    };
                    match search_models(&client, &params).await {
                        Ok(page) => tx.send(AsyncResult::ModelsLoaded(page)).await.ok(),
                        Err(e) => tx.send(AsyncResult::LoadFailed(tab, e.to_string())).await.ok(),
                    };
                }
                // ... similar for other tabs
            }
        });
    }
}
```

## Pagination

Infinite scroll with lazy loading:

```rust
impl AppState {
    fn check_load_more(&mut self) {
        let (items_len, total, page, loading) = match self.active_tab {
            Tab::Models => (
                self.models.items.len(),
                self.models.total as usize,
                self.models.page,
                self.models.loading,
            ),
            // ...
        };

        if loading || items_len >= total {
            return;
        }

        let threshold = items_len.saturating_sub(5);
        let selected = match self.active_tab {
            Tab::Models => self.models.selected,
            // ...
        };

        if selected >= threshold {
            self.load_next_page();
        }
    }

    fn load_next_page(&mut self) {
        // Set loading = true, increment page, spawn fetch
    }
}
```

## Status Messages

Timed feedback messages in the status bar:

```rust
pub struct StatusMessage {
    pub text: String,
    pub kind: StatusKind,
    pub expires_at: Option<Instant>,
}

pub enum StatusKind {
    Info,
    Success,
    Error,
}

impl AppState {
    fn set_status(&mut self, text: &str, kind: StatusKind) {
        self.status = StatusMessage {
            text: text.to_string(),
            kind,
            expires_at: Some(Instant::now() + Duration::from_secs(3)),
        };
    }

    fn tick_status(&mut self) {
        if let Some(expires) = self.status.expires_at {
            if Instant::now() > expires {
                self.status = StatusMessage::default();
            }
        }
    }
}
```

## Clipboard

```rust
fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text)?;

    #[cfg(target_os = "linux")]
    {
        let text = text.to_string();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            drop(text);
        });
    }

    Ok(())
}
```

## Styling

```rust
use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub header: Style,
    pub selected: Style,
    pub active_tab: Style,
    pub inactive_tab: Style,
    pub border: Style,
    pub status_ok: Style,
    pub status_err: Style,
    pub key_hint: Style,
    pub search: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            header: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            selected: Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            active_tab: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            inactive_tab: Style::default().fg(Color::Gray),
            border: Style::default().fg(Color::DarkGray),
            status_ok: Style::default().fg(Color::Green),
            status_err: Style::default().fg(Color::Red),
            key_hint: Style::default().fg(Color::DarkGray),
            search: Style::default().fg(Color::Yellow),
        }
    }
}
```

## Testing Strategy

TUI testing focuses on state mutation, not rendering:

```rust
#[test]
fn test_next_tab_wraps() {
    let mut app = test_app_state();
    app.active_tab = Tab::Integrations;
    app.update(Message::NextTab);
    assert_eq!(app.active_tab, Tab::Models);
}

#[test]
fn test_search_filters_results() {
    let mut app = test_app_state();
    app.models.items = vec![model("GPT-4o"), model("Claude"), model("Gemini")];
    app.update(Message::EnterSearch);
    app.update(Message::SearchInput('g'));
    app.update(Message::SearchInput('p'));
    app.update(Message::SearchInput('t'));
    app.update(Message::SearchSubmit);
    // Assert search query is set and fetch is triggered
}

#[test]
fn test_selection_bounds() {
    let mut app = test_app_state();
    app.models.items = vec![model("A"), model("B")];
    app.models.selected = 1;
    app.update(Message::NextItem);
    assert_eq!(app.models.selected, 1); // stays at last
}

#[test]
fn test_copy_sets_status() {
    let mut app = test_app_state();
    app.models.items = vec![model_with_id("abc123")];
    app.update(Message::CopyId);
    assert!(app.status.text.contains("Copied"));
}
```

## Acceptance Criteria

- [ ] `aix` (no args) launches the TUI
- [ ] `aix browse` launches the TUI
- [ ] Four tabs: Models, Agents, Tools, Integrations
- [ ] Tab navigation with `]`/`[`
- [ ] List navigation with `j`/`k`/`↑`/`↓`
- [ ] Detail panel shows full resource info
- [ ] Search filters results via API
- [ ] Sort cycles through relevant fields
- [ ] Pagination loads more data on scroll
- [ ] Run panel accepts input and shows output
- [ ] Agent chat works from TUI
- [ ] Copy to clipboard works cross-platform
- [ ] Status bar shows timed feedback messages
- [ ] Help overlay shows all keybindings
- [ ] Panic hook restores terminal
- [ ] No blocking I/O in the event loop
- [ ] Graceful degradation when API fails
