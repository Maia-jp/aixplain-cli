pub mod app;
mod event;
mod ui;
pub mod wizard;

pub use app::{App, AsyncResult, Tab};

use crate::client::AixClient;
use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

pub async fn run(client: Arc<AixClient>) -> Result<()> {
    let mut terminal = setup_terminal()?;
    install_panic_hook();

    let (tx, mut rx) = mpsc::channel::<AsyncResult>(64);
    let mut app = App::new(client.clone(), tx.clone());

    spawn_initial_load(&app);

    let tick = Duration::from_millis(200);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        while let Ok(result) = rx.try_recv() {
            app.handle_async(result);
        }

        if crossterm::event::poll(tick)? {
            let ev = crossterm::event::read()?;
            if let Some(msg) = event::map_event(ev, &app) {
                if !app.update(msg) {
                    break;
                }
            }
        } else {
            app.update(app::Message::Tick);
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
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

fn spawn_initial_load(app: &App) {
    {
        let client = app.client.clone();
        let tx = app.tx.clone();
        tokio::spawn(async move {
            match crate::api::models::search_models(
                &client,
                &crate::api::models::ModelSearchParams::default(),
            )
            .await
            {
                Ok(p) => tx.send(AsyncResult::ModelsLoaded(p)).await.ok(),
                Err(e) => tx
                    .send(AsyncResult::LoadFailed(Tab::Models, e.to_string()))
                    .await
                    .ok(),
            };
        });
    }
    {
        let client = app.client.clone();
        let tx = app.tx.clone();
        tokio::spawn(async move {
            match crate::api::agents::search_agents(&client, None, 0, 20).await {
                Ok(p) => tx.send(AsyncResult::AgentsLoaded(p)).await.ok(),
                Err(e) => tx
                    .send(AsyncResult::LoadFailed(Tab::Agents, e.to_string()))
                    .await
                    .ok(),
            };
        });
    }
    {
        let client = app.client.clone();
        let tx = app.tx.clone();
        tokio::spawn(async move {
            match crate::api::tools::search_tools(&client, None, 0, 20).await {
                Ok(p) => tx.send(AsyncResult::ToolsLoaded(p)).await.ok(),
                Err(e) => tx
                    .send(AsyncResult::LoadFailed(Tab::Tools, e.to_string()))
                    .await
                    .ok(),
            };
        });
    }
    {
        let client = app.client.clone();
        let tx = app.tx.clone();
        tokio::spawn(async move {
            match crate::api::integrations::search_integrations(&client, None, 0, 20).await {
                Ok(p) => tx.send(AsyncResult::IntegrationsLoaded(p)).await.ok(),
                Err(e) => tx
                    .send(AsyncResult::LoadFailed(Tab::Integrations, e.to_string()))
                    .await
                    .ok(),
            };
        });
    }
}
