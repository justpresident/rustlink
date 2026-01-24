use ratatui::{
    prelude::{
        Color, Constraint, Direction, Frame, Layout, Style, Stylize,
    },
    widgets::{canvas::*, *},
};

use crate::app::App;
use crate::model::{Server, ToolState};

pub fn render(f: &mut Frame, app: &mut App) {
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
    let firewall_status = if let Some(ip) = &app.target_ip {
        if let Some(server) = app.servers.get(ip) {
            if let Some(firewall) = &server.firewall {
                if firewall.is_active {
                    format!("ACTIVE ({})", firewall.strength)
                } else {
                    "DISABLED".to_string()
                }
            } else {
                "N/A".to_string()
            }
        } else {
            "N/A".to_string()
        }
    } else {
        "N/A".to_string()
    };
    let local_file_names: Vec<&str> = app.local_files.iter().map(|f| f.name.as_str()).collect();
    let info_text = format!(
        "Credits: {}c\n\nTarget: {}\nStatus: {}\nFirewall: {}\n\nLocal Files: {:?}\nSoftware: {:?}",
        app.credits,
        target_name,
        if app.target_ip.is_some() {
            "CONNECTED"
        } else {
            "IDLE"
        },
        firewall_status,
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
