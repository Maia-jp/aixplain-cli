use aixplain_cli::tui::app::*;

fn make_test_app() -> (App, tokio::sync::mpsc::Receiver<AsyncResult>) {
    let config = aixplain_cli::config::ResolvedConfig {
        api_key: "test".into(),
        key_type: aixplain_cli::config::KeyType::Team,
        backend_url: url::Url::parse("https://example.com").unwrap(),
        models_run_url: url::Url::parse("https://example.com/run").unwrap(),
    };
    let client = std::sync::Arc::new(aixplain_cli::client::AixClient::new(&config).unwrap());
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    (App::new(client, tx), rx)
}

#[test]
fn tui_app_initial_state() {
    let (app, _rx) = make_test_app();
    assert_eq!(app.active_tab, Tab::Models);
    assert_eq!(app.focus, Focus::List);
    assert!(!app.show_help);
    assert!(!app.search.active);
    assert!(app.models.items.is_empty());
    assert!(app.models.loading);
}

#[test]
fn tui_tab_navigation() {
    let (mut app, _rx) = make_test_app();
    assert_eq!(app.active_tab, Tab::Models);
    app.update(Message::NextTab);
    assert_eq!(app.active_tab, Tab::Agents);
    app.update(Message::NextTab);
    assert_eq!(app.active_tab, Tab::Tools);
    app.update(Message::NextTab);
    assert_eq!(app.active_tab, Tab::Integrations);
    app.update(Message::NextTab);
    assert_eq!(app.active_tab, Tab::Models);
    app.update(Message::PrevTab);
    assert_eq!(app.active_tab, Tab::Integrations);
}

#[test]
fn tui_list_navigation() {
    let (mut app, _rx) = make_test_app();

    app.models.loading = false;
    app.models.items = vec![
        default_model("Model A"),
        default_model("Model B"),
        default_model("Model C"),
    ];
    app.models.total = 3;

    assert_eq!(app.models.selected, 0);
    app.update(Message::NextItem);
    assert_eq!(app.models.selected, 1);
    app.update(Message::NextItem);
    assert_eq!(app.models.selected, 2);
    app.update(Message::NextItem);
    assert_eq!(app.models.selected, 2); // clamped

    app.update(Message::PrevItem);
    assert_eq!(app.models.selected, 1);

    app.update(Message::SelectFirst);
    assert_eq!(app.models.selected, 0);

    app.update(Message::SelectLast);
    assert_eq!(app.models.selected, 2);
}

#[test]
fn tui_focus_toggle() {
    let (mut app, _rx) = make_test_app();
    assert_eq!(app.focus, Focus::List);
    app.update(Message::ToggleFocus);
    assert_eq!(app.focus, Focus::Detail);
    app.update(Message::ToggleFocus);
    assert_eq!(app.focus, Focus::List);
}

#[test]
fn tui_search_flow() {
    let (mut app, _rx) = make_test_app();
    app.update(Message::EnterSearch);
    assert!(app.search.active);
    app.update(Message::SearchInput('g'));
    app.update(Message::SearchInput('p'));
    app.update(Message::SearchInput('t'));
    assert_eq!(app.search.query, "gpt");
    app.update(Message::SearchBackspace);
    assert_eq!(app.search.query, "gp");
    app.update(Message::ExitSearch);
    assert!(!app.search.active);
}

#[test]
fn tui_help_toggle() {
    let (mut app, _rx) = make_test_app();
    assert!(!app.show_help);
    app.update(Message::ShowHelp);
    assert!(app.show_help);
    app.update(Message::HideHelp);
    assert!(!app.show_help);
}

#[test]
fn tui_quit_returns_false() {
    let (mut app, _rx) = make_test_app();
    assert!(!app.update(Message::Quit));
}

#[test]
fn tui_async_models_loaded() {
    let (mut app, _rx) = make_test_app();
    assert!(app.models.loading);

    app.handle_async(AsyncResult::ModelsLoaded(
        aixplain_cli::models::common::Page {
            results: vec![default_model("GPT-4o"), default_model("Claude")],
            total: 100,
            page_total: 5,
        },
    ));

    assert!(!app.models.loading);
    assert_eq!(app.models.items.len(), 2);
    assert_eq!(app.models.total, 100);
    assert_eq!(app.models.selected, 0);
}

#[test]
fn tui_async_load_failed() {
    let (mut app, _rx) = make_test_app();
    app.handle_async(AsyncResult::LoadFailed(
        Tab::Models,
        "connection refused".into(),
    ));
    assert!(!app.models.loading);
    assert!(app.models.error.is_some());
}

#[test]
fn tui_page_down_up() {
    let (mut app, _rx) = make_test_app();
    app.models.loading = false;
    app.models.items = (0..20).map(|i| default_model(&format!("M{i}"))).collect();
    app.models.total = 20;

    app.update(Message::PageDown);
    assert_eq!(app.models.selected, 10);
    app.update(Message::PageDown);
    assert_eq!(app.models.selected, 19);
    app.update(Message::PageUp);
    assert_eq!(app.models.selected, 9);
}

#[test]
fn tui_copy_sets_status() {
    let (mut app, _rx) = make_test_app();
    app.models.loading = false;
    app.models.items = vec![model_with_id("abc123")];
    app.models.total = 1;

    app.update(Message::CopyId);
    assert!(app.status.text.contains("abc123"));
    assert_eq!(app.status.kind, StatusKind::Success);
}

// ── Wizard tests ────────────────────────────────────

#[test]
fn wizard_open_create() {
    let (mut app, _rx) = make_test_app();
    app.active_tab = Tab::Agents;
    app.update(Message::OpenWizardCreate);
    assert!(app.wizard.visible);
    assert_eq!(app.wizard.step, aixplain_cli::tui::wizard::WizardStep::Name);
    assert!(app.wizard.edit_id.is_none());
}

#[tokio::test]
async fn wizard_step_through() {
    let (mut app, _rx) = make_test_app();
    app.active_tab = Tab::Agents;
    app.update(Message::OpenWizardCreate);

    app.update(Message::Wiz(WizMsg::Input('T')));
    app.update(Message::Wiz(WizMsg::Input('e')));
    app.update(Message::Wiz(WizMsg::Input('s')));
    app.update(Message::Wiz(WizMsg::Input('t')));
    assert_eq!(app.wizard.name, "Test");

    app.update(Message::Wiz(WizMsg::Next));
    assert_eq!(
        app.wizard.step,
        aixplain_cli::tui::wizard::WizardStep::Instructions
    );

    app.update(Message::Wiz(WizMsg::Next));
    assert_eq!(app.wizard.step, aixplain_cli::tui::wizard::WizardStep::Llm);

    app.update(Message::Wiz(WizMsg::Next));
    assert_eq!(
        app.wizard.step,
        aixplain_cli::tui::wizard::WizardStep::Tools
    );

    app.update(Message::Wiz(WizMsg::Next));
    assert_eq!(
        app.wizard.step,
        aixplain_cli::tui::wizard::WizardStep::SubAgents
    );

    app.update(Message::Wiz(WizMsg::Next));
    assert_eq!(
        app.wizard.step,
        aixplain_cli::tui::wizard::WizardStep::Confirm
    );
}

#[test]
fn wizard_back_from_name_closes() {
    let (mut app, _rx) = make_test_app();
    app.active_tab = Tab::Agents;
    app.update(Message::OpenWizardCreate);
    assert!(app.wizard.visible);

    app.update(Message::Wiz(WizMsg::Back));
    assert!(!app.wizard.visible);
}

#[test]
fn wizard_back_from_instructions_goes_to_name() {
    let (mut app, _rx) = make_test_app();
    app.active_tab = Tab::Agents;
    app.update(Message::OpenWizardCreate);
    app.wizard.name = "X".into();
    app.update(Message::Wiz(WizMsg::Next));
    assert_eq!(
        app.wizard.step,
        aixplain_cli::tui::wizard::WizardStep::Instructions
    );

    app.update(Message::Wiz(WizMsg::Back));
    assert_eq!(app.wizard.step, aixplain_cli::tui::wizard::WizardStep::Name);
    assert!(app.wizard.visible);
}

#[test]
fn wizard_empty_name_blocks_advance() {
    let (mut app, _rx) = make_test_app();
    app.active_tab = Tab::Agents;
    app.update(Message::OpenWizardCreate);

    app.update(Message::Wiz(WizMsg::Next));
    assert_eq!(app.wizard.step, aixplain_cli::tui::wizard::WizardStep::Name);
}

#[tokio::test]
async fn wizard_llm_toggle() {
    let (mut app, _rx) = make_test_app();
    app.active_tab = Tab::Agents;
    app.update(Message::OpenWizardCreate);
    app.wizard.name = "X".into();
    app.update(Message::Wiz(WizMsg::Next));
    app.update(Message::Wiz(WizMsg::Next));
    assert_eq!(app.wizard.step, aixplain_cli::tui::wizard::WizardStep::Llm);

    assert!(!app.wizard.llm_custom);
    app.update(Message::Wiz(WizMsg::ToggleLlm));
    assert!(app.wizard.llm_custom);
    app.update(Message::Wiz(WizMsg::Input('a')));
    assert_eq!(app.wizard.llm_id, "a");
}

#[tokio::test]
async fn wizard_confirm_field_cycle() {
    let (mut app, _rx) = make_test_app();
    app.active_tab = Tab::Agents;
    app.update(Message::OpenWizardCreate);
    app.wizard.name = "X".into();
    app.wizard.step = aixplain_cli::tui::wizard::WizardStep::Confirm;

    assert_eq!(app.wizard.confirm_field, 0);
    app.update(Message::Wiz(WizMsg::CycleConfirmField));
    assert_eq!(app.wizard.confirm_field, 1);
    app.update(Message::Wiz(WizMsg::CycleConfirmField));
    assert_eq!(app.wizard.confirm_field, 2);

    app.update(Message::Wiz(WizMsg::ToggleConfirmValue));
    assert_eq!(
        app.wizard.output_format,
        aixplain_cli::tui::wizard::OutputFmt::Markdown
    );
}

#[test]
fn wizard_edit_prepopulates() {
    let agent = aixplain_cli::models::agent::Agent {
        id: Some("a123".into()),
        name: Some("Existing Agent".into()),
        instructions: Some("Do things".into()),
        max_iterations: Some(10),
        max_tokens: Some(4096),
        ..Default::default()
    };

    let wiz = aixplain_cli::tui::wizard::AgentWizard::open_edit(&agent);
    assert!(wiz.visible);
    assert_eq!(wiz.edit_id.as_deref(), Some("a123"));
    assert_eq!(wiz.name, "Existing Agent");
    assert_eq!(wiz.instructions, "Do things");
    assert_eq!(wiz.max_iterations, 10);
    assert_eq!(wiz.max_tokens, 4096);
    assert!(wiz.is_edit());
}

// ── Helpers ─────────────────────────────────────────

fn default_model(name: &str) -> aixplain_cli::models::model::Model {
    aixplain_cli::models::model::Model {
        name: Some(name.to_string()),
        ..Default::default()
    }
}

fn model_with_id(id: &str) -> aixplain_cli::models::model::Model {
    aixplain_cli::models::model::Model {
        id: Some(id.to_string()),
        name: Some("Test".to_string()),
        ..Default::default()
    }
}
