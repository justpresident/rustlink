use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{CrosstermBackend, Terminal};
use rustlink::{
    app::App,
    commands::{execute_input, get_completions, CommandRegistry},
    ui::render,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new();
    let registry = CommandRegistry::new();
    let tick_rate = Duration::from_millis(50);

    loop {
        terminal.draw(|f| render(f, &mut app, &registry))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        // Quit
                        KeyCode::Char('q')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        // Ctrl+A - move to start of line
                        KeyCode::Char('a')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.move_cursor_start();
                        }
                        // Ctrl+E - move to end of line
                        KeyCode::Char('e')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.move_cursor_end();
                        }
                        // Ctrl+U - clear line
                        KeyCode::Char('u')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.clear_line();
                        }
                        // Ctrl+W - delete word
                        KeyCode::Char('w')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.delete_word();
                        }
                        // Regular character input
                        KeyCode::Char(c) => app.insert_char(c),
                        // Backspace - delete char before cursor
                        KeyCode::Backspace => app.delete_char(),
                        // Delete - delete char at cursor
                        KeyCode::Delete => app.delete_char_forward(),
                        // Arrow keys
                        KeyCode::Left => app.move_cursor_left(),
                        KeyCode::Right => app.move_cursor_right(),
                        KeyCode::Up => app.history_up(),
                        KeyCode::Down => app.history_down(),
                        // Home/End
                        KeyCode::Home => app.move_cursor_start(),
                        KeyCode::End => app.move_cursor_end(),
                        // Tab - autocomplete
                        KeyCode::Tab => {
                            let completions = get_completions(&registry, &app, &app.input);
                            app.apply_completions(completions);
                        }
                        // Enter - execute command
                        KeyCode::Enter => {
                            app.save_to_history();
                            execute_input(&registry, &mut app);
                            app.input.clear();
                            app.cursor_pos = 0;
                        }
                        _ => {}
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