use ratatui::{
    prelude::{Color, Constraint, Direction, Frame, Layout, Style, Stylize},
    widgets::{canvas::*, *},
};

use crate::app::App;
use crate::commands::CommandRegistry;
use crate::model::Server;

pub fn render(f: &mut Frame, app: &mut App, registry: &CommandRegistry) {
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
    let animation_tick = app.animation_tick; // Capture animation_tick by value

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
            let color_cycle = [Color::LightYellow, Color::Yellow, Color::Green, Color::Blue];
            let current_color_idx = (animation_tick / 5) as usize % color_cycle.len();
            let animated_color = color_cycle[current_color_idx];

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
                        color: animated_color,
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
    let unread_mail_count = app.inbox.iter().filter(|m| !m.is_read).count();
    let mail_status = if unread_mail_count > 0 {
        format!("{} NEW", unread_mail_count)
    } else {
        "None".to_string()
    };
    let local_file_names: Vec<&str> = app.local_files.iter().map(|f| f.name.as_str()).collect();
    let tool_names = registry.tool_registry.names();
    let info_text = format!(
        "Credits: {}c\n\nTarget: {}\nStatus: {}\nFirewall: {}\nUnread Mail: {}\n\nLocal Files: {:?}\nTools: {:?}",
        app.credits,
        target_name,
        if app.target_ip.is_some() {
            "CONNECTED"
        } else {
            "IDLE"
        },
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
        interaction_chunks[1],
    );

    // 3. Bottom HUD (Logs + Missions/Tool Progress)
    let hud_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[2]);

    // Calculate how many lines fit in the logs area (height - 2 for borders)
    let logs_height = hud_chunks[0].height.saturating_sub(2) as usize;
    let logs_to_show: Vec<ListItem> = app
        .logs
        .iter()
        .skip(app.logs.len().saturating_sub(logs_height))
        .map(|l| ListItem::new(l.as_str()))
        .collect();
    let logs =
        List::new(logs_to_show).block(Block::default().title(" LOGS ").borders(Borders::ALL));
    f.render_widget(logs, hud_chunks[0]);

    // Right panel: Tool progress or Missions
    if let Some(ref active_tool) = app.active_tool {
        let tool_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(format!(" {} ", active_tool.tool_name))
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(active_tool.progress as u16);
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

    // 4. Console with cursor
    let input_area = main_layout[3];
    f.render_widget(
        Paragraph::new(format!("> {}", app.input))
            .block(Block::default().borders(Borders::ALL).fg(Color::Yellow)),
        input_area,
    );
    // Set cursor position (account for border and "> " prompt)
    f.set_cursor_position((
        input_area.x + 1 + 2 + app.cursor_pos as u16, // border + "> " + cursor
        input_area.y + 1,                             // border
    ));
}
