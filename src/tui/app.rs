use crate::client::AixClient;
use crate::models::agent::Agent;
use crate::models::common::Page;
use crate::models::integration::Integration;
use crate::models::model::Model;
use crate::models::tool::Tool;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

// ── Tab ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Models,
    Agents,
    Tools,
    Integrations,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Models, Tab::Agents, Tab::Tools, Tab::Integrations];

    pub fn title(&self) -> &str {
        match self {
            Tab::Models => "Models",
            Tab::Agents => "Agents",
            Tab::Tools => "Tools",
            Tab::Integrations => "Integrations",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Tab::Models => Tab::Agents,
            Tab::Agents => Tab::Tools,
            Tab::Tools => Tab::Integrations,
            Tab::Integrations => Tab::Models,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Tab::Models => Tab::Integrations,
            Tab::Agents => Tab::Models,
            Tab::Tools => Tab::Agents,
            Tab::Integrations => Tab::Tools,
        }
    }
}

// ── List state (generic per tab) ────────────────────

#[derive(Debug, Clone)]
pub struct ListState<T: Clone> {
    pub items: Vec<T>,
    pub selected: usize,
    pub page: i64,
    pub total: i64,
    pub loading: bool,
    pub error: Option<String>,
}

impl<T: Clone> Default for ListState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            page: 0,
            total: 0,
            loading: true,
            error: None,
        }
    }
}

impl<T: Clone> ListState<T> {
    pub fn select_next(&mut self) {
        if !self.items.is_empty() && self.selected < self.items.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    pub fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.items.get(self.selected)
    }

    pub fn apply_page(&mut self, page: Page<T>) {
        self.items = page.results;
        self.total = page.total;
        self.page = 0;
        self.loading = false;
        self.selected = 0;
        self.error = None;
    }

    pub fn append_page(&mut self, page: Page<T>) {
        self.items.extend(page.results);
        self.total = page.total;
        self.loading = false;
    }

    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
        self.loading = false;
    }

    pub fn needs_more(&self) -> bool {
        !self.loading
            && (self.items.len() as i64) < self.total
            && self.selected + 5 >= self.items.len()
    }
}

// ── Focus ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Detail,
}

// ── Search ──────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub active: bool,
    pub query: String,
}

// ── Status ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub kind: StatusKind,
    pub expires_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self {
            text: String::new(),
            kind: StatusKind::Info,
            expires_at: None,
        }
    }
}

// ── Run panel ───────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RunPanel {
    pub visible: bool,
    pub resource_name: String,
    pub resource_id: String,
    pub resource_kind: RunKind,
    pub input: String,
    pub running: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub credits: Option<f64>,
    pub run_time: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    Model,
    Agent,
}

impl Default for RunPanel {
    fn default() -> Self {
        Self {
            visible: false,
            resource_name: String::new(),
            resource_id: String::new(),
            resource_kind: RunKind::Model,
            input: String::new(),
            running: false,
            output: None,
            error: None,
            credits: None,
            run_time: None,
        }
    }
}

// ── Async results ───────────────────────────────────

pub enum AsyncResult {
    ModelsLoaded(Page<Model>),
    AgentsLoaded(Page<Agent>),
    ToolsLoaded(Page<Tool>),
    IntegrationsLoaded(Page<Integration>),
    ModelsAppended(Page<Model>),
    AgentsAppended(Page<Agent>),
    ToolsAppended(Page<Tool>),
    IntegrationsAppended(Page<Integration>),
    LoadFailed(Tab, String),
    RunCompleted {
        output: String,
        credits: Option<f64>,
        run_time: Option<f64>,
    },
    RunFailed(String),
}

// ── Message ─────────────────────────────────────────

pub enum Message {
    NextTab,
    PrevTab,
    NextItem,
    PrevItem,
    PageDown,
    PageUp,
    SelectFirst,
    SelectLast,
    ToggleFocus,
    EnterSearch,
    ExitSearch,
    SearchInput(char),
    SearchBackspace,
    SearchSubmit,
    CopyId,
    ShowHelp,
    HideHelp,
    OpenRun,
    CloseRun,
    RunInput(char),
    RunBackspace,
    RunExecute,
    Tick,
    Quit,
}

// ── App ─────────────────────────────────────────────

pub struct App {
    pub active_tab: Tab,
    pub focus: Focus,
    pub models: ListState<Model>,
    pub agents: ListState<Agent>,
    pub tools: ListState<Tool>,
    pub integrations: ListState<Integration>,
    pub search: SearchState,
    pub status: StatusMessage,
    pub show_help: bool,
    pub run_panel: RunPanel,
    pub client: Arc<AixClient>,
    pub tx: mpsc::Sender<AsyncResult>,
}

impl App {
    pub fn new(client: Arc<AixClient>, tx: mpsc::Sender<AsyncResult>) -> Self {
        Self {
            active_tab: Tab::Models,
            focus: Focus::List,
            models: ListState::default(),
            agents: ListState::default(),
            tools: ListState::default(),
            integrations: ListState::default(),
            search: SearchState::default(),
            status: StatusMessage::default(),
            show_help: false,
            run_panel: RunPanel::default(),
            client,
            tx,
        }
    }

    /// Returns false to signal quit
    pub fn update(&mut self, msg: Message) -> bool {
        match msg {
            Message::Quit => return false,
            Message::NextTab => {
                self.active_tab = self.active_tab.next();
                self.focus = Focus::List;
            }
            Message::PrevTab => {
                self.active_tab = self.active_tab.prev();
                self.focus = Focus::List;
            }
            Message::NextItem => self.current_list_select_next(),
            Message::PrevItem => self.current_list_select_prev(),
            Message::PageDown => {
                for _ in 0..10 {
                    self.current_list_select_next();
                }
            }
            Message::PageUp => {
                for _ in 0..10 {
                    self.current_list_select_prev();
                }
            }
            Message::SelectFirst => self.current_list_select_first(),
            Message::SelectLast => self.current_list_select_last(),
            Message::ToggleFocus => {
                self.focus = match self.focus {
                    Focus::List => Focus::Detail,
                    Focus::Detail => Focus::List,
                };
            }
            Message::EnterSearch => {
                self.search.active = true;
                self.search.query.clear();
            }
            Message::ExitSearch => {
                self.search.active = false;
            }
            Message::SearchInput(c) => {
                self.search.query.push(c);
            }
            Message::SearchBackspace => {
                self.search.query.pop();
            }
            Message::SearchSubmit => {
                self.search.active = false;
                self.trigger_search();
            }
            Message::CopyId => {
                if let Some(id) = self.selected_id() {
                    self.set_status(&format!("Copied: {id}"), StatusKind::Success);
                }
            }
            Message::ShowHelp => self.show_help = true,
            Message::HideHelp => self.show_help = false,
            Message::OpenRun => self.open_run_panel(),
            Message::CloseRun => {
                self.run_panel = RunPanel::default();
            }
            Message::RunInput(c) => {
                if self.run_panel.visible && !self.run_panel.running {
                    self.run_panel.input.push(c);
                }
            }
            Message::RunBackspace => {
                if self.run_panel.visible && !self.run_panel.running {
                    self.run_panel.input.pop();
                }
            }
            Message::RunExecute => {
                if self.run_panel.visible
                    && !self.run_panel.running
                    && !self.run_panel.input.is_empty()
                {
                    self.execute_run();
                }
            }
            Message::Tick => self.tick_status(),
        }

        self.check_load_more();
        true
    }

    pub fn handle_async(&mut self, result: AsyncResult) {
        match result {
            AsyncResult::ModelsLoaded(p) => self.models.apply_page(p),
            AsyncResult::AgentsLoaded(p) => self.agents.apply_page(p),
            AsyncResult::ToolsLoaded(p) => self.tools.apply_page(p),
            AsyncResult::IntegrationsLoaded(p) => self.integrations.apply_page(p),
            AsyncResult::ModelsAppended(p) => self.models.append_page(p),
            AsyncResult::AgentsAppended(p) => self.agents.append_page(p),
            AsyncResult::ToolsAppended(p) => self.tools.append_page(p),
            AsyncResult::IntegrationsAppended(p) => self.integrations.append_page(p),
            AsyncResult::LoadFailed(tab, msg) => match tab {
                Tab::Models => self.models.set_error(msg),
                Tab::Agents => self.agents.set_error(msg),
                Tab::Tools => self.tools.set_error(msg),
                Tab::Integrations => self.integrations.set_error(msg),
            },
            AsyncResult::RunCompleted {
                output,
                credits,
                run_time,
            } => {
                self.run_panel.running = false;
                self.run_panel.output = Some(output);
                self.run_panel.credits = credits;
                self.run_panel.run_time = run_time;
            }
            AsyncResult::RunFailed(msg) => {
                self.run_panel.running = false;
                self.run_panel.error = Some(msg);
            }
        }
    }

    fn current_list_select_next(&mut self) {
        match self.active_tab {
            Tab::Models => self.models.select_next(),
            Tab::Agents => self.agents.select_next(),
            Tab::Tools => self.tools.select_next(),
            Tab::Integrations => self.integrations.select_next(),
        }
    }

    fn current_list_select_prev(&mut self) {
        match self.active_tab {
            Tab::Models => self.models.select_prev(),
            Tab::Agents => self.agents.select_prev(),
            Tab::Tools => self.tools.select_prev(),
            Tab::Integrations => self.integrations.select_prev(),
        }
    }

    fn current_list_select_first(&mut self) {
        match self.active_tab {
            Tab::Models => self.models.select_first(),
            Tab::Agents => self.agents.select_first(),
            Tab::Tools => self.tools.select_first(),
            Tab::Integrations => self.integrations.select_first(),
        }
    }

    fn current_list_select_last(&mut self) {
        match self.active_tab {
            Tab::Models => self.models.select_last(),
            Tab::Agents => self.agents.select_last(),
            Tab::Tools => self.tools.select_last(),
            Tab::Integrations => self.integrations.select_last(),
        }
    }

    fn selected_id(&self) -> Option<String> {
        match self.active_tab {
            Tab::Models => self.models.selected_item().and_then(|m| m.id.clone()),
            Tab::Agents => self.agents.selected_item().and_then(|a| a.id.clone()),
            Tab::Tools => self.tools.selected_item().and_then(|t| t.model.id.clone()),
            Tab::Integrations => self
                .integrations
                .selected_item()
                .and_then(|i| i.model.id.clone()),
        }
    }

    fn trigger_search(&mut self) {
        let query = if self.search.query.is_empty() {
            None
        } else {
            Some(self.search.query.clone())
        };
        let tab = self.active_tab;
        let client = self.client.clone();
        let tx = self.tx.clone();

        match tab {
            Tab::Models => {
                self.models.loading = true;
                tokio::spawn(async move {
                    let params = crate::api::models::ModelSearchParams {
                        query,
                        ..Default::default()
                    };
                    match crate::api::models::search_models(&client, &params).await {
                        Ok(p) => tx.send(AsyncResult::ModelsLoaded(p)).await.ok(),
                        Err(e) => tx
                            .send(AsyncResult::LoadFailed(tab, e.to_string()))
                            .await
                            .ok(),
                    };
                });
            }
            Tab::Agents => {
                self.agents.loading = true;
                tokio::spawn(async move {
                    match crate::api::agents::search_agents(&client, query.as_deref(), 0, 20).await
                    {
                        Ok(p) => tx.send(AsyncResult::AgentsLoaded(p)).await.ok(),
                        Err(e) => tx
                            .send(AsyncResult::LoadFailed(tab, e.to_string()))
                            .await
                            .ok(),
                    };
                });
            }
            Tab::Tools => {
                self.tools.loading = true;
                tokio::spawn(async move {
                    match crate::api::tools::search_tools(&client, query.as_deref(), 0, 20).await {
                        Ok(p) => tx.send(AsyncResult::ToolsLoaded(p)).await.ok(),
                        Err(e) => tx
                            .send(AsyncResult::LoadFailed(tab, e.to_string()))
                            .await
                            .ok(),
                    };
                });
            }
            Tab::Integrations => {
                self.integrations.loading = true;
                tokio::spawn(async move {
                    match crate::api::integrations::search_integrations(
                        &client,
                        query.as_deref(),
                        0,
                        20,
                    )
                    .await
                    {
                        Ok(p) => tx.send(AsyncResult::IntegrationsLoaded(p)).await.ok(),
                        Err(e) => tx
                            .send(AsyncResult::LoadFailed(tab, e.to_string()))
                            .await
                            .ok(),
                    };
                });
            }
        }
    }

    fn check_load_more(&mut self) {
        let tab = self.active_tab;
        let needs = match tab {
            Tab::Models => self.models.needs_more(),
            Tab::Agents => self.agents.needs_more(),
            Tab::Tools => self.tools.needs_more(),
            Tab::Integrations => self.integrations.needs_more(),
        };

        if !needs {
            return;
        }

        let next_page = match tab {
            Tab::Models => {
                self.models.loading = true;
                self.models.page += 1;
                self.models.page
            }
            Tab::Agents => {
                self.agents.loading = true;
                self.agents.page += 1;
                self.agents.page
            }
            Tab::Tools => {
                self.tools.loading = true;
                self.tools.page += 1;
                self.tools.page
            }
            Tab::Integrations => {
                self.integrations.loading = true;
                self.integrations.page += 1;
                self.integrations.page
            }
        };

        let client = self.client.clone();
        let tx = self.tx.clone();
        let q = if self.search.query.is_empty() {
            None
        } else {
            Some(self.search.query.clone())
        };

        tokio::spawn(async move {
            match tab {
                Tab::Models => {
                    let params = crate::api::models::ModelSearchParams {
                        query: q,
                        page: next_page,
                        ..Default::default()
                    };
                    match crate::api::models::search_models(&client, &params).await {
                        Ok(p) => tx.send(AsyncResult::ModelsAppended(p)).await.ok(),
                        Err(e) => tx
                            .send(AsyncResult::LoadFailed(tab, e.to_string()))
                            .await
                            .ok(),
                    };
                }
                Tab::Agents => {
                    match crate::api::agents::search_agents(&client, q.as_deref(), next_page, 20)
                        .await
                    {
                        Ok(p) => tx.send(AsyncResult::AgentsAppended(p)).await.ok(),
                        Err(e) => tx
                            .send(AsyncResult::LoadFailed(tab, e.to_string()))
                            .await
                            .ok(),
                    };
                }
                Tab::Tools => {
                    match crate::api::tools::search_tools(&client, q.as_deref(), next_page, 20)
                        .await
                    {
                        Ok(p) => tx.send(AsyncResult::ToolsAppended(p)).await.ok(),
                        Err(e) => tx
                            .send(AsyncResult::LoadFailed(tab, e.to_string()))
                            .await
                            .ok(),
                    };
                }
                Tab::Integrations => {
                    match crate::api::integrations::search_integrations(
                        &client,
                        q.as_deref(),
                        next_page,
                        20,
                    )
                    .await
                    {
                        Ok(p) => tx.send(AsyncResult::IntegrationsAppended(p)).await.ok(),
                        Err(e) => tx
                            .send(AsyncResult::LoadFailed(tab, e.to_string()))
                            .await
                            .ok(),
                    };
                }
            }
        });
    }

    fn open_run_panel(&mut self) {
        let (name, id, kind) = match self.active_tab {
            Tab::Models => {
                let Some(m) = self.models.selected_item() else {
                    return;
                };
                (
                    m.name.clone().unwrap_or_default(),
                    m.id.clone().unwrap_or_default(),
                    RunKind::Model,
                )
            }
            Tab::Agents => {
                let Some(a) = self.agents.selected_item() else {
                    return;
                };
                (
                    a.name.clone().unwrap_or_default(),
                    a.id.clone().unwrap_or_default(),
                    RunKind::Agent,
                )
            }
            _ => {
                self.set_status("Run is available for Models and Agents", StatusKind::Info);
                return;
            }
        };

        self.run_panel = RunPanel {
            visible: true,
            resource_name: name,
            resource_id: id,
            resource_kind: kind,
            input: String::new(),
            running: false,
            output: None,
            error: None,
            credits: None,
            run_time: None,
        };
    }

    fn execute_run(&mut self) {
        self.run_panel.running = true;
        self.run_panel.output = None;
        self.run_panel.error = None;

        let client = self.client.clone();
        let tx = self.tx.clone();
        let id = self.run_panel.resource_id.clone();
        let input = self.run_panel.input.clone();
        let kind = self.run_panel.resource_kind;

        tokio::spawn(async move {
            match kind {
                RunKind::Model => {
                    let model_input = crate::api::models::ModelInput {
                        text: Some(input),
                        file_url: None,
                        params: std::collections::HashMap::new(),
                    };
                    match crate::api::models::run_model(&client, &id, &model_input, 300, None).await
                    {
                        Ok(result) => {
                            let output = result
                                .data
                                .as_ref()
                                .and_then(|d| d.as_str())
                                .map(String::from)
                                .or_else(|| {
                                    result.data.as_ref().map(|d| {
                                        serde_json::to_string_pretty(d).unwrap_or_default()
                                    })
                                })
                                .unwrap_or_else(|| "(no output)".into());
                            tx.send(AsyncResult::RunCompleted {
                                output,
                                credits: result.used_credits,
                                run_time: result.run_time,
                            })
                            .await
                            .ok();
                        }
                        Err(e) => {
                            tx.send(AsyncResult::RunFailed(e.to_string())).await.ok();
                        }
                    }
                }
                RunKind::Agent => {
                    match crate::api::agents::run_agent(
                        &client, &id, &input, None, "text", 300, None,
                    )
                    .await
                    {
                        Ok(result) => {
                            tx.send(AsyncResult::RunCompleted {
                                output: result.output_text(),
                                credits: result.used_credits,
                                run_time: result.run_time,
                            })
                            .await
                            .ok();
                        }
                        Err(e) => {
                            tx.send(AsyncResult::RunFailed(e.to_string())).await.ok();
                        }
                    }
                }
            }
        });
    }

    fn set_status(&mut self, text: &str, kind: StatusKind) {
        self.status = StatusMessage {
            text: text.to_string(),
            kind,
            expires_at: Some(Instant::now() + std::time::Duration::from_secs(3)),
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

// ── Tests ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_list() -> ListState<String> {
        let mut ls = ListState::default();
        ls.items = vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()];
        ls.total = 5;
        ls.loading = false;
        ls
    }

    #[test]
    fn tab_navigation_wraps() {
        assert_eq!(Tab::Integrations.next(), Tab::Models);
        assert_eq!(Tab::Models.prev(), Tab::Integrations);
    }

    #[test]
    fn tab_cycle_is_consistent() {
        let mut t = Tab::Models;
        for _ in 0..4 {
            t = t.next();
        }
        assert_eq!(t, Tab::Models);
    }

    #[test]
    fn list_select_next_clamps() {
        let mut ls = test_list();
        ls.selected = 4;
        ls.select_next();
        assert_eq!(ls.selected, 4);
    }

    #[test]
    fn list_select_prev_clamps() {
        let mut ls = test_list();
        ls.selected = 0;
        ls.select_prev();
        assert_eq!(ls.selected, 0);
    }

    #[test]
    fn list_select_last() {
        let mut ls = test_list();
        ls.select_last();
        assert_eq!(ls.selected, 4);
    }

    #[test]
    fn list_apply_page_resets() {
        let mut ls: ListState<i32> = ListState::default();
        ls.selected = 5;
        ls.loading = true;
        ls.apply_page(Page {
            results: vec![1, 2, 3],
            total: 100,
            page_total: 5,
        });
        assert_eq!(ls.selected, 0);
        assert!(!ls.loading);
        assert_eq!(ls.items.len(), 3);
        assert_eq!(ls.total, 100);
    }

    #[test]
    fn list_append_page() {
        let mut ls: ListState<i32> = ListState::default();
        ls.items = vec![1, 2, 3];
        ls.loading = true;
        ls.append_page(Page {
            results: vec![4, 5],
            total: 5,
            page_total: 1,
        });
        assert_eq!(ls.items, vec![1, 2, 3, 4, 5]);
        assert!(!ls.loading);
    }

    #[test]
    fn list_needs_more() {
        let mut ls: ListState<i32> = ListState::default();
        ls.items = vec![1, 2, 3];
        ls.total = 10;
        ls.loading = false;
        ls.selected = 0;
        assert!(ls.needs_more());

        ls.loading = true;
        assert!(!ls.needs_more());
    }

    #[test]
    fn search_state_input() {
        let mut s = SearchState::default();
        s.active = true;
        s.query.push('h');
        s.query.push('i');
        assert_eq!(s.query, "hi");
        s.query.pop();
        assert_eq!(s.query, "h");
    }

    #[test]
    fn status_expires() {
        let mut st = StatusMessage {
            text: "test".into(),
            kind: StatusKind::Info,
            expires_at: Some(Instant::now() - std::time::Duration::from_secs(1)),
        };
        assert!(st.expires_at.unwrap() < Instant::now());
        st = StatusMessage::default();
        assert!(st.text.is_empty());
    }
}
