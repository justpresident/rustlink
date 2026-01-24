use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{
        Color, Constraint, CrosstermBackend, Direction, Frame, Layout, Style, Stylize, Terminal,
    },
    widgets::{canvas::*, *},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

// --- DATA MODEL ---

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub size: u32,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct FileSystem {
    pub files: Vec<File>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolState {
    Idle,
    Running { progress: f64, target_ip: String },
    Complete,
}

#[derive(Debug, Clone)]
pub struct Mission {
    pub id: u32,
    pub description: String,
    pub target_ip: String,
    pub target_file: String,
    pub reward: u32,
    pub is_complete: bool,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub coords: (f64, f64),
    pub fs: FileSystem,
    pub is_locked: bool,
    pub password: Option<String>,
}

pub struct App {
    pub servers: HashMap<String, Server>,
    pub connection_path: Vec<String>,
    pub trace_percentage: f64,
    pub is_tracing: bool,
    pub logs: Vec<String>,
    pub input: String,
    pub last_tick: Instant,
    pub should_quit: bool,

    // Hacking mechanics
    pub active_tool: ToolState,
    pub target_ip: Option<String>,
    pub inventory: Vec<String>, // List of software names

    // Mission/economy system
    pub local_files: Vec<File>,
    pub credits: u32,
    pub missions: Vec<Mission>,
}

impl App {
    pub fn new() -> Self {
        let mut servers = HashMap::new();

        // Setup Bank Server with Files
        let bank_fs = FileSystem {
            files: vec![
                File {
                    name: "accounts.dat".into(),
                    size: 450,
                    content: "SECURE DATA".into(),
                },
                File {
                    name: "transfer_logs.log".into(),
                    size: 120,
                    content: "Log entry...".into(),
                },
            ],
        };

        servers.insert(
            "212.43.10.5".into(),
            Server {
                name: "Global Trust Bank".into(),
                ip: "212.43.10.5".into(),
                coords: (139.69, 35.68), // Tokyo
                fs: bank_fs,
                is_locked: true,
                password: Some("admin123".into()),
            },
        );

        // Setup Home
        servers.insert(
            "127.0.0.1".into(),
            Server {
                name: "Home Gateway".into(),
                ip: "127.0.0.1".into(),
                coords: (-0.12, 51.50), // London
                fs: FileSystem::default(),
                is_locked: false,
                password: None,
            },
        );

        // Setup Public DNS
        servers.insert(
            "8.8.8.8".into(),
            Server {
                name: "Public DNS".into(),
                ip: "8.8.8.8".into(),
                coords: (-122.08, 37.38), // Mountain View
                fs: FileSystem::default(),
                is_locked: false,
                password: None,
            },
        );

        // Setup Central Data server
        servers.insert(
            "172.16.0.4".into(),
            Server {
                name: "Central Data".into(),
                ip: "172.16.0.4".into(),
                coords: (-74.00, 40.71), // NYC
                fs: FileSystem {
                    files: vec![File {
                        name: "research.doc".into(),
                        size: 256,
                        content: "Classified research data...".into(),
                    }],
                },
                is_locked: true,
                password: Some("secret456".into()),
            },
        );

        Self {
            servers,
            connection_path: vec!["127.0.0.1".into()],
            trace_percentage: 0.0,
            is_tracing: false,
            logs: vec!["Uplink OS v0.0.1 - Hacker Edition".into()],
            input: String::new(),
            last_tick: Instant::now(),
            should_quit: false,
            active_tool: ToolState::Idle,
            target_ip: None,
            inventory: vec!["PasswordBreaker".into(), "FileManager".into()],
            local_files: vec![],
            credits: 500,
            missions: vec![
                Mission {
                    id: 1,
                    description: "Steal 'research.doc' from Central Data".into(),
                    target_ip: "172.16.0.4".into(),
                    target_file: "research.doc".into(),
                    reward: 2000,
                    is_complete: false,
                },
                Mission {
                    id: 2,
                    description: "Download 'accounts.dat' from Global Trust Bank".into(),
                    target_ip: "212.43.10.5".into(),
                    target_file: "accounts.dat".into(),
                    reward: 5000,
                    is_complete: false,
                },
            ],
        }
    }

    pub fn on_tick(&mut self) {
        // Handle Cracking Progress
        if let ToolState::Running {
            ref mut progress,
            ref target_ip,
        } = self.active_tool
        {
            *progress += 2.5; // Speed of the tool
            if *progress >= 100.0 {
                self.logs
                    .push(format!("SUCCESS: Target {} bypassed.", target_ip));
                if let Some(server) = self.servers.get_mut(target_ip) {
                    server.is_locked = false;
                }
                self.active_tool = ToolState::Complete;
            }
        }

        // Handle Trace
        if self.is_tracing {
            if self.trace_percentage < 100.0 {
                self.trace_percentage += 0.1;
            } else {
                self.logs
                    .push("!!! TERMINAL COMPROMISED - DISCONNECTING !!!".into());
                self.reset_connection();
            }
        }
    }

    fn reset_connection(&mut self) {
        self.connection_path = vec!["127.0.0.1".into()];
        self.target_ip = None;
        self.is_tracing = false;
        self.trace_percentage = 0.0;
        self.active_tool = ToolState::Idle;
        self.credits = self.credits.saturating_sub(100); // Penalty for getting traced
    }

    fn check_missions(&mut self, filename: &str) {
        for m in self.missions.iter_mut() {
            if !m.is_complete && m.target_file == filename {
                m.is_complete = true;
                self.credits += m.reward;
                self.logs.push(format!("MISSION COMPLETE: +{}c", m.reward));
            }
        }
    }

    pub fn handle_command(&mut self) {
        let input = self.input.trim().to_string();
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" => self.logs.push(
                "Commands: connect <ip>, ls, scp <file>, crack, run <tool>, disconnect, clear, exit"
                    .into(),
            ),
            "clear" => self.logs.clear(),
            "connect" => {
                if let Some(&ip) = parts.get(1) {
                    if self.servers.contains_key(ip) {
                        self.connection_path.push(ip.to_string());
                        self.target_ip = Some(ip.to_string());
                        self.is_tracing = true; // Starting a connection starts the trace!
                        self.logs.push(format!("Connected to {}", ip));
                    } else {
                        self.logs.push(format!("Error: Unknown IP {}", ip));
                    }
                }
            }
            "ls" => {
                if let Some(target) = &self.target_ip {
                    let server = &self.servers[target];
                    if server.is_locked {
                        self.logs.push("Access Denied: Server Locked".into());
                    } else {
                        let files: Vec<String> =
                            server.fs.files.iter().map(|f| f.name.clone()).collect();
                        self.logs.push(format!("Files: {}", files.join(", ")));
                    }
                }
            }
            "run" => {
                if let (Some(tool), Some(target)) = (parts.get(1), &self.target_ip) {
                    if *tool == "PasswordBreaker" {
                        self.active_tool = ToolState::Running {
                            progress: 0.0,
                            target_ip: target.clone(),
                        };
                        self.logs
                            .push(format!("Running PasswordBreaker on {}...", target));
                    }
                }
            }
            "scp" => {
                if let (Some(target), Some(&fname)) = (self.target_ip.clone(), parts.get(1)) {
                    let server = &self.servers[&target];
                    if server.is_locked {
                        self.logs.push("Access Denied: Server Locked".into());
                    } else if let Some(f) = server.fs.files.iter().find(|f| f.name == fname) {
                        self.local_files.push(f.clone());
                        self.logs
                            .push(format!("File '{}' downloaded successfully.", fname));
                        self.check_missions(fname);
                    } else {
                        self.logs.push(format!("File '{}' not found.", fname));
                    }
                }
            }
            "crack" => {
                if let Some(target) = &self.target_ip {
                    self.active_tool = ToolState::Running {
                        progress: 0.0,
                        target_ip: target.clone(),
                    };
                    self.logs.push(format!("Cracking {}...", target));
                }
            }
            "disconnect" => self.reset_connection(),
            "exit" => self.should_quit = true,
            _ => self.logs.push(format!("Unknown command: {}", parts[0])),
        }
        self.input.clear();
    }
}

// --- UI RENDERING ---

fn render(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Trace
            Constraint::Min(10),    // World/Interaction
            Constraint::Length(10), // Logs & Tools
            Constraint::Length(3),  // Input
        ])
        .split(f.area());

    // 1. Trace Gauge (with "Bouncing" Red/Cyan look)
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" ACTIVE TRACE ")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(if app.trace_percentage > 70.0 {
            Color::Red
        } else {
            Color::Yellow
        }))
        .percent(app.trace_percentage as u16);
    f.render_widget(gauge, main_layout[0]);

    // 2. Interaction Layer
    let interaction_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[1]);

    // MAP: Using Canvas
    let servers_clone: Vec<Server> = app.servers.values().cloned().collect();
    let path_clone = app.connection_path.clone();
    let servers_map_clone = app.servers.clone();
    let map = Canvas::default()
        .block(Block::default().title(" WORLD MAP ").borders(Borders::ALL))
        .x_bounds([-180.0, 180.0]) // Longitude
        .y_bounds([-90.0, 90.0]) // Latitude
        .paint(move |ctx| {
            // Draw world map background
            ctx.draw(&Map {
                color: Color::DarkGray,
                resolution: MapResolution::High,
            });
            // Draw all nodes
            for server in &servers_clone {
                ctx.print(server.coords.0, server.coords.1, "●".fg(Color::LightRed));
                ctx.print(
                    server.coords.0,
                    server.coords.1 - 3.0,
                    ratatui::prelude::Span::styled(server.name.clone(), Style::new().red()),
                );
                ctx.print(
                    server.coords.0,
                    server.coords.1 - 6.0,
                    ratatui::prelude::Span::styled(
                        format!("[{}]", server.ip),
                        Style::new().light_red().bold(),
                    ),
                );
            }
            // Draw active connection path
            for i in 0..path_clone.len().saturating_sub(1) {
                if let (Some(s1), Some(s2)) = (
                    servers_map_clone.get(&path_clone[i]),
                    servers_map_clone.get(&path_clone[i + 1]),
                ) {
                    ctx.draw(&Line {
                        x1: s1.coords.0,
                        y1: s1.coords.1,
                        x2: s2.coords.0,
                        y2: s2.coords.1,
                        color: Color::Yellow,
                    });
                }
            }
        });
    f.render_widget(map, interaction_chunks[0]);

    // Tool/Server Info Panel
    let target_name = app
        .target_ip
        .as_ref()
        .map(|ip| app.servers[ip].name.as_str())
        .unwrap_or("None");
    let local_file_names: Vec<&str> = app.local_files.iter().map(|f| f.name.as_str()).collect();
    let info_text = format!(
        "Credits: {}c\n\nTarget: {}\nStatus: {}\n\nLocal Files: {:?}\nSoftware: {:?}",
        app.credits,
        target_name,
        if app.target_ip.is_some() {
            "CONNECTED"
        } else {
            "IDLE"
        },
        local_file_names,
        app.inventory
    );
    f.render_widget(
        Paragraph::new(info_text).block(
            Block::default()
                .title(" SYSTEM STATUS ")
                .borders(Borders::ALL),
        ),
        interaction_chunks[1],
    );

    // 3. Bottom HUD (Logs + Missions/Tool Progress)
    let hud_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[2]);

    let logs = List::new(
        app.logs
            .iter()
            .rev()
            .map(|l| ListItem::new(l.as_str()))
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title(" LOGS ").borders(Borders::ALL));
    f.render_widget(logs, hud_chunks[0]);

    // Right panel: Tool progress or Missions
    if let ToolState::Running { progress, .. } = app.active_tool {
        let tool_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(" TOOL PROGRESS ")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(progress as u16);
        f.render_widget(tool_gauge, hud_chunks[1]);
    } else {
        // Show missions panel
        let mission_items: Vec<ListItem> = app
            .missions
            .iter()
            .map(|m| {
                let status = if m.is_complete { "[DONE]" } else { "[OPEN]" };
                ListItem::new(format!("{} {} (+{}c)", status, m.description, m.reward))
            })
            .collect();
        f.render_widget(
            List::new(mission_items)
                .block(Block::default().title(" MISSIONS ").borders(Borders::ALL)),
            hud_chunks[1],
        );
    }

    // 4. Console
    f.render_widget(
        Paragraph::new(format!("> {}", app.input))
            .block(Block::default().borders(Borders::ALL).fg(Color::Yellow)),
        main_layout[3],
    );
}
// --- MAIN LOOP ---

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
                        KeyCode::Enter => app.handle_command(),
                        _ => {}
                    }
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

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
