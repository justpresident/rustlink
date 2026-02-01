mod normal;
mod shop;

pub use normal::ViewRegistry;

use crossterm::event::KeyEvent;
use ratatui::prelude::Frame;

use crate::app::{App, UIMode};
use crate::commands::CommandRegistry;
use crate::model::ServerType;

/// Result of handling a key event
pub enum KeyResult {
    /// Continue normal operation
    Continue,
    /// User requested to quit
    Quit,
    /// Connection changed, requires registry updates
    ConnectionChanged {
        old_server_type: Option<ServerType>,
        new_server_type: Option<ServerType>,
    },
}

pub fn render(
    f: &mut Frame,
    app: &mut App,
    registry: &CommandRegistry,
    view_registry: &ViewRegistry,
) {
    match app.ui_mode {
        UIMode::Shop => {
            shop::render(f, app);
        }
        UIMode::Normal => {
            normal::render(f, app, registry, view_registry);
        }
    }
}

pub fn handle_key(key: KeyEvent, app: &mut App, registry: &mut CommandRegistry) -> KeyResult {
    match app.ui_mode {
        UIMode::Shop => shop::handle_key(key, app),
        UIMode::Normal => normal::handle_key(key, app, registry),
    }
}
