use crate::model::firewall::Firewall;
use crate::model::{File, FileSystem, Server};
use std::collections::HashMap;

/// Game world containing all servers
pub struct GameWorld {
    pub servers: HashMap<String, Server>,
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl GameWorld {
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

        Self { servers }
    }

    pub fn get(&self, ip: &str) -> Option<&Server> {
        self.servers.get(ip)
    }

    pub fn get_mut(&mut self, ip: &str) -> Option<&mut Server> {
        self.servers.get_mut(ip)
    }

    pub fn contains(&self, ip: &str) -> bool {
        self.servers.contains_key(ip)
    }

    pub fn is_server_illegal(&self, ip: &str) -> bool {
        self.servers
            .get(ip)
            .map(|s| s.is_locked || s.firewall.is_some())
            .unwrap_or(false)
    }
}
