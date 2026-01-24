mod connect;
mod disconnect;
mod files;
mod help;
mod mail;
mod misc;
mod tools;

use crate::app::App;
use crate::tools::ToolRegistry;

/// Result of executing a command
pub enum CommandResult {
    /// Command executed successfully
    Ok,
    /// Command executed but app should quit
    Quit,
    /// Command not found (try next handler)
    NotFound,
}

/// Trait for pluggable commands
pub trait Command: Send + Sync {
    /// Primary command name
    fn name(&self) -> &'static str;

    /// Optional command aliases
    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    /// Short description for help
    fn description(&self) -> &'static str;

    /// Usage pattern (e.g., "connect <ip>")
    fn usage(&self) -> &'static str {
        self.name()
    }

    /// Execute the command with given arguments
    /// The registry is provided for commands that need to introspect available commands/tools
    fn execute(&self, app: &mut App, args: &[&str], registry: &CommandRegistry) -> CommandResult;

    /// Get completions for the given argument position and prefix
    fn completions(&self, app: &App, arg_index: usize, prefix: &str) -> Vec<String> {
        let _ = (app, arg_index, prefix);
        Vec::new()
    }

    /// Get completions with access to tool registry
    fn completions_with_tools(
        &self,
        app: &App,
        arg_index: usize,
        prefix: &str,
        tool_registry: &ToolRegistry,
    ) -> Vec<String> {
        let _ = tool_registry;
        self.completions(app, arg_index, prefix)
    }
}

/// Registry of all available commands and tools
pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
    pub tool_registry: ToolRegistry,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: Vec::new(),
            tool_registry: ToolRegistry::new(),
        };
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        // Register all built-in commands
        self.register(Box::new(help::HelpCommand));
        self.register(Box::new(help::HelpKeysCommand));
        self.register(Box::new(misc::ClearCommand));
        self.register(Box::new(connect::ConnectCommand));
        self.register(Box::new(disconnect::DisconnectCommand));
        self.register(Box::new(files::LsCommand));
        self.register(Box::new(files::ScpCommand));
        self.register(Box::new(tools::RunCommand));
        self.register(Box::new(mail::MailCommand));
        self.register(Box::new(misc::ExitCommand));
    }

    /// Register a new command
    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    /// Find a command by name or alias
    pub fn find(&self, name: &str) -> Option<&dyn Command> {
        self.commands
            .iter()
            .find(|cmd| cmd.name() == name || cmd.aliases().contains(&name))
            .map(|b| b.as_ref())
    }

    /// Get all registered commands
    pub fn all(&self) -> &[Box<dyn Command>] {
        &self.commands
    }

    /// Get command name completions
    pub fn command_completions(&self, prefix: &str) -> Vec<String> {
        let mut completions = Vec::new();
        for cmd in &self.commands {
            if cmd.name().starts_with(prefix) {
                completions.push(cmd.name().to_string());
            }
            for alias in cmd.aliases() {
                if alias.starts_with(prefix) {
                    completions.push(alias.to_string());
                }
            }
        }
        completions
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a command line input
pub fn execute_input(registry: &CommandRegistry, app: &mut App) -> CommandResult {
    let input = app.input.trim().to_string();
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return CommandResult::Ok;
    }

    let cmd_name = parts[0];
    let args = &parts[1..];

    if let Some(cmd) = registry.find(cmd_name) {
        cmd.execute(app, args, registry)
    } else {
        app.logs.push(format!(
            "Unknown command: {}. Type 'help' for available commands.",
            cmd_name
        ));
        CommandResult::Ok
    }
}

/// Get completions for current input
pub fn get_completions(registry: &CommandRegistry, app: &App, input: &str) -> Vec<String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let ends_with_space = input.ends_with(' ');

    if parts.is_empty() || (parts.len() == 1 && !ends_with_space) {
        // Complete command name
        let prefix = parts.first().copied().unwrap_or("");
        registry.command_completions(prefix)
    } else {
        // Complete command arguments
        let cmd_name = parts[0];
        if let Some(cmd) = registry.find(cmd_name) {
            let arg_index = if ends_with_space {
                parts.len() - 1
            } else {
                parts.len() - 2
            };
            let prefix = if ends_with_space {
                ""
            } else {
                parts.last().copied().unwrap_or("")
            };
            // Use completions_with_tools to allow access to tool registry
            cmd.completions_with_tools(app, arg_index, prefix, &registry.tool_registry)
        } else {
            Vec::new()
        }
    }
}
