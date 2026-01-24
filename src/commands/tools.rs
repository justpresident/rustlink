use super::{Command, CommandResult};
use crate::app::App;
use crate::commands::CommandRegistry;
use crate::tools::{ActiveTool, ToolRegistry};

pub struct RunCommand;

impl Command for RunCommand {
    fn name(&self) -> &'static str {
        "run"
    }

    fn aliases(&self) -> &[&'static str] {
        &["exec"]
    }

    fn description(&self) -> &'static str {
        "Run a hacking tool on the current target"
    }

    fn usage(&self) -> &'static str {
        "run <tool>"
    }

    fn execute(&self, app: &mut App, args: &[&str], registry: &CommandRegistry) -> CommandResult {
        let tool_registry = &registry.tool_registry;

        let Some(&tool_name) = args.first() else {
            app.logs.push("Usage: run <tool>".into());
            app.logs.push("Available tools:".into());
            for tool in tool_registry.all() {
                app.logs
                    .push(format!("  {} - {}", tool.name(), tool.description()));
            }
            return CommandResult::Ok;
        };

        let Some(target) = app.target_ip.clone() else {
            app.logs.push("Not connected to any server.".into());
            return CommandResult::Ok;
        };

        // Find the tool in the registry
        let Some(tool) = tool_registry.find(tool_name) else {
            app.logs.push(format!("Tool '{}' not found.", tool_name));
            app.logs.push("Available tools:".into());
            for t in tool_registry.all() {
                app.logs
                    .push(format!("  {} - {}", t.name(), t.description()));
            }
            return CommandResult::Ok;
        };

        // Check if already running a tool
        if app.active_tool.is_some() {
            app.logs
                .push("A tool is already running. Wait for it to complete.".into());
            return CommandResult::Ok;
        }

        // Check if tool can run on this target
        if let Err(msg) = tool.can_run(app, &target) {
            app.logs.push(msg);
            return CommandResult::Ok;
        }

        // Start the tool
        tool.on_start(app, &target);
        app.active_tool = Some(ActiveTool::new(tool_name, &target));

        CommandResult::Ok
    }

    fn completions(&self, _app: &App, _arg_index: usize, _prefix: &str) -> Vec<String> {
        // Basic completions without tool registry access
        Vec::new()
    }

    fn completions_with_tools(
        &self,
        _app: &App,
        _arg_index: usize,
        prefix: &str,
        tool_registry: &ToolRegistry,
    ) -> Vec<String> {
        tool_registry.completions(prefix)
    }
}
