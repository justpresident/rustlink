use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{CrosstermBackend, Terminal};
use rustlink::{
    app::App,
    command::handle_command,
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
    let tick_rate = Duration::from_millis(50);

    loop {
        terminal.draw(|f| render(f, &mut app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Char(c) => app.input.push(c),
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => handle_command(&mut app),
                        _ => {}
                    }
                }
            }
        }

        if app.last_tick.elapsed() >= tick_rate {
            app.on_tick();
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