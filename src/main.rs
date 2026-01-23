use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{
    Color, Constraint, CrosstermBackend, Direction, Frame, Layout, Style, Terminal,
};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

// --- DATA MODEL ---

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub security_level: u8,
    pub is_compromised: bool,
    pub coords: (f64, f64), // For the map: (x, y)
}

pub struct App {
    pub servers: HashMap<String, Server>,
    pub connection_path: Vec<String>, // List of IPs
    pub trace_percentage: f64,
    pub is_tracing: bool,
    pub logs: Vec<String>,
    pub last_tick: Instant,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut servers = HashMap::new();
        // Seed some data
        servers.insert(
            "127.0.0.1".into(),
            Server {
                name: "Home Gateway".into(),
                ip: "127.0.0.1".into(),
                security_level: 0,
                is_compromised: true,
                coords: (10.0, 10.0),
            },
        );
        servers.insert(
            "212.43.10.5".into(),
            Server {
                name: "Global Trust Bank".into(),
                ip: "212.43.10.5".into(),
                security_level: 5,
                is_compromised: false,
                coords: (60.0, 30.0),
            },
        );

        Self {
            servers,
            connection_path: vec!["127.0.0.1".into()],
            trace_percentage: 0.0,
            is_tracing: false,
            logs: vec!["System Initialized...".into()],
            last_tick: Instant::now(),
            should_quit: false,
        }
    }

    pub fn on_tick(&mut self) {
        if self.is_tracing {
            if self.trace_percentage >= 100.0 {
                self.logs.push("FATAL ERROR: CONNECTION TRACED".into());
                self.is_tracing = false;
            } else {
                // Trace speed could be a function of the path length and security_level
                self.trace_percentage += 0.5;
            }
        }
    }
}

// --- UI RENDERING ---

fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main Map/Interface
            Constraint::Length(8), // Bottom Logs
        ])
        .split(f.area());

    // Header: Trace Gauge
    let trace_color = if app.trace_percentage > 70.0 {
        Color::Red
    } else {
        Color::Green
    };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" TRACE PROGRESS ")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(trace_color).bg(Color::Black))
        .percent(app.trace_percentage as u16);
    f.render_widget(gauge, chunks[0]);

    // Main: Map and Servers (Simplified for now)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[1]);

    let server_list: Vec<ListItem> = app
        .servers
        .values()
        .map(|s| ListItem::new(format!("{} [{}]", s.name, s.ip)))
        .collect();
    let list = List::new(server_list).block(
        Block::default()
            .title(" Network Nodes ")
            .borders(Borders::ALL),
    );
    f.render_widget(list, main_chunks[1]);

    // Logs
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .map(|l| ListItem::new(l.as_str()))
        .collect();
    let log_list = List::new(log_items).block(
        Block::default()
            .title(" System Logs ")
            .borders(Borders::ALL),
    );
    f.render_widget(log_list, chunks[2]);
}

// --- MAIN LOOP ---

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TUI Setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| render(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(app.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char('t') => app.is_tracing = !app.is_tracing, // Toggle trace for testing
                    _ => {}
                }
            }
        }

        if app.last_tick.elapsed() >= tick_rate {
            app.on_tick();
            app.last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
