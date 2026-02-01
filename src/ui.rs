use ratatui::{
    prelude::{Color, Constraint, Direction, Frame, Layout, Rect, Style, Stylize},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph, Tabs,
        canvas::{Canvas, Line, Map, MapResolution},
    },
};
use std::fmt::Write as _;

/// Convert a percentage (0.0-100.0) to u16 for gauge widgets
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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

use crate::app::{App, UIMode};
use crate::commands::CommandRegistry;
use crate::model::{HardwareKind, MotherboardTier, Server, ServerType};
use crate::shop::Shop;

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
    // Check UI mode and render accordingly
    match app.ui_mode {
        UIMode::Shop => {
            render_shop(f, app);
            return;
        }
        UIMode::Normal => {}
    }

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
        // Show active missions from mission system
        let mission_items: Vec<ListItem> = app
            .missions
            .all_active()
            .iter()
            .map(|m| {
                let status = if m.is_complete() {
                    "[DONE]"
                } else {
                    "[ACTIVE]"
                };
                ListItem::new(format!("{} {} (+{}c)", status, m.title, m.reward))
            })
            .collect();

        let title = if mission_items.is_empty() {
            " MISSIONS (none active) "
        } else {
            " MISSIONS "
        };

        f.render_widget(
            List::new(mission_items).block(Block::default().title(title).borders(Borders::ALL)),
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
    let cursor_pos_u16 =
        u16::try_from(app.terminal.cursor_pos).expect("cursor_pos exceeds u16::MAX");
    let cursor_x = (area.x + 3 + cursor_pos_u16).min(area.x + area.width.saturating_sub(2));
    f.set_cursor_position((cursor_x, area.y + 1));
}

// ============================================================================
// Shop View - Complete redesign with proper visualization
// ============================================================================

fn render_shop(f: &mut Frame, app: &App) {
    use crate::app::ShopTab;

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title + tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Controls
        ])
        .split(f.area());

    // Title and category tabs
    render_shop_header(f, main_layout[0], app);

    // Content: PC Status (left) + Items (right)
    // Give more space to the PC visualization
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    render_pc_status_detailed(f, content_layout[0], app);
    render_shop_items(f, content_layout[1], app);

    // Controls - different based on tab
    let controls = match app.shop_tab {
        ShopTab::Available => "Tab: Owned | ←/→: Category | ↑/↓: Select | Enter: Buy | Esc: Exit",
        ShopTab::Owned => {
            "Tab: Shop | ←/→: Category | ↑/↓: Select | Enter: Install | Backspace: Uninstall | Esc: Exit"
        }
    };

    let can_exit = app.player.pc_functional();
    let exit_hint = if can_exit {
        ""
    } else {
        " [PC incomplete - cannot exit]"
    };

    f.render_widget(
        Paragraph::new(format!("{controls}{exit_hint}"))
            .style(Style::default().fg(if can_exit {
                Color::DarkGray
            } else {
                Color::Red
            }))
            .block(Block::default().borders(Borders::ALL)),
        main_layout[2],
    );
}

fn render_shop_header(f: &mut Frame, area: Rect, app: &App) {
    use crate::app::ShopTab;

    let titles: Vec<&str> = HardwareKind::all().iter().map(HardwareKind::name).collect();

    let selected_idx = HardwareKind::all()
        .iter()
        .position(|c| *c == app.shop_category)
        .unwrap_or(0);

    // Title shows current mode
    let title = match app.shop_tab {
        ShopTab::Available => " PC SHOP - AVAILABLE ",
        ShopTab::Owned => " PC SHOP - YOUR INVENTORY ",
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().title(title).borders(Borders::ALL))
        .select(selected_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn render_pc_status_detailed(f: &mut Frame, area: Rect, app: &App) {
    let functional = app.player.pc_functional();

    // Main block
    let title = if functional {
        " YOUR PC "
    } else {
        " YOUR PC [NOT FUNCTIONAL] "
    };
    let border_color = if functional { Color::Cyan } else { Color::Red };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Inner layout for credits header + PC ASCII art
    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Credits line
            Constraint::Min(1),    // Large PC ASCII art
        ])
        .split(inner_area);

    // Credits + inventory count
    let inv_count = app.player.inventory.total_count();
    let inv_text = if inv_count > 0 {
        format!(" | Inventory: {inv_count} parts")
    } else {
        String::new()
    };
    f.render_widget(
        Paragraph::new(format!("Credits: {}c{}", app.player.credits, inv_text)).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        inner_layout[0],
    );

    // Render the large PC ASCII art
    let pc_lines = render_large_pc_ascii(app);
    f.render_widget(Paragraph::new(pc_lines), inner_layout[1]);
}

/// Item to display in the shop list
struct ShopDisplayItem {
    name: String,
    description: String,
    price: Option<u32>,
    socket_info: Option<String>,
    warnings: Vec<String>,
    affordable: bool,
    installed: bool,
    count: usize, // Number of items in this group (1 for shop items and installed)
}

fn render_shop_items(f: &mut Frame, area: Rect, app: &App) {
    use crate::app::ShopTab;

    let inv = &app.player.inventory;
    let kind = app.shop_category;

    // Build display items and configure based on tab
    let (items, title_suffix, border_color, empty_msg): (Vec<ShopDisplayItem>, _, _, _) =
        match app.shop_tab {
            ShopTab::Available => {
                let items = Shop::items_for(app.shop_category)
                    .map(|item| ShopDisplayItem {
                        name: item.name().to_string(),
                        description: item.description(),
                        price: Some(item.price()),
                        socket_info: item.socket_info(),
                        warnings: Shop::purchase_warnings(&app.player.inventory, item),
                        affordable: app.player.credits >= item.price(),
                        installed: false,
                        count: 1,
                    })
                    .collect();
                (
                    items,
                    "For Sale",
                    Color::Cyan,
                    "No items available in this category.",
                )
            }
            ShopTab::Owned => {
                // Use grouped display: sorted and grouped by model for spare items
                let items = inv
                    .grouped_display(kind)
                    .into_iter()
                    .map(|group| {
                        let warnings = if group.installed {
                            Vec::new()
                        } else {
                            inv.can_install(group.first_index())
                                .err()
                                .into_iter()
                                .collect()
                        };
                        ShopDisplayItem {
                            name: group.component.name().to_string(),
                            description: group.component.description(),
                            price: None,
                            socket_info: group.component.socket_info(),
                            warnings,
                            affordable: true,
                            installed: group.installed,
                            count: group.count(),
                        }
                    })
                    .collect();
                (
                    items,
                    "Your Inventory",
                    Color::Green,
                    "No items in inventory.\n\nSwitch to Available tab to purchase.",
                )
            }
        };

    let title = format!(" {} - {} ", app.shop_category.name(), title_suffix);
    render_shop_list(
        f,
        title,
        area,
        &items,
        app.shop_selection,
        border_color,
        empty_msg,
    );
}

/// Generic shop list renderer
fn render_shop_list(
    f: &mut Frame,
    title: String,
    area: Rect,
    items: &[ShopDisplayItem],
    selection: usize,
    border_color: Color,
    empty_msg: &str,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if items.is_empty() {
        f.render_widget(Paragraph::new(empty_msg).block(block), area);
        return;
    }

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let selected = i == selection;

            // Build display text
            let (lines, style) = get_shop_item_list_lines(selected, item);

            ListItem::new(lines.join("\n")).style(style)
        })
        .collect();

    f.render_widget(List::new(list_items).block(block), area);
}

fn get_shop_item_list_lines(selected: bool, item: &ShopDisplayItem) -> (Vec<String>, Style) {
    let prefix = if selected { "> " } else { "  " };
    let checkbox = if item.installed { "[x]" } else { "[ ]" };
    let count_suffix = if item.count > 1 {
        format!(" (x{})", item.count)
    } else {
        String::new()
    };
    let mut lines = vec![if let Some(p) = item.price {
        format!("{prefix}{p}$ {} [{}]", item.name, item.description)
    } else {
        format!(
            "{prefix}{checkbox} {}{count_suffix} [{}]",
            item.name, item.description
        )
    }];

    let warnings_str = if item.warnings.is_empty() {
        String::new()
    } else {
        format!("⚠ {}", item.warnings.join("; "))
    };
    if let Some(ref info) = item.socket_info {
        lines.push(format!("    Socket/Type: {info} {warnings_str}"));
    }
    // Determine style
    let style = if selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(ratatui::style::Modifier::BOLD)
    } else if !item.affordable {
        Style::default().fg(Color::DarkGray)
    } else if item.warnings.is_empty() {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };
    (lines, style)
}

// ============================================================================
// Large ASCII PC Case Visualization
// ============================================================================

/// Render a large ASCII art PC case with internal components
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
pub fn render_large_pc_ascii(app: &App) -> Vec<ratatui::text::Line<'static>> {
    use ratatui::text::{Line, Span};

    let inv = &app.player.inventory;
    let mb = inv.installed_motherboard();

    // Colors based on motherboard tier (or default if no motherboard)
    let (case_color, accent_color, led_color) = mb.map_or(
        (Color::DarkGray, Color::Gray, Color::DarkGray),
        |m| match m.tier {
            MotherboardTier::Basic => (Color::DarkGray, Color::Gray, Color::Gray),
            MotherboardTier::Standard => (Color::Gray, Color::Blue, Color::Blue),
            MotherboardTier::Performance => (Color::Cyan, Color::LightCyan, Color::LightCyan),
            MotherboardTier::Enthusiast => (Color::Yellow, Color::LightYellow, Color::LightRed),
        },
    );

    let case_style = Style::default().fg(case_color);
    let accent_style = Style::default().fg(accent_color);
    let led_style = Style::default().fg(led_color);
    let dim_style = Style::default().fg(Color::DarkGray);
    let cpu_style = Style::default().fg(Color::Cyan);
    let ram_style = Style::default().fg(Color::Magenta);
    let storage_style = Style::default().fg(Color::Blue);
    let net_style = Style::default().fg(Color::Green);
    let psu_style = Style::default().fg(Color::Yellow);

    // Collect component info from inventory
    let has_mb = mb.is_some();
    let cpu_filled = inv.count_installed(HardwareKind::Cpu) > 0;
    let cooler_filled = inv.count_installed(HardwareKind::Cooler) > 0;
    let ram_filled = inv.count_installed(HardwareKind::Ram);
    let ram_total = mb.map_or(0, |m| m.slot_count(HardwareKind::Ram));
    let storage_filled = inv.count_installed(HardwareKind::Storage);
    let storage_total = mb.map_or(0, |m| m.slot_count(HardwareKind::Storage));
    let net_filled = inv.count_installed(HardwareKind::Network);
    let net_total = mb.map_or(0, |m| m.slot_count(HardwareKind::Network));
    let functional = inv.is_functional();

    // Get component names for display
    let cpu_name = inv
        .cpu()
        .map_or_else(|| "Empty".to_string(), |c| c.name.to_string());
    let cooler_name = inv
        .cooler()
        .map_or_else(|| "None".to_string(), |c| c.name.to_string());
    let mb_name = mb.map_or_else(|| "No Motherboard".to_string(), |m| m.name.to_string());
    let cpu_socket_name = mb
        .and_then(crate::model::hardware::Motherboard::socket)
        .map_or_else(|| "N/A".to_string(), |s| format!("{s}"));
    let ram_socket_name = mb
        .and_then(crate::model::hardware::Motherboard::ram_type)
        .map_or_else(|| "N/A".to_string(), |s| format!("{s}"));

    let mut lines = Vec::new();

    // Power LED indicator
    let power_led = if functional { "●" } else { "○" };
    let power_color = if functional { Color::Green } else { Color::Red };

    // Case top with vents
    lines.push(Line::from(vec![
        Span::styled("  ╔", case_style),
        Span::styled("═══════════════════════════════════════", case_style),
        Span::styled("╗", case_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄ ", dim_style),
        Span::styled("║", case_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" █                                   █ ", dim_style),
        Span::styled("║", case_style),
    ]));

    // Case front with power button and status
    let hdd_indicator = if storage_filled > 0 { "●" } else { "○" };
    let activity_led = if functional { "▓▓▓" } else { "░░░" };
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" █  ", dim_style),
        Span::styled(power_led.to_string(), Style::default().fg(power_color)),
        Span::styled(" POWER   ", dim_style),
        Span::styled(activity_led.to_string(), led_style),
        Span::styled(" HDD ", dim_style),
        Span::styled(hdd_indicator.to_string(), storage_style),
        Span::styled("              █ ", dim_style),
        Span::styled("║", case_style),
    ]));

    // Separator
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" █▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█ ", dim_style),
        Span::styled("║", case_style),
    ]));

    // Internal view header
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" ╔═══════════════════════════════════╗ ", accent_style),
        Span::styled("║", case_style),
    ]));

    // Motherboard area
    if has_mb {
        // CPU/Cooler section
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║ ", accent_style),
            Span::styled("┌──────────────┐", cpu_style),
            Span::styled("  RAM SLOTS       ", dim_style),
            Span::styled("║ ", accent_style),
            Span::styled("║", case_style),
        ]));

        // CPU with cooler visualization
        let cpu_art = if cooler_filled {
            "│ ▓▓▓▓▓▓▓▓▓▓▓▓ │".to_string()
        } else if cpu_filled {
            "│ ┌────────┐   │".to_string()
        } else {
            format!("│ {cpu_socket_name:<13}│")
        };
        let cpu_color = if cpu_filled { cpu_style } else { dim_style };

        // RAM slot visualization
        let mut ram_slots = String::new();
        for i in 0..ram_total {
            if i < ram_filled {
                ram_slots.push('█');
            } else {
                ram_slots.push('░');
            }
            ram_slots.push(' ');
        }
        let ram_slots_display = format!("{ram_slots:<16}");

        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║ ", accent_style),
            Span::styled(cpu_art, cpu_color),
            Span::styled("  ", dim_style),
            Span::styled(ram_slots_display.clone(), ram_style),
            Span::styled("║ ", accent_style),
            Span::styled("║", case_style),
        ]));

        // CPU label row
        let cpu_label = if cooler_filled {
            "│   COOLER     │".to_string()
        } else if cpu_filled {
            "│ │  CPU   │   │".to_string()
        } else {
            "│ │ EMPTY  │   │".to_string()
        };

        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║ ", accent_style),
            Span::styled(cpu_label, cpu_color),
            Span::styled("  ", dim_style),
            Span::styled(ram_slots_display, ram_style),
            Span::styled("║ ", accent_style),
            Span::styled("║", case_style),
        ]));

        // CPU bottom / socket info
        let ram_socket_display = format!("  {ram_socket_name:<16}");
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║ ", accent_style),
            Span::styled("└──────────────┘", cpu_style),
            Span::styled(ram_socket_display, dim_style),
            Span::styled("║ ", accent_style),
            Span::styled("║", case_style),
        ]));

        // Spacer
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║                                   ║ ", accent_style),
            Span::styled("║", case_style),
        ]));

        // Storage section header
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║ ", accent_style),
            Span::styled("STORAGE DRIVES", storage_style),
            Span::styled("                 ", dim_style),
            Span::styled("", net_style),
            Span::styled("   ║ ", accent_style),
            Span::styled("║", case_style),
        ]));

        // Storage drives visualization
        let mut storage_art = String::new();
        for i in 0..storage_total {
            if i < storage_filled {
                storage_art.push_str("[▓▓▓]");
            } else {
                storage_art.push_str("[   ]");
            }
        }

        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║ ", accent_style),
            Span::styled(format!("{storage_art:<34}"), storage_style),
            Span::styled("║ ", accent_style),
            Span::styled("║", case_style),
        ]));

        // Spacer
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║                                   ║ ", accent_style),
            Span::styled("║", case_style),
        ]));

        // Network section header
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║ ", accent_style),
            Span::styled("NETWORK       ", net_style),
            Span::styled("                 ", dim_style),
            Span::styled("   ║ ", accent_style),
            Span::styled("║", case_style),
        ]));
        // Network visualization
        let mut net_slots = String::new();
        for i in 0..net_total {
            if i < net_filled {
                net_slots.push_str("◆------        ");
            } else {
                net_slots.push_str("◇------        ");
            }
        }
        let net_art = format!("{net_slots:<34}");
        let net_color = if net_filled > 0 { net_style } else { dim_style };
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║ ", accent_style),
            Span::styled(net_art, net_color),
            Span::styled("║ ", accent_style),
            Span::styled("║", case_style),
        ]));
    } else {
        // No motherboard installed
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║                                   ║ ", dim_style),
            Span::styled("║", case_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(
                " ║       NO MOTHERBOARD INSTALLED    ║ ",
                Style::default().fg(Color::Red),
            ),
            Span::styled("║", case_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║                                   ║ ", dim_style),
            Span::styled("║", case_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║     Purchase a motherboard to     ║ ", dim_style),
            Span::styled("║", case_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║        begin building your PC     ║ ", dim_style),
            Span::styled("║", case_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║                                   ║ ", dim_style),
            Span::styled("║", case_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║                                   ║ ", dim_style),
            Span::styled("║", case_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ║", case_style),
            Span::styled(" ║                                   ║ ", dim_style),
            Span::styled("║", case_style),
        ]));
    }

    // Internal view footer
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" ╚═══════════════════════════════════╝ ", accent_style),
        Span::styled("║", case_style),
    ]));

    // PSU section
    let psu_bar = if functional {
        "█████████████████████████████"
    } else {
        "░░░░░░░░░░░░░░░░░░░░░░░░░░░░░"
    };
    let psu_bar_style = if functional {
        Style::default().fg(Color::Green)
    } else {
        dim_style
    };
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" ┌───────────────────────────────────┐ ", psu_style),
        Span::styled("║", case_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" │ ", psu_style),
        Span::styled("PSU ", psu_style),
        Span::styled(psu_bar.to_string(), psu_bar_style),
        Span::styled(" │ ", psu_style),
        Span::styled("║", case_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ║", case_style),
        Span::styled(" └───────────────────────────────────┘ ", psu_style),
        Span::styled("║", case_style),
    ]));

    // Case bottom
    lines.push(Line::from(vec![
        Span::styled("  ╚", case_style),
        Span::styled("═══════════════════════════════════════", case_style),
        Span::styled("╝", case_style),
    ]));

    // Component info below case
    lines.push(Line::from(vec![Span::styled(String::new(), dim_style)]));

    // Motherboard name
    let mb_display_style = mb.map_or_else(
        || Style::default().fg(Color::Red),
        |m| match m.tier {
            MotherboardTier::Basic => Style::default().fg(Color::DarkGray),
            MotherboardTier::Standard => Style::default().fg(Color::White),
            MotherboardTier::Performance => Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
            MotherboardTier::Enthusiast => Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        },
    );
    lines.push(Line::from(vec![
        Span::styled("  MB: ".to_string(), dim_style),
        Span::styled(mb_name, mb_display_style),
    ]));

    // CPU name
    let cpu_display_style = if cpu_filled {
        cpu_style
    } else {
        Style::default().fg(Color::Red)
    };
    lines.push(Line::from(vec![
        Span::styled("  CPU: ".to_string(), dim_style),
        Span::styled(cpu_name, cpu_display_style),
    ]));

    // Cooler
    let cooler_display_style = if cooler_filled {
        accent_style
    } else {
        Style::default().fg(Color::Red)
    };
    lines.push(Line::from(vec![
        Span::styled("  Cooler: ".to_string(), dim_style),
        Span::styled(cooler_name, cooler_display_style),
    ]));

    // Status line
    let status_text = if functional {
        "SYSTEM READY"
    } else {
        "SYSTEM INCOMPLETE"
    };
    let status_style = if functional {
        Style::default()
            .fg(Color::Green)
            .add_modifier(ratatui::style::Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Red)
            .add_modifier(ratatui::style::Modifier::BOLD)
    };
    lines.push(Line::from(vec![Span::styled(String::new(), dim_style)]));
    lines.push(Line::from(vec![
        Span::styled("  Status: ".to_string(), dim_style),
        Span::styled(status_text.to_string(), status_style),
    ]));

    lines
}
