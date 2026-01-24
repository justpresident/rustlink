use super::{Command, CommandResult};
use crate::app::App;
use crate::commands::CommandRegistry;
use crate::model::{ToolState, ToolType};

pub struct RunCommand;

impl Command for RunCommand {
    fn name(&self) -> &'static str {
        "run"
    }

    fn aliases(&self) -> &[&'static str] {
        &["exec", "start"]
    }

    fn description(&self) -> &'static str {
        "Run a hacking tool on the current target"
    }

    fn usage(&self) -> &'static str {
        "run <tool>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        let Some(&tool_name) = args.first() else {
            app.logs.push("Usage: run <tool>".into());
            app.logs
                .push(format!("Available tools: {}", app.inventory.join(", ")));
            return CommandResult::Ok;
        };

        let Some(target) = app.target_ip.clone() else {
            app.logs.push("Not connected to any server.".into());
            return CommandResult::Ok;
        };

        // Check if tool exists in inventory
        if !app.inventory.iter().any(|t| t == tool_name) {
            app.logs
                .push(format!("Tool '{}' not found in inventory.", tool_name));
            app.logs
                .push(format!("Available tools: {}", app.inventory.join(", ")));
            return CommandResult::Ok;
        }

        // Check if already running a tool
        if let ToolState::Running { .. } = app.active_tool {
            app.logs
                .push("A tool is already running. Wait for it to complete.".into());
            return CommandResult::Ok;
        }

        match tool_name {
            "PasswordBreaker" => {
                let server = &app.servers[&target];
                if !server.is_locked {
                    app.logs
                        .push("Server is not locked. No need for PasswordBreaker.".into());
                    return CommandResult::Ok;
                }
                app.active_tool = ToolState::Running {
                    progress: 0.0,
                    target_ip: target.clone(),
                    tool_type: ToolType::PasswordBreaker,
                };
                app.logs
                    .push(format!("Running PasswordBreaker on {}...", target));
            }
            "FirewallBuster" => {
                let server = &app.servers[&target];
                if let Some(firewall) = &server.firewall {
                    if !firewall.is_active {
                        app.logs.push("Firewall is already disabled.".into());
                        return CommandResult::Ok;
                    }
                    app.active_tool = ToolState::Running {
                        progress: 0.0,
                        target_ip: target.clone(),
                        tool_type: ToolType::FirewallBuster,
                    };
                    app.logs
                        .push(format!("Running FirewallBuster on {}...", target));
                } else {
                    app.logs.push("No firewall detected on target.".into());
                }
            }
            _ => {
                app.logs
                    .push(format!("Tool '{}' is not implemented yet.", tool_name));
            }
        }

        CommandResult::Ok
    }

    fn completions(&self, app: &App, _arg_index: usize, prefix: &str) -> Vec<String> {
        app.inventory
            .iter()
            .filter(|tool| tool.starts_with(prefix))
            .cloned()
            .collect()
    }
}
