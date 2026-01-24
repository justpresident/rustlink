use crate::model::{Server, ServerType};
use crate::tools::ActiveTool;
use std::collections::HashMap;

/// Connection state for network routing and trace mechanics
pub struct Connection {
    pub path: Vec<String>,
    pub target_ip: Option<String>,
    pub trace_percentage: f64,
    pub is_tracing: bool,
    pub active_tool: Option<ActiveTool>,
    pub connected_server_type: Option<ServerType>,
}

impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}

impl Connection {
    pub fn new() -> Self {
        Self {
            path: vec!["127.0.0.1".into()],
            target_ip: None,
            trace_percentage: 0.0,
            is_tracing: false,
            active_tool: None,
            connected_server_type: Some(ServerType::Home),
        }
    }

    pub fn reset(&mut self) {
        self.path = vec!["127.0.0.1".into()];
        self.target_ip = None;
        self.is_tracing = false;
        self.trace_percentage = 0.0;
        self.active_tool = None;
        self.connected_server_type = Some(ServerType::Home);
    }

    pub fn connect(&mut self, ip: &str, is_illegal: bool) {
        self.path.push(ip.to_string());
        self.target_ip = Some(ip.to_string());
        if is_illegal {
            self.is_tracing = true;
        }
        // NOTE: connected_server_type will be updated by the ConnectCommand after it gets the server details from App.world
    }

    pub fn is_in_path(&self, ip: &str) -> bool {
        self.path.contains(&ip.to_string())
    }

    /// Find the index of the first illegal node in the connection path
    fn first_illegal_index(&self, servers: &HashMap<String, Server>) -> Option<usize> {
        self.path.iter().position(|ip| {
            servers
                .get(ip)
                .map(|s| s.is_locked || s.firewall.is_some())
                .unwrap_or(false)
        })
    }

    /// Calculate the total distance of the connection path up to first illegal node
    fn connection_distance(&self, servers: &HashMap<String, Server>) -> f64 {
        if self.path.len() < 2 {
            return 0.0;
        }

        let end_index = self
            .first_illegal_index(servers)
            .map(|i| i + 1)
            .unwrap_or(self.path.len());
        let relevant_path = &self.path[..end_index];

        let mut total_distance = 0.0;
        for window in relevant_path.windows(2) {
            if let (Some(server_a), Some(server_b)) =
                (servers.get(&window[0]), servers.get(&window[1]))
            {
                let (x1, y1) = server_a.coords;
                let (x2, y2) = server_b.coords;
                let dx = x2 - x1;
                let dy = y2 - y1;
                total_distance += (dx * dx + dy * dy).sqrt();
            }
        }
        total_distance
    }

    /// Calculate the trace speed based on hop count and total distance
    pub fn trace_speed(&self, servers: &HashMap<String, Server>) -> f64 {
        let base_speed = 0.5;
        let hop_count = self.first_illegal_index(servers).unwrap_or(0);
        let distance = self.connection_distance(servers);

        let hop_factor = 1.0 / (1.0 + hop_count as f64 * 0.4);
        let distance_factor = 1.0 / (1.0 + distance / 500.0);

        base_speed * hop_factor * distance_factor
    }

    /// Advance trace and return true if terminal was compromised
    pub fn tick_trace(&mut self, servers: &HashMap<String, Server>) -> bool {
        if self.is_tracing {
            if self.trace_percentage < 100.0 {
                self.trace_percentage += self.trace_speed(servers);
                false
            } else {
                true // Compromised
            }
        } else {
            false
        }
    }
}
