use std::{
    collections::HashMap,
    time::Instant,
};
use crate::model::*;

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
}
