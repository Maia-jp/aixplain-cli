use crate::models::agent::{Agent, AgentRunResult};
use crate::models::api_key::ApiKey;
use crate::models::common::Page;
use crate::models::integration::Integration;
use crate::models::model::{Model, ModelResult};
use crate::models::tool::Tool;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Table};
use std::sync::OnceLock;

static IS_TTY: OnceLock<bool> = OnceLock::new();

pub fn is_tty() -> bool {
    *IS_TTY.get_or_init(|| std::io::IsTerminal::is_terminal(&std::io::stdout()))
}

pub fn print_models_table(page: &Page<Model>) {
    if page.is_empty() {
        println!("  No models found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["ID", "Name", "Function", "Supplier", "Status"]);

    for m in &page.results {
        table.add_row(vec![
            truncate_id(m.id.as_deref().unwrap_or("-")),
            m.name.as_deref().unwrap_or("-").to_string(),
            m.function
                .as_ref()
                .and_then(|f| f.name.as_deref())
                .unwrap_or("-")
                .to_string(),
            m.vendor
                .as_ref()
                .and_then(|v| v.name.as_deref())
                .unwrap_or("-")
                .to_string(),
            m.status
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
        ]);
    }

    println!("{table}");
    println!(
        "\n  Showing {} of {} results",
        page.results.len(),
        page.total
    );
}

pub fn print_model_detail(m: &Model) {
    println!();
    println!("  {}", m.name.as_deref().unwrap_or("(unnamed)"));
    if let Some(ref vendor) = m.vendor {
        if let Some(ref name) = vendor.name {
            let func = m
                .function
                .as_ref()
                .and_then(|f| f.name.as_deref())
                .unwrap_or("unknown");
            println!("  {name} · {func}");
        }
    }
    println!("  ────────────────────────────────────");
    println!("  ID          {}", m.id.as_deref().unwrap_or("-"));
    println!(
        "  Status      {}",
        m.status
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".into())
    );
    if let Some(ref path) = m.path {
        println!("  Path        {path}");
    }
    if let Some(streaming) = m.supports_streaming {
        println!(
            "  Streaming   {}",
            if streaming { "Supported" } else { "No" }
        );
    }
    if let Some(ref params) = m.params {
        println!("  ────────────────────────────────────");
        println!("  Parameters:");
        for p in params {
            let req = if p.required { " (required)" } else { "" };
            let dtype = p.data_type.as_deref().unwrap_or("any");
            println!("    {:<16} {dtype}{req}", p.name);
        }
    }
    println!();
}

pub fn print_model_result(result: &ModelResult) {
    println!();
    if let Some(ref data) = result.data {
        if let Some(s) = data.as_str() {
            println!("  {s}");
        } else {
            println!(
                "  {}",
                serde_json::to_string_pretty(data).unwrap_or_default()
            );
        }
    } else {
        println!("  (no output)");
    }
    println!();

    let mut footer = Vec::new();
    if let Some(credits) = result.used_credits {
        footer.push(format!("Credits: {credits:.4}"));
    }
    if let Some(time) = result.run_time {
        footer.push(format!("Time: {time:.1}s"));
    }
    if let Some(ref usage) = result.usage {
        footer.push(format!(
            "Tokens: {}/{}",
            usage.prompt_tokens, usage.completion_tokens
        ));
    }
    if !footer.is_empty() {
        println!("  {}", footer.join("  │  "));
        println!();
    }
}

// ── Agents ──────────────────────────────────────────

pub fn print_agents_table(page: &Page<Agent>) {
    if page.is_empty() {
        println!("  No agents found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["ID", "Name", "Status", "Tools", "Updated"]);

    for a in &page.results {
        let tool_count = a.tools.as_ref().map(|t| t.len()).unwrap_or(0);
        table.add_row(vec![
            truncate_id(a.id.as_deref().unwrap_or("-")),
            a.name.as_deref().unwrap_or("-").to_string(),
            a.status
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            tool_count.to_string(),
            a.updated_at
                .as_deref()
                .map(|d| d.split('T').next().unwrap_or(d))
                .unwrap_or("-")
                .to_string(),
        ]);
    }

    println!("{table}");
    println!(
        "\n  Showing {} of {} results",
        page.results.len(),
        page.total
    );
}

pub fn print_agent_detail(a: &Agent) {
    println!();
    println!("  {}", a.name.as_deref().unwrap_or("(unnamed)"));
    println!("  ────────────────────────────────────");
    println!("  ID          {}", a.id.as_deref().unwrap_or("-"));
    println!(
        "  Status      {}",
        a.status
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".into())
    );
    if let Some(ref inst) = a.instructions {
        let preview = if inst.len() > 80 {
            format!("{}…", &inst[..80])
        } else {
            inst.clone()
        };
        println!("  Prompt      {preview}");
    }
    if let Some(ref tools) = a.tools {
        println!("  Tools       {} attached", tools.len());
        for t in tools {
            let name = t.name.as_deref().unwrap_or("unnamed");
            let id = t.asset_id.as_deref().or(t.id.as_deref()).unwrap_or("-");
            println!("              · {name} ({id})");
        }
    }
    if let Some(iters) = a.max_iterations {
        println!("  Max Iters   {iters}");
    }
    if let Some(tokens) = a.max_tokens {
        println!("  Max Tokens  {tokens}");
    }
    println!();
}

pub fn print_agent_result(result: &AgentRunResult) {
    println!();
    println!("  {}", result.output_text());
    println!();

    let mut footer = Vec::new();
    if let Some(ref sid) = result.session_id {
        footer.push(format!("Session: {}", truncate_id(sid)));
    }
    if let Some(credits) = result.used_credits {
        footer.push(format!("Credits: {credits:.4}"));
    }
    if let Some(time) = result.run_time {
        footer.push(format!("Time: {time:.1}s"));
    }
    if !footer.is_empty() {
        println!("  {}", footer.join("  │  "));
        println!();
    }
}

// ── Tools ───────────────────────────────────────────

pub fn print_tools_table(page: &Page<Tool>) {
    if page.is_empty() {
        println!("  No tools found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["ID", "Name", "Function", "Status"]);

    for t in &page.results {
        table.add_row(vec![
            truncate_id(t.model.id.as_deref().unwrap_or("-")),
            t.model.name.as_deref().unwrap_or("-").to_string(),
            t.model
                .function
                .as_ref()
                .and_then(|f| f.name.as_deref())
                .unwrap_or("-")
                .to_string(),
            t.model
                .status
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
        ]);
    }

    println!("{table}");
    println!(
        "\n  Showing {} of {} results",
        page.results.len(),
        page.total
    );
}

pub fn print_tool_detail(t: &Tool) {
    println!();
    println!("  {}", t.model.name.as_deref().unwrap_or("(unnamed)"));
    println!("  ────────────────────────────────────");
    println!("  ID          {}", t.model.id.as_deref().unwrap_or("-"));
    if let Some(ref aid) = t.asset_id {
        println!("  Asset ID    {aid}");
    }
    println!(
        "  Status      {}",
        t.model
            .status
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".into())
    );
    if let Some(ref actions) = t.allowed_actions {
        println!("  Actions     {}", actions.join(", "));
    }
    println!();
}

// ── Integrations ────────────────────────────────────

pub fn print_integrations_table(page: &Page<Integration>) {
    if page.is_empty() {
        println!("  No integrations found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["ID", "Name", "Status", "Actions"]);

    for i in &page.results {
        table.add_row(vec![
            truncate_id(i.model.id.as_deref().unwrap_or("-")),
            i.model.name.as_deref().unwrap_or("-").to_string(),
            i.model
                .status
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".into()),
            if i.actions_available == Some(true) {
                "Yes"
            } else {
                "-"
            }
            .to_string(),
        ]);
    }

    println!("{table}");
    println!(
        "\n  Showing {} of {} results",
        page.results.len(),
        page.total
    );
}

pub fn print_integration_detail(i: &Integration) {
    println!();
    println!("  {}", i.model.name.as_deref().unwrap_or("(unnamed)"));
    println!("  ────────────────────────────────────");
    println!("  ID          {}", i.model.id.as_deref().unwrap_or("-"));
    println!(
        "  Status      {}",
        i.model
            .status
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".into())
    );
    println!(
        "  Actions     {}",
        if i.actions_available == Some(true) {
            "Available"
        } else {
            "None"
        }
    );
    println!();
}

// ── API Keys ────────────────────────────────────────

pub fn print_api_keys_table(keys: &[ApiKey]) {
    if keys.is_empty() {
        println!("  No API keys found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["ID", "Name", "Budget", "Expires", "Admin"]);

    for k in keys {
        table.add_row(vec![
            truncate_id(k.id.as_deref().unwrap_or("-")),
            k.name.as_deref().unwrap_or("-").to_string(),
            k.budget
                .map(|b| format!("${b:.2}"))
                .unwrap_or_else(|| "-".into()),
            k.expires_at.as_deref().unwrap_or("Never").to_string(),
            if k.is_admin { "Yes" } else { "No" }.to_string(),
        ]);
    }

    println!("{table}");
}

// ── Helpers ─────────────────────────────────────────

fn truncate_id(id: &str) -> String {
    if id.len() > 20 {
        format!("{}…", &id[..19])
    } else {
        id.to_string()
    }
}

pub fn create_spinner(message: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
