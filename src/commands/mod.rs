mod bank;
mod connect;
mod disconnect;
mod files;
mod help;
mod mail;
mod misc;
mod tools;

use crate::app::App;
use crate::tools::ToolRegistry;
use std::collections::HashMap;

/// Result of executing a command
pub enum CommandResult {
    /// Command executed successfully
    Ok,
    /// Command executed but app should quit
    Quit,
    /// Command not found (try next handler)
    NotFound,
    /// Connection state changed, triggers command re-evaluation
    ConnectionChanged {
        old_server_type: Option<crate::model::ServerType>,
        new_server_type: Option<crate::model::ServerType>,
    },
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
    master_commands: HashMap<String, Box<dyn Command>>,
    active_command_names: Vec<String>, // Stores names of currently active commands
    pub tool_registry: ToolRegistry,
    always_active_command_names: Vec<String>, // Commands that cannot be deactivated
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            master_commands: HashMap::new(),
            active_command_names: Vec::new(),
            tool_registry: ToolRegistry::new(),
            always_active_command_names: Vec::new(),
        };
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        // Commands that are always active
        let always_active_cmds: Vec<Box<dyn Command>> = vec![
            Box::new(help::HelpCommand),
            Box::new(help::HelpKeysCommand),
            Box::new(misc::ClearCommand),
            Box::new(connect::ConnectCommand),
            Box::new(disconnect::DisconnectCommand),
            Box::new(tools::RunCommand),
            Box::new(mail::MailCommand),
            Box::new(misc::ExitCommand),
            Box::new(files::LsCommand),
            Box::new(files::ScpCommand),
        ];
        for cmd in always_active_cmds {
            self.always_active_command_names
                .push(cmd.name().to_string());
            self.master_commands.insert(cmd.name().to_string(), cmd);
        }

        // Commands that are context-specific (initially inactive)
        let context_specific_cmds: Vec<Box<dyn Command>> = vec![
            Box::new(bank::AccountInfoCommand),
            Box::new(bank::TransferCommand),
        ];
        for cmd in context_specific_cmds {
            self.master_commands.insert(cmd.name().to_string(), cmd);
        }

        // Initially, all always_active commands are active
        self.active_command_names
            .extend(self.always_active_command_names.clone());
    }

    /// Activate a list of commands
    pub fn activate_commands(&mut self, names: &[&str]) {
        for name in names {
            if self.master_commands.contains_key(*name)
                && !self.active_command_names.contains(&name.to_string())
            {
                self.active_command_names.push(name.to_string());
            }
        }
    }

    /// Deactivate a list of commands, preventing deactivation of always-active commands
    pub fn deactivate_commands(&mut self, names: &[&str]) {
        self.active_command_names.retain(|cmd_name| {
            self.always_active_command_names.contains(cmd_name)
                || !names.contains(&cmd_name.as_str())
        });
    }

    /// Find an active command by name or alias, returning its canonical name
    pub fn find_active_command_name(&self, name: &str) -> Option<String> {
        if self.active_command_names.contains(&name.to_string())
            && self.master_commands.contains_key(name)
        {
            return Some(name.to_string());
        }

        for active_cmd_name in &self.active_command_names {
            if let Some(cmd) = self.master_commands.get(active_cmd_name)
                && cmd.aliases().contains(&name)
            {
                return Some(cmd.name().to_string());
            }
        }
        None
    }

    /// Get all currently active commands
    pub fn all_active(&self) -> Vec<&dyn Command> {
        self.active_command_names
            .iter()
            .filter_map(|name| self.master_commands.get(name).map(|cmd| cmd.as_ref()))
            .collect()
    }

    /// Execute a command by its canonical name
    pub fn execute_command_by_name(
        &mut self,
        app: &mut App,
        cmd_name: &str,
        args: &[&str],
    ) -> CommandResult {
        if let Some(cmd_trait_obj) = self.master_commands.get(cmd_name) {
            cmd_trait_obj.execute(app, args, &*self)
        } else {
            app.log(format!(
                "Error: Internal - Attempted to execute unknown active command '{}'",
                cmd_name
            ));
            CommandResult::Ok
        }
    }

    /// Get command name completions from currently active commands
    pub fn command_completions(&self, prefix: &str) -> Vec<String> {
        let mut completions = Vec::new();
        for cmd_name in &self.active_command_names {
            if let Some(cmd) = self.master_commands.get(cmd_name) {
                if cmd.name().starts_with(prefix) {
                    completions.push(cmd.name().to_string());
                }
                for alias in cmd.aliases() {
                    if alias.starts_with(prefix) {
                        completions.push(alias.to_string());
                    }
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
pub fn execute_input(registry: &mut CommandRegistry, app: &mut App) -> CommandResult {
    let input = app.terminal.input.trim().to_string();
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return CommandResult::Ok;
    }

    let cmd_name = parts[0];
    let args = &parts[1..];

    if let Some(canonical_name) = registry.find_active_command_name(cmd_name) {
        registry.execute_command_by_name(app, &canonical_name, args)
    } else {
        app.log(format!(
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
        let prefix = parts.first().copied().unwrap_or("");
        return registry.command_completions(prefix);
    }

    let Some(canonical_name) = registry.find_active_command_name(parts[0]) else {
        return Vec::new();
    };

    let Some(cmd) = registry.master_commands.get(&canonical_name) else {
        return Vec::new();
    };

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

    cmd.completions_with_tools(app, arg_index, prefix, &registry.tool_registry)
}
