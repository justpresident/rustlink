use clap::Parser;
use crossterm::event::{self, Event};
use ratatui::prelude::{CrosstermBackend, Terminal};
use rustlink::{
    app::App,
    commands::CommandRegistry,
    ui::{KeyResult, ViewRegistry, handle_key, render},
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
            match handle_key(key, &mut app, &mut registry) {
                KeyResult::Continue => {}
                KeyResult::Quit => {
                    app.should_quit = true;
                }
                KeyResult::ConnectionChanged {
                    old_server_type,
                    new_server_type,
                } => {
                    if let Some(old_type) = old_server_type {
                        let commands_to_deactivate: &[&str] = old_type.associated_commands();
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
