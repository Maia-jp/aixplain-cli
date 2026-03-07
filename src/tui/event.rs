use super::app::{App, Focus, Message};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub fn map_event(event: Event, app: &App) -> Option<Message> {
    match event {
        Event::Key(key) => map_key(key, app),
        _ => None,
    }
}

fn map_key(key: KeyEvent, app: &App) -> Option<Message> {
    if app.run_panel.visible {
        return map_run_panel_key(key, app);
    }

    if app.show_help {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => Some(Message::HideHelp),
            _ => Some(Message::HideHelp),
        };
    }

    if app.search.active {
        return map_search_key(key);
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Message::Quit),

        KeyCode::Tab | KeyCode::Char(']') => Some(Message::NextTab),
        KeyCode::BackTab | KeyCode::Char('[') => Some(Message::PrevTab),

        KeyCode::Down | KeyCode::Char('j') => Some(Message::NextItem),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::PrevItem),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::PageDown)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::PageUp)
        }
        KeyCode::Char('g') => Some(Message::SelectFirst),
        KeyCode::Char('G') => Some(Message::SelectLast),

        KeyCode::Left | KeyCode::Char('h') => {
            if app.focus == Focus::Detail {
                Some(Message::ToggleFocus)
            } else {
                None
            }
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => Some(Message::ToggleFocus),

        KeyCode::Char('/') => Some(Message::EnterSearch),
        KeyCode::Char('?') => Some(Message::ShowHelp),
        KeyCode::Char('c') => Some(Message::CopyId),
        KeyCode::Char('r') => Some(Message::OpenRun),

        KeyCode::Esc => {
            if app.focus == Focus::Detail {
                Some(Message::ToggleFocus)
            } else {
                None
            }
        }

        _ => None,
    }
}

fn map_search_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc => Some(Message::ExitSearch),
        KeyCode::Enter => Some(Message::SearchSubmit),
        KeyCode::Backspace => Some(Message::SearchBackspace),
        KeyCode::Char(c) => Some(Message::SearchInput(c)),
        _ => None,
    }
}

fn map_run_panel_key(key: KeyEvent, app: &App) -> Option<Message> {
    if app.run_panel.running {
        return match key.code {
            KeyCode::Esc => Some(Message::CloseRun),
            _ => None,
        };
    }

    if app.run_panel.output.is_some() || app.run_panel.error.is_some() {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(Message::CloseRun),
            KeyCode::Char('r') => Some(Message::RunExecute),
            _ => Some(Message::CloseRun),
        };
    }

    match key.code {
        KeyCode::Esc => Some(Message::CloseRun),
        KeyCode::Enter => Some(Message::RunExecute),
        KeyCode::Backspace => Some(Message::RunBackspace),
        KeyCode::Char(c) => Some(Message::RunInput(c)),
        _ => None,
    }
}
