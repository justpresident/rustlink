use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{CrosstermBackend, Terminal};
use rustlink::{
    app::App,
    commands::{CommandRegistry, CommandResult, execute_input, get_completions},
    ui::{ViewRegistry, render},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new();
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
                    app.terminal.apply_completions(completions);
                }
                // Enter - execute command
                KeyCode::Enter => {
                    app.terminal.save_to_history();
                    let command_result = execute_input(&mut registry, &mut app); // Capture the result
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
                            // Deactivate commands for the old server type
                            if let Some(old_type) = old_server_type {
                                let commands_to_deactivate: &[&str] =
                                    old_type.associated_commands();
                                registry.deactivate_commands(commands_to_deactivate);
                            }

                            // Activate commands for the new server type
                            if let Some(new_type) = new_server_type {
                                let commands_to_activate: &[&str] = new_type.associated_commands();
                                registry.activate_commands(commands_to_activate);
                                app.log(format!(
                                    "Commands available on the server: {:?}",
                                    commands_to_activate
                                ));
                            } else {
                                // If disconnected to nowhere, clear all context-specific commands
                                // For now, the `disconnect` command returning `Home` and `connect` handling this is sufficient.
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
