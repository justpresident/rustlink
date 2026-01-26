use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{CrosstermBackend, Terminal};
use rustlink::{
    app::{App, ShopCategory, ShopTab, UIMode},
    commands::{CommandRegistry, CommandResult, execute_input, get_completions},
    model::ComponentSlot,
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
                        let categories = ShopCategory::all();
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
                        let categories = ShopCategory::all();
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
                        // Uninstall current component from PC
                        let slot = category_to_slot(app.shop_category);
                        if let Some(name) = app.player.uninstall_component(slot) {
                            app.log(format!("Uninstalled {} - moved to inventory", name));
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

/// Convert ShopCategory to ComponentSlot
fn category_to_slot(category: ShopCategory) -> ComponentSlot {
    match category {
        ShopCategory::Cpu => ComponentSlot::Cpu,
        ShopCategory::Cooler => ComponentSlot::Cooler,
        ShopCategory::Motherboard => ComponentSlot::Motherboard,
        ShopCategory::Ram => ComponentSlot::Ram,
        ShopCategory::Storage => ComponentSlot::Storage,
        ShopCategory::Network => ComponentSlot::Network,
    }
}

/// Get item count for current tab and category
fn get_item_count(app: &App) -> usize {
    match app.shop_tab {
        ShopTab::Available => Shop::items_for_category(app.shop_category).len(),
        ShopTab::Owned => {
            let inv = &app.player.inventory;
            match app.shop_category {
                ShopCategory::Cpu => inv.cpus.len(),
                ShopCategory::Cooler => inv.coolers.len(),
                ShopCategory::Motherboard => inv.motherboards.len(),
                ShopCategory::Ram => inv.rams.len(),
                ShopCategory::Storage => inv.storage.len(),
                ShopCategory::Network => inv.networks.len(),
            }
        }
    }
}

/// Handle Enter key in shop mode
fn handle_shop_enter(app: &mut App) {
    match app.shop_tab {
        ShopTab::Available => {
            // Purchase item
            let items = Shop::items_for_category(app.shop_category);
            if let Some(item) = items.get(app.shop_selection) {
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
            // Install component from inventory
            let slot = category_to_slot(app.shop_category);
            let id = get_inventory_id(&app.player.inventory, slot, app.shop_selection);
            if let Some(id) = id {
                match app.player.install_from_inventory(slot, &id) {
                    Ok(warnings) => {
                        app.log("Component installed");
                        for w in &warnings {
                            app.log(format!("  ⚠ {}", w));
                        }
                        // Reset selection if needed
                        let count = get_item_count(app);
                        if app.shop_selection >= count {
                            app.shop_selection = count.saturating_sub(1);
                        }
                    }
                    Err(msg) => app.log(format!("Error: {}", msg)),
                }
            }
        }
    }
}

/// Get the id of an item in inventory at a given index
fn get_inventory_id(
    inv: &rustlink::model::ComponentInventory,
    slot: ComponentSlot,
    index: usize,
) -> Option<String> {
    match slot {
        ComponentSlot::Cpu => inv.cpus.get(index).map(|c| c.id.to_string()),
        ComponentSlot::Cooler => inv.coolers.get(index).map(|c| c.id.to_string()),
        ComponentSlot::Motherboard => inv.motherboards.get(index).map(|m| m.id.to_string()),
        ComponentSlot::Ram => inv.rams.get(index).map(|r| r.id.to_string()),
        ComponentSlot::Storage => inv.storage.get(index).map(|s| s.id.to_string()),
        ComponentSlot::Network => inv.networks.get(index).map(|n| n.id.to_string()),
    }
}
