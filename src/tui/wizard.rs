use serde_json::json;

// ── Wizard state ────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct AgentWizard {
    pub visible: bool,
    pub step: WizardStep,
    pub edit_id: Option<String>,

    pub name: String,
    pub instructions: String,
    pub llm_custom: bool,
    pub llm_id: String,

    pub selected_tools: Vec<IdName>,
    pub picker: PickerState,

    pub selected_subagents: Vec<IdName>,

    pub max_iterations: i32,
    pub max_tokens: i32,
    pub output_format: OutputFmt,
    pub as_draft: bool,
    pub confirm_field: usize,

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

impl WizardStep {
    pub fn index(self) -> usize {
        match self {
            Self::Name => 0,
            Self::Instructions => 1,
            Self::Llm => 2,
            Self::Tools => 3,
            Self::SubAgents => 4,
            Self::Confirm => 5,
        }
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Self::Name => Some(Self::Instructions),
            Self::Instructions => Some(Self::Llm),
            Self::Llm => Some(Self::Tools),
            Self::Tools => Some(Self::SubAgents),
            Self::SubAgents => Some(Self::Confirm),
            Self::Confirm => None,
        }
    }

    pub fn prev(self) -> Option<Self> {
        match self {
            Self::Name => None,
            Self::Instructions => Some(Self::Name),
            Self::Llm => Some(Self::Instructions),
            Self::Tools => Some(Self::Llm),
            Self::SubAgents => Some(Self::Tools),
            Self::Confirm => Some(Self::SubAgents),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Instructions => "Instructions",
            Self::Llm => "LLM",
            Self::Tools => "Tools",
            Self::SubAgents => "Sub-Agents",
            Self::Confirm => "Confirm",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFmt {
    #[default]
    Text,
    Markdown,
    Json,
}

impl OutputFmt {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Markdown => "markdown",
            Self::Json => "json",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Text => Self::Markdown,
            Self::Markdown => Self::Json,
            Self::Json => Self::Text,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdName {
    pub id: String,
    pub name: String,
}

// ── Picker (reusable for tools and agents) ──────────

#[derive(Debug, Clone, Default)]
pub struct PickerState {
    pub items: Vec<PickerItem>,
    pub selected: usize,
    pub search_active: bool,
    pub search_query: String,
    pub loading: bool,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct PickerItem {
    pub id: String,
    pub name: String,
    pub checked: bool,
}

impl PickerState {
    pub fn select_next(&mut self) {
        if !self.items.is_empty() && self.selected < self.items.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn toggle_selected(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected) {
            item.checked = !item.checked;
        }
    }

    pub fn checked_items(&self) -> Vec<IdName> {
        self.items
            .iter()
            .filter(|i| i.checked)
            .map(|i| IdName {
                id: i.id.clone(),
                name: i.name.clone(),
            })
            .collect()
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.selected = 0;
        self.search_active = false;
        self.search_query.clear();
        self.loading = true;
        self.total = 0;
    }
}

// ── Wizard lifecycle ────────────────────────────────

const DEFAULT_LLM: &str = "669a63646eb56306647e1091";

impl AgentWizard {
    pub fn open_create() -> Self {
        Self {
            visible: true,
            step: WizardStep::Name,
            max_iterations: 5,
            max_tokens: 2048,
            ..Default::default()
        }
    }

    pub fn open_edit(agent: &crate::models::agent::Agent) -> Self {
        let selected_tools: Vec<IdName> = agent
            .tools
            .as_ref()
            .map(|tools| {
                tools
                    .iter()
                    .map(|t| IdName {
                        id: t.asset_id.clone().or(t.id.clone()).unwrap_or_default(),
                        name: t.name.clone().unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let selected_subagents: Vec<IdName> = agent
            .subagents
            .as_ref()
            .map(|subs| {
                subs.iter()
                    .filter_map(|s| {
                        let id = s.get("id")?.as_str()?.to_string();
                        let name = s
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("agent")
                            .to_string();
                        Some(IdName { id, name })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Self {
            visible: true,
            step: WizardStep::Name,
            edit_id: agent.id.clone(),
            name: agent.name.clone().unwrap_or_default(),
            instructions: agent.instructions.clone().unwrap_or_default(),
            llm_custom: false,
            llm_id: String::new(),
            selected_tools,
            picker: PickerState::default(),
            selected_subagents,
            max_iterations: agent.max_iterations.unwrap_or(5),
            max_tokens: agent.max_tokens.unwrap_or(2048),
            output_format: OutputFmt::Text,
            as_draft: agent.status.as_ref().map(|s| s.to_string()) == Some("Draft".into()),
            confirm_field: 0,
            saving: false,
            save_error: None,
            saved_id: None,
        }
    }

    pub fn is_edit(&self) -> bool {
        self.edit_id.is_some()
    }

    pub fn can_advance(&self) -> bool {
        match self.step {
            WizardStep::Name => !self.name.trim().is_empty(),
            _ => true,
        }
    }

    pub fn build_payload(&self) -> serde_json::Value {
        let instructions = convert_template_vars(&self.instructions);

        let tools: Vec<serde_json::Value> = self
            .selected_tools
            .iter()
            .map(|t| json!({"assetId": &t.id}))
            .collect();

        let subagents: Vec<serde_json::Value> = self
            .selected_subagents
            .iter()
            .map(|a| json!({"id": &a.id, "inspectors": []}))
            .collect();

        let llm_id = if self.llm_custom && !self.llm_id.is_empty() {
            &self.llm_id
        } else {
            DEFAULT_LLM
        };

        let status = if self.as_draft { "draft" } else { "onboarded" };

        json!({
            "name": self.name.trim(),
            "description": &instructions,
            "instructions": &instructions,
            "model": {"id": llm_id},
            "tools": tools,
            "status": status,
            "maxIterations": self.max_iterations,
            "maxTokens": self.max_tokens,
            "outputFormat": self.output_format.as_str(),
            "agents": subagents,
            "tasks": [],
        })
    }
}

/// Convert {{variable}} → {variable} per SDK v2
pub fn convert_template_vars(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end) = text[i + 2..].find("}}") {
                let var = &text[i + 2..i + 2 + end];
                if var.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    result.push('{');
                    result.push_str(var);
                    result.push('}');
                    i += 2 + end + 2;
                    continue;
                }
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

// ── Tests ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_var_conversion() {
        assert_eq!(convert_template_vars("Hello {{name}}!"), "Hello {name}!");
        assert_eq!(convert_template_vars("No vars here"), "No vars here");
        assert_eq!(convert_template_vars("{{a}} and {{b}}"), "{a} and {b}");
        assert_eq!(convert_template_vars(""), "");
        assert_eq!(convert_template_vars("{{x}}"), "{x}");
    }

    #[test]
    fn wizard_step_progression() {
        let mut s = WizardStep::Name;
        let steps = [
            "Name",
            "Instructions",
            "LLM",
            "Tools",
            "Sub-Agents",
            "Confirm",
        ];
        for (i, expected) in steps.iter().enumerate() {
            assert_eq!(s.index(), i);
            assert_eq!(s.label(), *expected);
            s = s.next().unwrap_or(WizardStep::Confirm);
        }
    }

    #[test]
    fn wizard_step_back_from_name_is_none() {
        assert!(WizardStep::Name.prev().is_none());
    }

    #[test]
    fn wizard_step_next_from_confirm_is_none() {
        assert!(WizardStep::Confirm.next().is_none());
    }

    #[test]
    fn wizard_name_validation() {
        let mut w = AgentWizard::open_create();
        assert!(!w.can_advance());
        w.name = "  ".into();
        assert!(!w.can_advance());
        w.name = "My Agent".into();
        assert!(w.can_advance());
    }

    #[test]
    fn wizard_payload_structure() {
        let mut w = AgentWizard::open_create();
        w.name = "Test Agent".into();
        w.instructions = "Help with {{topic}}".into();
        w.selected_tools = vec![IdName {
            id: "t1".into(),
            name: "Search".into(),
        }];
        w.max_iterations = 10;
        w.max_tokens = 4096;

        let payload = w.build_payload();
        assert_eq!(payload["name"], "Test Agent");
        assert_eq!(payload["instructions"], "Help with {topic}");
        assert_eq!(payload["description"], "Help with {topic}");
        assert_eq!(payload["model"]["id"], DEFAULT_LLM);
        assert_eq!(payload["tools"][0]["assetId"], "t1");
        assert_eq!(payload["maxIterations"], 10);
        assert_eq!(payload["maxTokens"], 4096);
        assert_eq!(payload["status"], "onboarded");
        assert_eq!(payload["outputFormat"], "text");
        assert!(payload["tasks"].as_array().unwrap().is_empty());
        assert!(payload["agents"].as_array().unwrap().is_empty());
    }

    #[test]
    fn wizard_payload_draft() {
        let mut w = AgentWizard::open_create();
        w.name = "Draft".into();
        w.as_draft = true;
        let payload = w.build_payload();
        assert_eq!(payload["status"], "draft");
    }

    #[test]
    fn wizard_payload_custom_llm() {
        let mut w = AgentWizard::open_create();
        w.name = "X".into();
        w.llm_custom = true;
        w.llm_id = "custom123".into();
        let payload = w.build_payload();
        assert_eq!(payload["model"]["id"], "custom123");
    }

    #[test]
    fn wizard_payload_subagents() {
        let mut w = AgentWizard::open_create();
        w.name = "Orch".into();
        w.selected_subagents = vec![
            IdName {
                id: "a1".into(),
                name: "Sub1".into(),
            },
            IdName {
                id: "a2".into(),
                name: "Sub2".into(),
            },
        ];
        let payload = w.build_payload();
        let agents = payload["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0]["id"], "a1");
        assert!(agents[0]["inspectors"].as_array().unwrap().is_empty());
    }

    #[test]
    fn output_fmt_cycle() {
        let f = OutputFmt::Text;
        assert_eq!(f.cycle(), OutputFmt::Markdown);
        assert_eq!(f.cycle().cycle(), OutputFmt::Json);
        assert_eq!(f.cycle().cycle().cycle(), OutputFmt::Text);
    }

    #[test]
    fn picker_toggle() {
        let mut p = PickerState::default();
        p.items = vec![
            PickerItem {
                id: "1".into(),
                name: "A".into(),
                checked: false,
            },
            PickerItem {
                id: "2".into(),
                name: "B".into(),
                checked: false,
            },
        ];
        p.toggle_selected();
        assert!(p.items[0].checked);
        assert!(!p.items[1].checked);

        p.select_next();
        p.toggle_selected();
        assert!(p.items[1].checked);

        let checked = p.checked_items();
        assert_eq!(checked.len(), 2);
    }

    #[test]
    fn picker_nav_clamps() {
        let mut p = PickerState::default();
        p.items = vec![PickerItem {
            id: "1".into(),
            name: "A".into(),
            checked: false,
        }];
        p.select_next();
        assert_eq!(p.selected, 0);
        p.select_prev();
        assert_eq!(p.selected, 0);
    }
}
