use ratatui::{
    prelude::{Color, Constraint, Direction, Frame, Layout, Rect, Style, Stylize},
    widgets::{canvas::{Canvas, Map, MapResolution, Line}, Paragraph, Block, Borders, Gauge, ListItem, List},
};
use std::fmt::Write;

/// Convert a percentage (0.0-100.0) to u16 for gauge widgets
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
const fn percent_to_u16(value: f64) -> u16 {
    if value < 0.0 {
        0
    } else if value > 100.0 {
        100
    } else {
        value as u16
    }
}
use std::collections::HashMap;

use crate::app::App;
use crate::commands::CommandRegistry;
use crate::model::{Server, ServerType};

/// Function signature for server-specific panel renderers
type ServerPanelRenderer = fn(&mut Frame, Rect, &Server);

/// Registry mapping server types to their custom panel renderers
pub struct ViewRegistry {
    renderers: HashMap<ServerType, ServerPanelRenderer>,
}

impl ViewRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            renderers: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        self.register(ServerType::Bank, render_bank_panel);
        // Register more server-type views here as needed
    }

    pub fn register(&mut self, server_type: ServerType, renderer: ServerPanelRenderer) {
        self.renderers.insert(server_type, renderer);
    }

    pub fn render_panel(&self, f: &mut Frame, area: Rect, server: &Server) {
        if let Some(renderer) = self.renderers.get(&server.server_type) {
            renderer(f, area, server);
        } else {
            render_empty_panel(f, area);
        }
    }
}

impl Default for ViewRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Server-specific panel renderers
// ============================================================================

fn render_bank_panel(f: &mut Frame, area: Rect, server: &Server) {
    let mut info_text = format!("Bank: {}\nIP: {}\n\n", server.name, server.ip);

    if server.is_locked {
        info_text.push_str("Access Denied: Server Locked.\nRun PasswordBreaker first.");
    } else if let Some(accounts) = &server.accounts {
        info_text.push_str("--- ACCOUNTS ---\n");
        for account in accounts {
            let _ = writeln!(
                info_text,
                "  {} ({}) : {}c",
                account.account_number, account.owner, account.balance
            );
        }
        info_text.push_str("----------------");
    } else {
        info_text.push_str("No accounts found.");
    }

    f.render_widget(
        Paragraph::new(info_text).block(
            Block::default()
                .title(" BANK ACCOUNTS ")
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn render_empty_panel(f: &mut Frame, area: Rect) {
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" SERVER INFO "),
        area,
    );
}

// ============================================================================
// Generic system status panel (always shown)
// ============================================================================

fn render_system_status(f: &mut Frame, area: Rect, app: &App, registry: &CommandRegistry) {
    let target_name = app
        .connection
        .target_ip
        .as_ref()
        .and_then(|ip| app.world.servers.get(ip))
        .map_or("None", |server| server.name.as_str());

    let firewall_status = app
        .connection
        .target_ip
        .as_ref()
        .and_then(|ip| app.world.servers.get(ip))
        .and_then(|server| server.firewall.as_ref())
        .map_or_else(
            || "N/A".to_string(),
            |fw| {
                if fw.is_active {
                    format!("ACTIVE ({})", fw.strength)
                } else {
                    "DISABLED".to_string()
                }
            },
        );

    let unread_mail_count = app.player.unread_mail_count();
    let mail_status = if unread_mail_count > 0 {
        format!("{unread_mail_count} NEW")
    } else {
        "None".to_string()
    };

    let local_file_names: Vec<&str> = app
        .player
        .local_files
        .iter()
        .map(|f| f.name.as_str())
        .collect();

    let tool_names = registry.tool_registry.names();

    let status = if app.connection.target_ip.is_some() {
        "CONNECTED"
    } else {
        "IDLE"
    };

    let info_text = format!(
        "Credits: {}c\n\nTarget: {}\nStatus: {}\nFirewall: {}\nUnread Mail: {}\n\nLocal Files: {:?}\nTools: {:?}",
        app.player.credits,
        target_name,
        status,
        firewall_status,
        mail_status,
        local_file_names,
        tool_names
    );

    f.render_widget(
        Paragraph::new(info_text).block(
            Block::default()
                .title(" SYSTEM STATUS ")
                .borders(Borders::ALL),
        ),
        area,
    );
}

// ============================================================================
// Main render function
// ============================================================================

pub fn render(
    f: &mut Frame,
    app: &mut App,
    registry: &CommandRegistry,
    view_registry: &ViewRegistry,
) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Trace
            Constraint::Min(10),    // World/Interaction
            Constraint::Length(10), // Logs & Tools
            Constraint::Length(3),  // Input
        ])
        .split(f.area());

    // 1. Trace Gauge
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" ACTIVE TRACE ")
                .borders(Borders::ALL),
        )
        .gauge_style(
            Style::default().fg(if app.connection.trace_percentage > 70.0 {
                Color::Red
            } else {
                Color::Yellow
            }),
        )
        .percent(percent_to_u16(app.connection.trace_percentage));
    f.render_widget(gauge, main_layout[0]);

    // 2. Interaction Layer (Map + Info Panels)
    let interaction_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[1]);

    // World Map
    render_world_map(f, interaction_chunks[0], app);

    // Right panel split: System Status (top) + Server-specific (bottom)
    let right_panel_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(interaction_chunks[1]);

    render_system_status(f, right_panel_chunks[0], app, registry);

    // Server-specific panel via registry
    if let Some(target_ip) = &app.connection.target_ip
        && let Some(server) = app.world.servers.get(target_ip)
    {
        view_registry.render_panel(f, right_panel_chunks[1], server);
    } else {
        render_empty_panel(f, right_panel_chunks[1]);
    }

    // 3. Bottom HUD (Logs + Missions/Tool Progress)
    render_bottom_hud(f, main_layout[2], app);

    // 4. Input Console
    render_input(f, main_layout[3], app);
}

/// Minimal data needed to render a server on the map
struct MapServer {
    name: String,
    ip: String,
    coords: (f64, f64),
}

fn render_world_map(f: &mut Frame, area: Rect, app: &App) {
    // Extract only the data needed for rendering (avoid cloning entire Server structs)
    let servers: Vec<MapServer> = app
        .world
        .servers
        .values()
        .map(|s| MapServer {
            name: s.name.clone(),
            ip: s.ip.clone(),
            coords: s.coords,
        })
        .collect();

    // For the path, we only need coordinates of connected servers
    let path_coords: Vec<(f64, f64)> = app
        .connection
        .path
        .iter()
        .filter_map(|ip| app.world.servers.get(ip).map(|s| s.coords))
        .collect();

    let animation_tick = app.animation_tick;

    let map = Canvas::default()
        .block(Block::default().title(" WORLD MAP ").borders(Borders::ALL))
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
        .paint(move |ctx| {
            ctx.draw(&Map {
                color: Color::DarkGray,
                resolution: MapResolution::High,
            });

            for server in &servers {
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

            let color_cycle = [Color::LightYellow, Color::Yellow, Color::Green, Color::Blue];
            let current_color_idx = (animation_tick / 5) as usize % color_cycle.len();
            let animated_color = color_cycle[current_color_idx];

            for window in path_coords.windows(2) {
                ctx.draw(&Line {
                    x1: window[0].0,
                    y1: window[0].1,
                    x2: window[1].0,
                    y2: window[1].1,
                    color: animated_color,
                });
            }
        });

    f.render_widget(map, area);
}

fn render_bottom_hud(f: &mut Frame, area: Rect, app: &mut App) {
    let hud_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Logs panel
    let logs_height = hud_chunks[0].height.saturating_sub(2) as usize;
    let max_scroll = app.terminal.logs_len().saturating_sub(logs_height);
    if app.terminal.log_scroll > max_scroll {
        app.terminal.log_scroll = max_scroll;
    }

    let end_index = app
        .terminal
        .logs_len()
        .saturating_sub(app.terminal.log_scroll);
    let start_index = end_index.saturating_sub(logs_height);
    let logs_to_show: Vec<ListItem> = app.terminal.logs()[start_index..end_index]
        .iter()
        .map(|l| ListItem::new(l.as_str()))
        .collect();

    let scroll_indicator = if app.terminal.log_scroll > 0 {
        format!(" LOGS [+{}] ", app.terminal.log_scroll)
    } else {
        " LOGS ".to_string()
    };

    f.render_widget(
        List::new(logs_to_show).block(
            Block::default()
                .title(scroll_indicator)
                .borders(Borders::ALL),
        ),
        hud_chunks[0],
    );

    // Right panel: Tool progress or Missions
    if let Some(ref active_tool) = app.connection.active_tool {
        let tool_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(format!(" {} ", active_tool.tool_name))
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(percent_to_u16(active_tool.progress));
        f.render_widget(tool_gauge, hud_chunks[1]);
    } else {
        let mission_items: Vec<ListItem> = app
            .player
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
}

fn render_input(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(
        Paragraph::new(format!("> {}", app.terminal.input))
            .block(Block::default().borders(Borders::ALL).fg(Color::Yellow)),
        area,
    );
    // Clamp cursor position to stay within the input area bounds
    let cursor_pos_u16 = u16::try_from(app.terminal.cursor_pos).expect("cursor_pos exceeds u16::MAX");
    let cursor_x = (area.x + 3 + cursor_pos_u16).min(area.x + area.width.saturating_sub(2));
    f.set_cursor_position((cursor_x, area.y + 1));
}
