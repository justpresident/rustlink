mod firewall_buster;
mod password_breaker;

use crate::app::App;

/// Result of a tool tick (progress update)
pub enum ToolTickResult {
    /// Tool is still running, progress updated
    Running(f64),
    /// Tool completed successfully
    Complete,
}

/// Trait for pluggable hacking tools
pub trait Tool: Send + Sync {
    /// Tool name (used in `run <name>`)
    fn name(&self) -> &'static str;

    /// Short description for help
    fn description(&self) -> &'static str;

    /// Check if the tool can run on the current target
    /// Returns Ok(()) if it can run, Err(message) if not
    fn can_run(&self, app: &App, target_ip: &str) -> Result<(), String>;

    /// Called when the tool starts running
    fn on_start(&self, app: &mut App, target_ip: &str) {
        app.log(format!("Running {} on {}...", self.name(), target_ip));
    }

    /// Called each tick while the tool is running
    /// Returns the new progress (0.0 to 100.0)
    fn on_tick(&self, app: &App, target_ip: &str, current_progress: f64) -> f64 {
        let _ = (app, target_ip);
        // Default: increase by 2.5 per tick
        (current_progress + 2.5).min(100.0)
    }

    /// Called when the tool completes (progress reaches 100)
    fn on_complete(&self, app: &mut App, target_ip: &str);
}

/// Registry of all available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self { tools: Vec::new() };
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        self.register(Box::new(password_breaker::PasswordBreaker));
        self.register(Box::new(firewall_buster::FirewallBuster));
    }

    /// Register a new tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Find a tool by name
    pub fn find(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|b| b.as_ref())
    }

    /// Get all registered tools
    pub fn all(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }

    /// Get tool names for completion
    pub fn names(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name().to_string()).collect()
    }

    /// Get tool names that match a prefix
    pub fn completions(&self, prefix: &str) -> Vec<String> {
        self.tools
            .iter()
            .map(|t| t.name())
            .filter(|name| name.starts_with(prefix))
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// State of an active tool
#[derive(Debug, Clone)]
pub struct ActiveTool {
    pub tool_name: String,
    pub target_ip: String,
    pub progress: f64,
}

impl ActiveTool {
    pub fn new(tool_name: &str, target_ip: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            target_ip: target_ip.to_string(),
            progress: 0.0,
        }
    }
}
