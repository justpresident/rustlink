use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{CrosstermBackend, Terminal};
use rustlink::{
    app::{App, ShopTab, UIMode},
    commands::{CommandRegistry, CommandResult, execute_input, get_completions},
    model::HardwareKind,
    shop::Shop,
    ui::{ViewRegistry, render},
};
use std::time::Duration;

/// Rustlink: A terminal-based hacking simulator.
#[derive(Parser, Debug)]
#[command(author, version, about = "Rustlink: A terminal-based hacking simulator.", long_about = None)]
#[command(hide = true)] // Hide the help message from general users
struct Args {
    /// Enable rich text rendering
    #[arg(long, hide = true)]
    rich: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new(args.rich);
    let mut registry = CommandRegistry::new();
    let view_registry = ViewRegistry::new();
    let tick_rate = Duration::from_millis(50);

    // Initial activation for Home server
    let initial_server_type = app.connection.connected_server_type.clone();
    if let Some(st) = initial_server_type {
        registry.activate_commands(st.associated_commands());
    }

    loop {
        terminal.draw(|f| render(f, &mut app, &registry, &view_registry))?;

        if event::poll(tick_rate)?
            && let Event::Key(key) = event::read()?
            && key.kind == event::KeyEventKind::Press
        {
            // Handle shop mode (merged with assembly)
            if app.ui_mode == UIMode::Shop {
                match key.code {
                    KeyCode::Esc => {
                        if !app.try_close_shop() {
                            app.log("Cannot exit shop - PC must be fully assembled first!");
                        }
                    }
                    KeyCode::Tab => {
                        app.toggle_shop_tab();
                    }
                    KeyCode::Left => {
                        let categories = HardwareKind::all();
                        let current_idx = categories
                            .iter()
                            .position(|c| *c == app.shop_category)
                            .unwrap_or(0);
                        let new_idx = if current_idx == 0 {
                            categories.len() - 1
                        } else {
                            current_idx - 1
                        };
                        app.shop_category = categories[new_idx];
                        app.shop_selection = 0;
                    }
                    KeyCode::Right => {
                        let categories = HardwareKind::all();
                        let current_idx = categories
                            .iter()
                            .position(|c| *c == app.shop_category)
                            .unwrap_or(0);
                        let new_idx = (current_idx + 1) % categories.len();
                        app.shop_category = categories[new_idx];
                        app.shop_selection = 0;
                    }
                    KeyCode::Up => {
                        if app.shop_selection > 0 {
                            app.shop_selection -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let max_items = get_item_count(&app);
                        if app.shop_selection < max_items.saturating_sub(1) {
                            app.shop_selection += 1;
                        }
                    }
                    KeyCode::Enter => {
                        handle_shop_enter(&mut app);
                    }
                    KeyCode::Backspace if app.shop_tab == ShopTab::Owned => {
                        // Toggle install state of selected component using grouped display
                        let groups = app.player.inventory.grouped_display(app.shop_category);
                        if let Some(group) = groups.get(app.shop_selection) {
                            let abs_idx = group.first_index();
                            match app.player.toggle_install(abs_idx) {
                                Ok(msg) => app.log(msg),
                                Err(msg) => app.log(format!("Error: {msg}")),
                            }
                        }
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                // Quit
                KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    break;
                }
                // Ctrl+A - move to start of line
                KeyCode::Char('a') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    app.terminal.move_cursor_start();
                }
                // Ctrl+E - move to end of line
                KeyCode::Char('e') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    app.terminal.move_cursor_end();
                }
                // Ctrl+U - clear line
                KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    app.terminal.clear_line();
                }
                // Ctrl+W - delete word
                KeyCode::Char('w') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    app.terminal.delete_word();
                }
                // Regular character input
                KeyCode::Char(c) => app.terminal.insert_char(c),
                // Backspace - delete char before cursor
                KeyCode::Backspace => app.terminal.delete_char(),
                // Delete - delete char at cursor
                KeyCode::Delete => app.terminal.delete_char_forward(),
                // Arrow keys
                KeyCode::Left => app.terminal.move_cursor_left(),
                KeyCode::Right => app.terminal.move_cursor_right(),
                KeyCode::Up => app.terminal.scroll_logs_up(2),
                KeyCode::Down => app.terminal.scroll_logs_down(2),
                // Home/End
                KeyCode::Home => app.terminal.move_cursor_start(),
                KeyCode::End => app.terminal.move_cursor_end(),
                // Page Up/Down
                KeyCode::PageUp => app.terminal.history_up(),
                KeyCode::PageDown => app.terminal.history_down(),
                // Tab - autocomplete
                KeyCode::Tab => {
                    let completions = get_completions(&registry, &app, &app.terminal.input);
                    app.terminal.apply_completions(&completions);
                }
                // Enter - execute command
                KeyCode::Enter => {
                    app.terminal.save_to_history();
                    let command_result = execute_input(&mut registry, &mut app);
                    app.terminal.input.clear();
                    app.terminal.cursor_pos = 0;

                    match command_result {
                        CommandResult::Ok => {}
                        CommandResult::Quit => {
                            app.should_quit = true;
                        }
                        CommandResult::ConnectionChanged {
                            old_server_type,
                            new_server_type,
                        } => {
                            if let Some(old_type) = old_server_type {
                                let commands_to_deactivate: &[&str] =
                                    old_type.associated_commands();
                                registry.deactivate_commands(commands_to_deactivate);
                            }

                            if let Some(new_type) = new_server_type {
                                let commands_to_activate: &[&str] = new_type.associated_commands();
                                registry.activate_commands(commands_to_activate);
                                app.log(format!(
                                    "Commands available on the server: {:?}",
                                    commands_to_activate
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if app.last_tick.elapsed() >= tick_rate {
            app.on_tick(&registry.tool_registry);
            app.last_tick = std::time::Instant::now();
        }
        if app.should_quit {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    Ok(())
}

/// Get item count for current tab and category
fn get_item_count(app: &App) -> usize {
    match app.shop_tab {
        ShopTab::Available => Shop::count_items_for(app.shop_category),
        // Use grouped display count - this accounts for grouped spare items
        ShopTab::Owned => app
            .player
            .inventory
            .grouped_display(app.shop_category)
            .len(),
    }
}

/// Handle Enter key in shop mode
fn handle_shop_enter(app: &mut App) {
    match app.shop_tab {
        ShopTab::Available => {
            // Purchase item
            if let Some(item) = Shop::get_at(app.shop_category, app.shop_selection) {
                match Shop::purchase(&mut app.player, item) {
                    Ok(result) => {
                        app.log(format!(
                            "Purchased {} for {}c - added to inventory",
                            result.item_name, result.price
                        ));
                        if !result.warnings.is_empty() {
                            for w in &result.warnings {
                                app.log(format!("  ⚠ {}", w));
                            }
                        }
                    }
                    Err(msg) => app.log(format!("Error: {msg}")),
                }
            }
        }
        ShopTab::Owned => {
            // Use grouped display to get the correct inventory index
            let groups = app.player.inventory.grouped_display(app.shop_category);
            if let Some(group) = groups.get(app.shop_selection) {
                let abs_idx = group.first_index();
                match app.player.toggle_install(abs_idx) {
                    Ok(msg) => app.log(msg),
                    Err(msg) => app.log(format!("Error: {}", msg)),
                }
            }
        }
    }
}
