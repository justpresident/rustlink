use crate::model::firewall::Firewall;
use std::{
    collections::HashMap,
    time::Instant,
};
use crate::model::{File, FileSystem, Mail, Mission, Server, ToolState, ToolType};

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
    pub inbox: Vec<Mail>,
    pub animation_tick: u64,

    // Readline-like input state
    pub cursor_pos: usize,
    pub command_history: Vec<String>,
    pub history_index: Option<usize>,
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
                firewall: Some(Firewall {
                    is_active: true,
                    strength: 80,
                }),
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
                firewall: None,
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
                firewall: None,
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
                firewall: Some(Firewall {
                    is_active: true,
                    strength: 50,
                }),
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
            inventory: vec!["PasswordBreaker".into(), "FileManager".into(), "FirewallBuster".into()],
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
            inbox: vec![
                Mail {
                    id: 1,
                    sender: "admin@uplink.net".into(),
                    subject: "Welcome to Uplink!".into(),
                    body: "Welcome, Agent. Your journey into the digital underworld begins now. Good luck.".into(),
                    is_read: false,
                },
                Mail {
                    id: 2,
                    sender: "intern@globaltrust.com".into(),
                    subject: "Urgent: System Vulnerability".into(),
                    body: "We've detected a critical vulnerability in our systems. Please assist immediately.".into(),
                    is_read: false,
                },
            ],
            animation_tick: 0,
            cursor_pos: 0,
            command_history: Vec::new(),
            history_index: None,
        }
    }

    pub fn on_tick(&mut self) {
        // Handle Cracking Progress
        if let ToolState::Running {
            ref mut progress,
            ref target_ip,
            ref tool_type,
        } = self.active_tool
        {
            *progress += 2.5; // Speed of the tool
            if *progress >= 100.0 {
                match tool_type {
                    ToolType::PasswordBreaker => {
                        self.logs
                            .push(format!("SUCCESS: Target {} bypassed.", target_ip));
                        if let Some(server) = self.servers.get_mut(target_ip) {
                            server.is_locked = false;
                        }
                    }
                    ToolType::FirewallBuster => {
                        self.logs
                            .push(format!("SUCCESS: Firewall on {} disabled.", target_ip));
                        if let Some(server) = self.servers.get_mut(target_ip) {
                            if let Some(firewall) = &mut server.firewall {
                                firewall.is_active = false;
                            }
                        }
                    }
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
        self.animation_tick = self.animation_tick.wrapping_add(1);
    }

    pub fn reset_connection(&mut self) {
        self.connection_path = vec!["127.0.0.1".into()];
        self.target_ip = None;
        self.is_tracing = false;
        self.trace_percentage = 0.0;
        self.active_tool = ToolState::Idle;
        self.credits = self.credits.saturating_sub(100); // Penalty for getting traced
    }

    pub fn check_missions(&mut self, filename: &str) {
        for m in self.missions.iter_mut() {
            if !m.is_complete && m.target_file == filename {
                m.is_complete = true;
                self.credits += m.reward;
                self.logs.push(format!("MISSION COMPLETE: +{}c", m.reward));
            }
        }
    }

    // Input handling methods
    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.history_index = None;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_pos = self.input.len();
    }

    pub fn clear_line(&mut self) {
        self.input.clear();
        self.cursor_pos = 0;
    }

    pub fn delete_word(&mut self) {
        // Delete word before cursor (Ctrl+W)
        while self.cursor_pos > 0 && self.input.chars().nth(self.cursor_pos - 1) == Some(' ') {
            self.cursor_pos -= 1;
            self.input.remove(self.cursor_pos);
        }
        while self.cursor_pos > 0 && self.input.chars().nth(self.cursor_pos - 1) != Some(' ') {
            self.cursor_pos -= 1;
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }
        match self.history_index {
            None => {
                self.history_index = Some(self.command_history.len() - 1);
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
            }
            _ => return,
        }
        if let Some(idx) = self.history_index {
            self.input = self.command_history[idx].clone();
            self.cursor_pos = self.input.len();
        }
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.command_history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.input = self.command_history[idx + 1].clone();
                self.cursor_pos = self.input.len();
            }
            Some(_) => {
                self.history_index = None;
                self.input.clear();
                self.cursor_pos = 0;
            }
            None => {}
        }
    }

    pub fn save_to_history(&mut self) {
        let trimmed = self.input.trim().to_string();
        if !trimmed.is_empty() {
            // Don't add duplicates consecutively
            if self.command_history.last() != Some(&trimmed) {
                self.command_history.push(trimmed);
            }
        }
        self.history_index = None;
    }

    pub fn autocomplete(&mut self) {
        let parts: Vec<&str> = self.input.split_whitespace().collect();
        let input_ends_with_space = self.input.ends_with(' ');

        let completions: Vec<String> = if parts.is_empty() || (parts.len() == 1 && !input_ends_with_space) {
            // Complete command
            let prefix = parts.first().map(|s| *s).unwrap_or("");
            self.get_command_completions(prefix)
        } else {
            // Complete argument based on command
            let cmd = parts[0];
            let arg_prefix = if input_ends_with_space { "" } else { parts.last().map(|s| *s).unwrap_or("") };
            self.get_argument_completions(cmd, arg_prefix)
        };

        if completions.len() == 1 {
            // Single match - complete it
            self.apply_completion(&completions[0]);
        } else if completions.len() > 1 {
            // Multiple matches - show them
            self.logs.push(format!("Completions: {}", completions.join(" ")));
            // Find common prefix and apply it
            if let Some(common) = Self::common_prefix(&completions) {
                self.apply_completion(&common);
            }
        }
    }

    fn get_command_completions(&self, prefix: &str) -> Vec<String> {
        let commands = ["help", "connect", "ls", "scp", "run", "inbox", "read", "delete", "disconnect", "clear", "exit"];
        commands
            .iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .map(|s| s.to_string())
            .collect()
    }

    fn get_argument_completions(&self, cmd: &str, prefix: &str) -> Vec<String> {
        match cmd {
            "connect" => {
                // Complete IP addresses
                self.servers
                    .keys()
                    .filter(|ip| ip.starts_with(prefix))
                    .cloned()
                    .collect()
            }
            "run" => {
                // Complete tool names
                self.inventory
                    .iter()
                    .filter(|tool| tool.starts_with(prefix))
                    .cloned()
                    .collect()
            }
            "scp" => {
                // Complete filenames from current target
                if let Some(target) = &self.target_ip {
                    if let Some(server) = self.servers.get(target) {
                        return server
                            .fs
                            .files
                            .iter()
                            .map(|f| &f.name)
                            .filter(|name| name.starts_with(prefix))
                            .cloned()
                            .collect();
                    }
                }
                Vec::new()
            }
            "read" | "delete" => {
                // Complete mail IDs
                self.inbox
                    .iter()
                    .map(|m| m.id.to_string())
                    .filter(|id| id.starts_with(prefix))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn apply_completion(&mut self, completion: &str) {
        let parts: Vec<&str> = self.input.split_whitespace().collect();
        let input_ends_with_space = self.input.ends_with(' ');

        if parts.is_empty() || (parts.len() == 1 && !input_ends_with_space) {
            // Completing command
            self.input = completion.to_string() + " ";
        } else {
            // Completing argument - rebuild input
            let mut new_input: Vec<&str> = parts[..parts.len() - 1].to_vec();
            if !input_ends_with_space {
                // Replace last partial arg
            } else {
                new_input = parts.to_vec();
            }
            self.input = new_input.join(" ");
            if !self.input.is_empty() {
                self.input.push(' ');
            }
            self.input.push_str(completion);
            self.input.push(' ');
        }
        self.cursor_pos = self.input.len();
    }

    fn common_prefix(strings: &[String]) -> Option<String> {
        if strings.is_empty() {
            return None;
        }
        let first = &strings[0];
        let mut prefix_len = first.len();
        for s in &strings[1..] {
            prefix_len = first
                .chars()
                .zip(s.chars())
                .take_while(|(a, b)| a == b)
                .count()
                .min(prefix_len);
        }
        if prefix_len > 0 {
            Some(first[..first.char_indices().nth(prefix_len).map(|(i, _)| i).unwrap_or(first.len())].to_string())
        } else {
            None
        }
    }
}
