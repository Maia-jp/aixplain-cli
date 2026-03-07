use super::app::{App, Focus, StatusKind, Tab};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Tabs, Wrap,
};
use ratatui::Frame;

const CYAN: Color = Color::Cyan;
const YELLOW: Color = Color::Yellow;
const GRAY: Color = Color::DarkGray;
const GREEN: Color = Color::Green;
const RED: Color = Color::Red;
const WHITE: Color = Color::White;
const HIGHLIGHT_BG: Color = Color::Rgb(40, 44, 62);

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(2),
        ])
        .split(f.area());

    render_tabs(f, app, chunks[0]);

    if app.search.active {
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(chunks[1]);
        render_search_bar(f, app, body_chunks[0]);
        render_body(f, app, body_chunks[1]);
    } else {
        render_body(f, app, chunks[1]);
    }

    render_status_bar(f, app, chunks[2]);

    if app.show_help {
        render_help_overlay(f);
    }

    if app.run_panel.visible {
        render_run_panel(f, app);
    }
}

// ── Tabs ────────────────────────────────────────────

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| {
            let style = if *t == app.active_tab {
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(GRAY)
            };
            Line::from(Span::styled(t.title(), style))
        })
        .collect();

    let idx = Tab::ALL
        .iter()
        .position(|t| *t == app.active_tab)
        .unwrap_or(0);

    let title_suffix = if !app.search.query.is_empty() && !app.search.active {
        format!("  search: \"{}\" ", app.search.query)
    } else {
        String::new()
    };

    let tabs = Tabs::new(titles)
        .select(idx)
        .highlight_style(Style::default().fg(YELLOW).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(GRAY))
                .title(Span::styled(
                    format!(" aiXplain{title_suffix}"),
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                )),
        );

    f.render_widget(tabs, area);
}

// ── Search bar ──────────────────────────────────────

fn render_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let cursor_char = "▏";
    let text = format!(" /{}{cursor_char}", app.search.query);

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(
            " / ",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}{cursor_char}", app.search.query),
            Style::default().fg(WHITE),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(YELLOW))
            .title(Span::styled(
                " Search (Enter to submit, Esc to cancel) ",
                Style::default().fg(YELLOW),
            )),
    );

    let _ = text;
    f.render_widget(bar, area);
}

// ── Body ────────────────────────────────────────────

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    if app.focus == Focus::Detail {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);
        render_list_panel(f, app, chunks[0], false);
        render_detail_panel(f, app, chunks[1]);
    } else {
        render_list_panel(f, app, area, true);
    }
}

// ── Scrollable list rendering ───────────────────────

fn render_list_panel(f: &mut Frame, app: &App, area: Rect, full_width: bool) {
    match app.active_tab {
        Tab::Models => render_scrollable_list(
            f,
            area,
            full_width,
            &app.models,
            &format!(" Models ({}/{}) ", app.models.items.len(), app.models.total),
            |m, wide| {
                let name = m.name.as_deref().unwrap_or("(unnamed)");
                let func = m
                    .function
                    .as_ref()
                    .and_then(|f| f.name.as_deref())
                    .unwrap_or("");
                let vendor = m
                    .vendor
                    .as_ref()
                    .and_then(|v| v.name.as_deref())
                    .unwrap_or("");
                if wide {
                    format!(" {name:<30} {func:<22} {vendor}")
                } else {
                    format!(" {name}")
                }
            },
        ),
        Tab::Agents => render_scrollable_list(
            f,
            area,
            full_width,
            &app.agents,
            &format!(" Agents ({}/{}) ", app.agents.items.len(), app.agents.total),
            |a, wide| {
                let name = a.name.as_deref().unwrap_or("(unnamed)");
                let tools = a.tools.as_ref().map(|t| t.len()).unwrap_or(0);
                let status = a
                    .status
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "-".into());
                if wide {
                    format!(" {name:<30} {status:<10} {tools} tools")
                } else {
                    format!(" {name}")
                }
            },
        ),
        Tab::Tools => render_scrollable_list(
            f,
            area,
            full_width,
            &app.tools,
            &format!(" Tools ({}/{}) ", app.tools.items.len(), app.tools.total),
            |t, wide| {
                let name = t.model.name.as_deref().unwrap_or("(unnamed)");
                let func = t
                    .model
                    .function
                    .as_ref()
                    .and_then(|f| f.name.as_deref())
                    .unwrap_or("");
                if wide {
                    format!(" {name:<30} {func}")
                } else {
                    format!(" {name}")
                }
            },
        ),
        Tab::Integrations => render_scrollable_list(
            f,
            area,
            full_width,
            &app.integrations,
            &format!(
                " Integrations ({}/{}) ",
                app.integrations.items.len(),
                app.integrations.total
            ),
            |ig, wide| {
                let name = ig.model.name.as_deref().unwrap_or("(unnamed)");
                let status = ig
                    .model
                    .status
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "-".into());
                if wide {
                    format!(" {name:<30} {status}")
                } else {
                    format!(" {name}")
                }
            },
        ),
    }
}

fn render_scrollable_list<T: Clone>(
    f: &mut Frame,
    area: Rect,
    full_width: bool,
    ls: &super::app::ListState<T>,
    title: &str,
    format_item: impl Fn(&T, bool) -> String,
) {
    if ls.loading && ls.items.is_empty() {
        render_loading(f, area, "Loading...");
        return;
    }
    if let Some(ref err) = ls.error {
        render_error(f, area, err);
        return;
    }
    if ls.items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(GRAY))
            .title(Span::styled(title.to_string(), Style::default().fg(CYAN)));
        let para = Paragraph::new(Line::from(Span::styled(
            "  No results.",
            Style::default().fg(GRAY),
        )))
        .block(block);
        f.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = ls
        .items
        .iter()
        .map(|item| {
            let line = format_item(item, full_width);
            ListItem::new(Line::from(line))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(ls.selected));

    let loading_suffix = if ls.loading { " ◌" } else { "" };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(GRAY))
                .title(Span::styled(
                    format!("{title}{loading_suffix}"),
                    Style::default().fg(CYAN),
                )),
        )
        .highlight_style(
            Style::default()
                .bg(HIGHLIGHT_BG)
                .fg(WHITE)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    f.render_stateful_widget(list, area, &mut list_state);
}

// ── Detail panel ────────────────────────────────────

fn render_detail_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN))
        .title(Span::styled(
            " Detail ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::new(2, 2, 1, 1));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = match app.active_tab {
        Tab::Models => model_detail_lines(app),
        Tab::Agents => agent_detail_lines(app),
        Tab::Tools => tool_detail_lines(app),
        Tab::Integrations => integration_detail_lines(app),
    };

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, inner);
}

fn model_detail_lines(app: &App) -> Vec<Line<'static>> {
    let Some(m) = app.models.selected_item() else {
        return vec![Line::from("No model selected")];
    };
    let mut lines = vec![
        Line::from(Span::styled(
            m.name.clone().unwrap_or_default(),
            Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    detail_kv(&mut lines, "ID", m.id.as_deref());
    detail_kv(
        &mut lines,
        "Status",
        m.status.as_ref().map(|s| s.to_string()).as_deref(),
    );
    detail_kv(
        &mut lines,
        "Function",
        m.function.as_ref().and_then(|f| f.name.as_deref()),
    );
    detail_kv(
        &mut lines,
        "Vendor",
        m.vendor.as_ref().and_then(|v| v.name.as_deref()),
    );
    detail_kv(&mut lines, "Path", m.path.as_deref());
    detail_kv(
        &mut lines,
        "Streaming",
        m.supports_streaming.map(|b| if b { "Yes" } else { "No" }),
    );
    if let Some(ref params) = m.params {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Parameters:",
            Style::default().fg(YELLOW),
        )));
        for p in params {
            let req = if p.required { " *" } else { "" };
            let dt = p.data_type.as_deref().unwrap_or("any");
            lines.push(Line::from(format!("  {} ({dt}){req}", p.name)));
        }
    }
    lines
}

fn agent_detail_lines(app: &App) -> Vec<Line<'static>> {
    let Some(a) = app.agents.selected_item() else {
        return vec![Line::from("No agent selected")];
    };
    let mut lines = vec![
        Line::from(Span::styled(
            a.name.clone().unwrap_or_default(),
            Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    detail_kv(&mut lines, "ID", a.id.as_deref());
    detail_kv(
        &mut lines,
        "Status",
        a.status.as_ref().map(|s| s.to_string()).as_deref(),
    );
    if let Some(ref inst) = a.instructions {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Instructions:",
            Style::default().fg(YELLOW),
        )));
        for l in inst.lines() {
            lines.push(Line::from(format!("  {l}")));
        }
    }
    if let Some(ref tools) = a.tools {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Tools ({}):", tools.len()),
            Style::default().fg(YELLOW),
        )));
        for t in tools {
            let name = t.name.as_deref().unwrap_or("unnamed");
            lines.push(Line::from(format!("  · {name}")));
        }
    }
    detail_kv(
        &mut lines,
        "Max Iters",
        a.max_iterations.map(|i| i.to_string()).as_deref(),
    );
    detail_kv(
        &mut lines,
        "Max Tokens",
        a.max_tokens.map(|i| i.to_string()).as_deref(),
    );
    lines
}

fn tool_detail_lines(app: &App) -> Vec<Line<'static>> {
    let Some(t) = app.tools.selected_item() else {
        return vec![Line::from("No tool selected")];
    };
    let mut lines = vec![
        Line::from(Span::styled(
            t.model.name.clone().unwrap_or_default(),
            Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    detail_kv(&mut lines, "ID", t.model.id.as_deref());
    detail_kv(&mut lines, "Asset ID", t.asset_id.as_deref());
    detail_kv(
        &mut lines,
        "Status",
        t.model.status.as_ref().map(|s| s.to_string()).as_deref(),
    );
    detail_kv(
        &mut lines,
        "Function",
        t.model.function.as_ref().and_then(|f| f.name.as_deref()),
    );
    if let Some(ref actions) = t.allowed_actions {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Actions:",
            Style::default().fg(YELLOW),
        )));
        for a in actions {
            lines.push(Line::from(format!("  · {a}")));
        }
    }
    lines
}

fn integration_detail_lines(app: &App) -> Vec<Line<'static>> {
    let Some(i) = app.integrations.selected_item() else {
        return vec![Line::from("No integration selected")];
    };
    let mut lines = vec![
        Line::from(Span::styled(
            i.model.name.clone().unwrap_or_default(),
            Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    detail_kv(&mut lines, "ID", i.model.id.as_deref());
    detail_kv(
        &mut lines,
        "Status",
        i.model.status.as_ref().map(|s| s.to_string()).as_deref(),
    );
    detail_kv(
        &mut lines,
        "Actions",
        Some(if i.actions_available == Some(true) {
            "Available"
        } else {
            "None"
        }),
    );
    if let Some(ref desc) = i.model.description {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Description:",
            Style::default().fg(YELLOW),
        )));
        for l in desc.lines() {
            lines.push(Line::from(format!("  {l}")));
        }
    }
    lines
}

fn detail_kv(lines: &mut Vec<Line<'static>>, key: &str, value: Option<&str>) {
    if let Some(v) = value {
        lines.push(Line::from(vec![
            Span::styled(format!("{key:<12} "), Style::default().fg(GRAY)),
            Span::raw(v.to_string()),
        ]));
    }
}

// ── Status bar ──────────────────────────────────────

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = if !app.status.text.is_empty() {
        let color = match app.status.kind {
            StatusKind::Success => GREEN,
            StatusKind::Error => RED,
            StatusKind::Info => CYAN,
        };
        Span::styled(format!(" {}", app.status.text), Style::default().fg(color))
    } else if app.search.active {
        Span::styled(
            " Type to search · Enter submit · Esc cancel".to_string(),
            Style::default().fg(YELLOW),
        )
    } else {
        Span::styled(
            " j/k navigate  l detail  r run  ]/[ tabs  / search  c copy  ? help  q quit".to_string(),
            Style::default().fg(GRAY),
        )
    };

    let bar = Paragraph::new(Line::from(status_text)).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(GRAY)),
    );
    f.render_widget(bar, area);
}

// ── Help overlay ────────────────────────────────────

fn render_help_overlay(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        help_line("q / Ctrl+C", "Quit"),
        help_line("] / Tab", "Next tab"),
        help_line("[ / Shift+Tab", "Previous tab"),
        help_line("j/k  ↑/↓", "Navigate list"),
        help_line("l / Enter / →", "Open detail panel"),
        help_line("h / Esc / ←", "Close detail panel"),
        help_line("g / G", "First / Last item"),
        help_line("Ctrl+D / Ctrl+U", "Page down / up"),
        help_line("/", "Search"),
        help_line("r", "Run selected model/agent"),
        help_line("c", "Copy selected ID"),
        help_line("?", "Toggle this help"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(GRAY),
        )),
    ];

    let para = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(CYAN))
            .title(Span::styled(
                " Help ",
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ))
            .padding(Padding::new(2, 2, 1, 1)),
    );

    f.render_widget(para, area);
}

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{key:<18}"), Style::default().fg(YELLOW)),
        Span::raw(desc),
    ])
}

// ── Utility ─────────────────────────────────────────

fn render_loading(f: &mut Frame, area: Rect, msg: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GRAY));
    let para = Paragraph::new(Line::from(Span::styled(
        format!("  ◌ {msg}"),
        Style::default().fg(YELLOW),
    )))
    .block(block);
    f.render_widget(para, area);
}

fn render_error(f: &mut Frame, area: Rect, msg: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(RED));
    let para = Paragraph::new(Line::from(Span::styled(
        format!("  Error: {msg}"),
        Style::default().fg(RED),
    )))
    .block(block);
    f.render_widget(para, area);
}

fn render_run_panel(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 75, f.area());
    f.render_widget(Clear, area);

    let rp = &app.run_panel;
    let kind_label = match rp.resource_kind {
        super::app::RunKind::Model => "Model",
        super::app::RunKind::Agent => "Agent",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN))
        .title(Span::styled(
            format!(" Run {kind_label}: {} ", rp.resource_name),
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::new(2, 2, 1, 1));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if rp.output.is_none() && rp.error.is_none() && !rp.running {
        lines.push(Line::from(Span::styled(
            "Input:",
            Style::default().fg(YELLOW),
        )));
        lines.push(Line::from(""));

        let cursor = "▏";
        let input_display = if rp.input.is_empty() {
            Span::styled(
                format!("  Type your query here...{cursor}"),
                Style::default().fg(GRAY),
            )
        } else {
            Span::styled(
                format!("  {}{cursor}", rp.input),
                Style::default().fg(WHITE),
            )
        };
        lines.push(Line::from(input_display));
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Enter to execute · Esc to cancel",
            Style::default().fg(GRAY),
        )));
    } else if rp.running {
        lines.push(Line::from(Span::styled(
            format!("Input: {}", rp.input),
            Style::default().fg(GRAY),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ◌ Running...",
            Style::default().fg(YELLOW),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Esc to close",
            Style::default().fg(GRAY),
        )));
    } else if let Some(ref err) = rp.error {
        lines.push(Line::from(Span::styled(
            format!("Input: {}", rp.input),
            Style::default().fg(GRAY),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Error:",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        )));
        for l in err.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {l}"),
                Style::default().fg(RED),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Esc to close · r to retry",
            Style::default().fg(GRAY),
        )));
    } else if let Some(ref output) = rp.output {
        lines.push(Line::from(Span::styled(
            format!("Input: {}", rp.input),
            Style::default().fg(GRAY),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Output:",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        for l in output.lines() {
            lines.push(Line::from(format!("  {l}")));
        }
        lines.push(Line::from(""));

        let mut footer_parts = Vec::new();
        if let Some(credits) = rp.credits {
            footer_parts.push(format!("Credits: {credits:.4}"));
        }
        if let Some(time) = rp.run_time {
            footer_parts.push(format!("Time: {time:.1}s"));
        }
        if !footer_parts.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("  {}", footer_parts.join("  │  ")),
                Style::default().fg(CYAN),
            )));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(Span::styled(
            "Esc to close · r to run again",
            Style::default().fg(GRAY),
        )));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
